//! API tests for schema2object Rust — mirrors Python pytest suite.
use schema2object::{validate, ObjectTree, ValidationError};
use serde_json::{json, Value};

// ─── validate() ───────────────────────────────────────────────────────────────

#[test]
fn validate_valid_integer() {
    assert!(validate(&json!(42), &json!({"type": "integer"}), None).is_ok());
}

#[test]
fn validate_invalid_type() {
    assert!(validate(&json!("hello"), &json!({"type": "integer"}), None).is_err());
}

#[test]
fn validate_valid_string_min_length() {
    assert!(validate(&json!("hi"), &json!({"type": "string", "minLength": 2}), None).is_ok());
}

#[test]
fn validate_below_minimum() {
    assert!(validate(&json!(5), &json!({"type": "integer", "minimum": 10}), None).is_err());
}

#[test]
fn validate_valid_object() {
    let schema = json!({"type": "object", "properties": {"name": {"type": "string"}}, "required": ["name"]});
    assert!(validate(&json!({"name": "Alice"}), &schema, None).is_ok());
}

#[test]
fn validate_missing_required() {
    let schema = json!({"type": "object", "required": ["name"]});
    assert!(validate(&json!({}), &schema, None).is_err());
}

#[test]
fn validate_boolean_schema_true() {
    assert!(validate(&json!(42), &json!(true), None).is_ok());
}

#[test]
fn validate_boolean_schema_false() {
    assert!(validate(&json!(42), &json!(false), None).is_err());
}

// ─── ObjectTree construction ──────────────────────────────────────────────────

#[test]
fn construction_valid_data() {
    let schema = json!({"type": "object", "properties": {"name": {"type": "string"}}});
    let o = ObjectTree::try_new(json!({"name": "Alice"}), schema, None).unwrap();
    assert_eq!(o.get("name"), Some(&json!("Alice")));
}

#[test]
fn construction_invalid_data_fails() {
    let result = ObjectTree::try_new(json!("hello"), json!({"type": "integer"}), None);
    assert!(result.is_err());
}

#[test]
fn construction_defaults_applied() {
    let schema = json!({
        "type": "object",
        "properties": {
            "status": {"type": "string", "default": "pending"},
            "count":  {"type": "integer", "default": 0}
        }
    });
    let o = ObjectTree::try_new(json!({}), schema, None).unwrap();
    assert_eq!(o.get("status"), Some(&json!("pending")));
    assert_eq!(o.get("count"), Some(&json!(0)));
}

#[test]
fn construction_boolean_schema_false_fails() {
    assert!(ObjectTree::try_new(json!(42), json!(false), None).is_err());
}

#[test]
fn construction_boolean_schema_true() {
    let o = ObjectTree::try_new(json!(42), json!(true), None).unwrap();
    assert_eq!(o.value(), &json!(42));
}

// ─── get / set ────────────────────────────────────────────────────────────────

#[test]
fn get_property() {
    let schema = json!({"type": "object", "properties": {"name": {"type": "string"}}});
    let o = ObjectTree::try_new(json!({"name": "Bob"}), schema, None).unwrap();
    assert_eq!(o.get("name"), Some(&json!("Bob")));
}

#[test]
fn set_property_valid() {
    let schema = json!({"type": "object", "properties": {"name": {"type": "string"}}});
    let mut o = ObjectTree::try_new(json!({"name": "Bob"}), schema, None).unwrap();
    o.set("name", json!("Alice")).unwrap();
    assert_eq!(o.get("name"), Some(&json!("Alice")));
}

#[test]
fn set_property_invalid_fails() {
    let schema = json!({"type": "object", "properties": {"name": {"type": "string"}}});
    let mut o = ObjectTree::try_new(json!({"name": "Bob"}), schema, None).unwrap();
    assert!(o.set("name", json!(123)).is_err());
}

#[test]
fn set_value_valid() {
    let schema = json!({"type": "object", "properties": {
        "name": {"type": "string"},
        "age":  {"type": "integer", "minimum": 0}
    }});
    let mut o = ObjectTree::try_new(json!({}), schema, None).unwrap();
    o.set_value(json!({"name": "Alice", "age": 30})).unwrap();
    assert_eq!(o.get("name"), Some(&json!("Alice")));
}

#[test]
fn set_value_invalid_fails() {
    let schema = json!({"type": "object", "properties": {"age": {"type": "integer"}}});
    let mut o = ObjectTree::try_new(json!({}), schema, None).unwrap();
    assert!(o.set_value(json!({"age": "not-a-number"})).is_err());
}

// ─── to_dict ──────────────────────────────────────────────────────────────────

#[test]
fn to_dict_excludes_unknown_fields() {
    let schema = json!({"type": "object", "properties": {"name": {"type": "string"}}});
    let o = ObjectTree::try_new(json!({"name": "Alice", "extra": "ignored"}), schema, None).unwrap();
    assert_eq!(o.to_dict(), json!({"name": "Alice"}));
}

#[test]
fn to_dict_only_present_defined_fields() {
    let schema = json!({"type": "object", "properties": {
        "a": {"type": "string"},
        "b": {"type": "string"}
    }});
    let o = ObjectTree::try_new(json!({"a": "x"}), schema, None).unwrap();
    assert_eq!(o.to_dict(), json!({"a": "x"}));
}

#[test]
fn to_dict_scalar() {
    let o = ObjectTree::try_new(json!(42), json!({"type": "integer"}), None).unwrap();
    assert_eq!(o.to_dict(), json!(42));
}

// ─── value ────────────────────────────────────────────────────────────────────

#[test]
fn value_scalar_get() {
    let o = ObjectTree::try_new(json!(42), json!({"type": "integer"}), None).unwrap();
    assert_eq!(o.value(), &json!(42));
}

#[test]
fn value_object_get() {
    let schema = json!({"type": "object", "properties": {"x": {"type": "integer"}}});
    let o = ObjectTree::try_new(json!({"x": 1}), schema, None).unwrap();
    assert_eq!(o.value(), &json!({"x": 1}));
}

// ─── oneOf ────────────────────────────────────────────────────────────────────

#[test]
fn one_of_match() {
    let schema = json!({"oneOf": [{"type": "string"}, {"type": "integer"}]});
    let o = ObjectTree::try_new(json!(42), schema, None).unwrap();
    let result = o.one_of().unwrap();
    assert_eq!(result.value(), &json!(42));
}

#[test]
fn one_of_no_match_construction_fails() {
    let schema = json!({"oneOf": [{"type": "string"}, {"type": "array"}]});
    assert!(ObjectTree::try_new(json!(42), schema, None).is_err());
}

// ─── anyOf ────────────────────────────────────────────────────────────────────

#[test]
fn any_of_match() {
    let schema = json!({"anyOf": [{"type": "string"}, {"type": "integer"}]});
    let o = ObjectTree::try_new(json!(42), schema, None).unwrap();
    let results = o.any_of().unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].value(), &json!(42));
}

#[test]
fn any_of_no_match_construction_fails() {
    let schema = json!({"anyOf": [{"type": "string"}, {"type": "array"}]});
    assert!(ObjectTree::try_new(json!(42), schema, None).is_err());
}

// ─── allOf ────────────────────────────────────────────────────────────────────

#[test]
fn all_of_merge() {
    let schema = json!({"allOf": [
        {"type": "object", "properties": {"a": {"type": "string"}}},
        {"type": "object", "properties": {"b": {"type": "integer"}}}
    ]});
    let o = ObjectTree::try_new(json!({"a": "x", "b": 1}), schema, None).unwrap();
    let result = o.all_of();
    assert_eq!(result.value(), &json!({"a": "x", "b": 1}));
}

// ─── notOf ────────────────────────────────────────────────────────────────────

#[test]
fn not_of_passes() {
    let o = ObjectTree::try_new(json!(42), json!({"not": {"type": "string"}}), None).unwrap();
    assert!(o.not_of());
}

#[test]
fn not_of_fails_construction() {
    assert!(ObjectTree::try_new(json!("hello"), json!({"not": {"type": "string"}}), None).is_err());
}

// ─── ifThen ───────────────────────────────────────────────────────────────────

#[test]
fn if_then_branch() {
    let schema = json!({
        "if":   {"properties": {"country": {"const": "US"}}},
        "then": {"properties": {"zip": {"type": "string"}}},
        "else": {"properties": {"zip": {"type": "integer"}}}
    });
    let o = ObjectTree::try_new(json!({"country": "US", "zip": "12345"}), schema, None).unwrap();
    let result = o.if_then();
    assert_eq!(result.value(), &json!({"country": "US", "zip": "12345"}));
}

#[test]
fn if_then_no_if_returns_same_value() {
    let o = ObjectTree::try_new(json!(42), json!({"type": "integer"}), None).unwrap();
    let result = o.if_then();
    assert_eq!(result.value(), &json!(42));
}

// ─── project ──────────────────────────────────────────────────────────────────

#[test]
fn project_filters_to_schema() {
    let schema = json!({"type": "object", "properties": {"a": {"type": "string"}}});
    let o = ObjectTree::try_new(json!({"a": "x", "b": "y"}), schema, None).unwrap();
    let p = o.project().unwrap();
    assert_eq!(p.to_dict(), json!({"a": "x"}));
}

#[test]
fn project_non_object_fails() {
    let o = ObjectTree::try_new(json!(42), json!({"type": "integer"}), None).unwrap();
    assert!(o.project().is_err());
}

// ─── withDefaults ─────────────────────────────────────────────────────────────

#[test]
fn with_defaults_fills_missing() {
    let schema = json!({"type": "object", "properties": {
        "status": {"type": "string", "default": "active"}
    }});
    let o = ObjectTree::try_new(json!({}), schema, None).unwrap();
    let result = o.with_defaults();
    assert_eq!(result.get("status"), Some(&json!("active")));
}

// ─── get_schema / get_extensions ──────────────────────────────────────────────

#[test]
fn get_schema_root() {
    let schema = json!({"type": "object", "properties": {"name": {"type": "string"}}});
    let o = ObjectTree::try_new(json!({}), schema, None).unwrap();
    assert_eq!(o.get_schema("").unwrap()["type"], json!("object"));
}

#[test]
fn get_schema_nested() {
    let schema = json!({"type": "object", "properties": {"name": {"type": "string", "minLength": 1}}});
    let o = ObjectTree::try_new(json!({"name": "x"}), schema, None).unwrap();
    let s = o.get_schema("name").unwrap();
    assert_eq!(s["type"], json!("string"));
    assert_eq!(s["minLength"], json!(1));
}

#[test]
fn get_extensions() {
    let schema = json!({"type": "object", "x-foo": "bar", "properties": {}});
    let o = ObjectTree::try_new(json!({}), schema, None).unwrap();
    let ext = o.get_extensions("");
    assert_eq!(ext.get("x-foo"), Some(&json!("bar")));
}

// ─── $ref ─────────────────────────────────────────────────────────────────────

#[test]
fn ref_definitions() {
    let schema = json!({
        "type": "object",
        "properties": {"addr": {"$ref": "#/definitions/Address"}},
        "definitions": {
            "Address": {"type": "object", "properties": {"city": {"type": "string"}}}
        }
    });
    let o = ObjectTree::try_new(json!({"addr": {"city": "Taipei"}}), schema, None).unwrap();
    assert_eq!(o.get("addr"), Some(&json!({"city": "Taipei"})));
}

#[test]
fn ref_invalid_fails() {
    let schema = json!({
        "type": "object",
        "properties": {"n": {"$ref": "#/definitions/Int"}},
        "definitions": {"Int": {"type": "integer"}}
    });
    assert!(ObjectTree::try_new(json!({"n": "not-int"}), schema, None).is_err());
}

#[test]
fn ref_recursive() {
    let schema = json!({
        "type": "object",
        "properties": {
            "name": {"type": "string"},
            "child": {"$ref": "#"}
        }
    });
    let o = ObjectTree::try_new(
        json!({"name": "root", "child": {"name": "leaf"}}),
        schema, None
    ).unwrap();
    assert_eq!(o.get("name"), Some(&json!("root")));
}

// ─── contains ────────────────────────────────────────────────────────────────

#[test]
fn contains_match() {
    let schema = json!({"type": "array", "contains": {"type": "integer"}});
    let o = ObjectTree::try_new(json!(["a", 1, "b"]), schema, None).unwrap();
    assert_eq!(o.contains(), Some(true));
}

#[test]
fn contains_no_match_fails_construction() {
    // contains is a validation keyword — fails if no element matches
    let schema = json!({"contains": {"type": "integer"}});
    assert!(ObjectTree::try_new(json!(["a", "b"]), schema, None).is_err());
}

#[test]
fn contains_non_array_returns_none() {
    let o = ObjectTree::try_new(json!(42), json!({"type": "integer"}), None).unwrap();
    assert_eq!(o.contains(), None);
}
