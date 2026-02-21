//! Integration tests: oneOf, anyOf, allOf, not, if/then/else, project, contains.

use schema_value::SchemaValue;
use serde_json::json;

// --- oneOf (XOR) ---

#[test]
fn one_of_selects_unique_branch() {
    let schema = json!({
        "oneOf": [
            {"properties": {"kind": {"const": "circle"}, "radius": {"type": "number"}}},
            {"properties": {"kind": {"const": "rect"}, "width": {"type": "number"}}}
        ]
    });
    let sv = SchemaValue::new(json!({"kind": "circle", "radius": 5}), schema);
    let branch = sv.one_of().unwrap();
    assert!(branch.schema().is_some());
    let s = branch.schema().unwrap();
    assert!(s.get("properties").unwrap().get("radius").is_some());
}

#[test]
fn one_of_fails_on_zero_matches() {
    let schema = json!({
        "oneOf": [
            {"properties": {"x": {"const": "a"}}},
            {"properties": {"x": {"const": "b"}}}
        ]
    });
    let sv = SchemaValue::new(json!({"x": "z"}), schema);
    assert!(sv.one_of().is_err());
}

#[test]
fn one_of_fails_on_multiple_matches() {
    // Both branches accept integer x
    let schema = json!({
        "oneOf": [
            {"properties": {"x": {"type": "integer"}}},
            {"properties": {"x": {"type": "number"}}}
        ]
    });
    let sv = SchemaValue::new(json!({"x": 42}), schema);
    assert!(sv.one_of().is_err());
}

// --- anyOf (OR) ---

#[test]
fn any_of_returns_all_matches() {
    let schema = json!({
        "anyOf": [
            {"properties": {"x": {"type": "integer"}}},
            {"properties": {"x": {"type": "number"}}},
            {"properties": {"x": {"const": "nope"}}}
        ]
    });
    let sv = SchemaValue::new(json!({"x": 42}), schema);
    let branches = sv.any_of().unwrap();
    assert_eq!(branches.len(), 2); // integer + number match
}

#[test]
fn any_of_fails_on_no_matches() {
    let schema = json!({
        "anyOf": [
            {"properties": {"x": {"const": 1}}},
            {"properties": {"x": {"const": 2}}}
        ]
    });
    let sv = SchemaValue::new(json!({"x": 99}), schema);
    assert!(sv.any_of().is_err());
}

// --- allOf (AND) ---

#[test]
fn all_of_merges_schemas() {
    let schema = json!({
        "properties": {"id": {"type": "integer"}},
        "allOf": [
            {"properties": {"name": {"type": "string"}}, "required": ["name"]},
            {"properties": {"age": {"type": "integer"}}, "required": ["age"]}
        ]
    });
    let sv = SchemaValue::new(json!({"id": 1, "name": "Alice", "age": 30}), schema);
    let merged = sv.all_of();
    let ms = merged.schema().unwrap();
    let props = ms.get("properties").unwrap();
    // All three properties merged
    assert!(props.get("id").is_some());
    assert!(props.get("name").is_some());
    assert!(props.get("age").is_some());
    // Required merged without duplicates
    let req: Vec<_> = ms.get("required").unwrap().as_array().unwrap()
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    assert!(req.contains(&"name"));
    assert!(req.contains(&"age"));
}

// --- not ---

#[test]
fn not_of_rejects_matching() {
    let schema = json!({"not": {"type": "string"}});
    let sv = SchemaValue::new(json!("hello"), schema);
    assert!(!sv.not_of(None));
}

#[test]
fn not_of_accepts_non_matching() {
    let schema = json!({"not": {"type": "string"}});
    let sv = SchemaValue::new(json!(42), schema);
    assert!(sv.not_of(None));
}

#[test]
fn not_of_with_explicit_schema() {
    let sv = SchemaValue::without_schema(json!([1, 2]));
    assert!(sv.not_of(Some(&json!({"type": "object"}))));
    assert!(!sv.not_of(Some(&json!({"type": "array"}))));
}

// --- if/then/else ---

#[test]
fn if_then_follows_then_branch() {
    let schema = json!({
        "if": {"properties": {"country": {"const": "US"}}},
        "then": {"properties": {"state": {"type": "string"}}},
        "else": {"properties": {"province": {"type": "string"}}}
    });
    let sv = SchemaValue::new(json!({"country": "US", "state": "CA"}), schema);
    let result = sv.if_then();
    assert!(result.schema().unwrap().get("properties").unwrap().get("state").is_some());
}

#[test]
fn if_then_follows_else_branch() {
    let schema = json!({
        "if": {"properties": {"country": {"const": "US"}}},
        "then": {"properties": {"state": {"type": "string"}}},
        "else": {"properties": {"province": {"type": "string"}}}
    });
    let sv = SchemaValue::new(json!({"country": "CA", "province": "ON"}), schema);
    let result = sv.if_then();
    assert!(result.schema().unwrap().get("properties").unwrap().get("province").is_some());
}

#[test]
fn if_then_no_if_returns_self() {
    let schema = json!({"properties": {"x": {"type": "integer"}}});
    let sv = SchemaValue::new(json!({"x": 1}), schema);
    let result = sv.if_then();
    assert_eq!(result, sv);
}

// --- project ---

#[test]
fn project_keeps_only_schema_properties() {
    let schema = json!({
        "properties": {
            "name": {"type": "string"},
            "age": {"type": "integer"}
        }
    });
    let sv = SchemaValue::new(
        json!({"name": "Alice", "age": 30, "extra": true, "secret": "x"}),
        schema,
    );
    let projected = sv.project().unwrap();
    let data = projected.to_value();
    let obj = data.as_object().unwrap();
    assert_eq!(obj.len(), 2);
    assert!(obj.contains_key("name"));
    assert!(obj.contains_key("age"));
}

#[test]
fn project_auto_resolves_one_of() {
    let schema = json!({
        "oneOf": [
            {"properties": {"type": {"const": "a"}, "x": {"type": "integer"}}},
            {"properties": {"type": {"const": "b"}, "y": {"type": "string"}}}
        ]
    });
    let sv = SchemaValue::new(json!({"type": "a", "x": 1, "extra": true}), schema);
    let projected = sv.project().unwrap();
    let data = projected.to_value();
    let obj = data.as_object().unwrap();
    assert!(obj.contains_key("type"));
    assert!(obj.contains_key("x"));
    assert!(!obj.contains_key("extra"));
}

// --- contains ---

#[test]
fn contains_finds_matching_element() {
    let schema = json!({
        "type": "array",
        "contains": {"type": "string", "const": "admin"}
    });
    let sv = SchemaValue::new(json!(["user", "admin", "guest"]), schema);
    assert!(sv.contains(None));
}

#[test]
fn contains_no_match() {
    let schema = json!({
        "type": "array",
        "contains": {"const": 999}
    });
    let sv = SchemaValue::new(json!([1, 2, 3]), schema);
    assert!(!sv.contains(None));
}

#[test]
fn contains_with_explicit_schema() {
    let sv = SchemaValue::without_schema(json!([1, "hello", true]));
    assert!(sv.contains(Some(&json!({"type": "boolean"}))));
    assert!(!sv.contains(Some(&json!({"type": "object"}))));
}

#[test]
fn contains_on_non_array() {
    let sv = SchemaValue::without_schema(json!(42));
    assert!(!sv.contains(Some(&json!({"const": 42}))));
}

// --- Combined: composition + validation ---

#[test]
fn one_of_branch_then_validate() {
    let schema = json!({
        "oneOf": [
            {
                "properties": {
                    "kind": {"const": "circle"},
                    "radius": {"type": "number", "minimum": 0}
                },
                "required": ["kind", "radius"]
            },
            {
                "properties": {
                    "kind": {"const": "rect"},
                    "width": {"type": "number", "minimum": 0},
                    "height": {"type": "number", "minimum": 0}
                },
                "required": ["kind", "width", "height"]
            }
        ]
    });
    let data = json!({"kind": "circle", "radius": 5.0});
    let sv = SchemaValue::new(data, schema);
    let branch = sv.one_of().unwrap();
    assert!(branch.validate().is_ok());
}

#[test]
fn all_of_merge_then_validate() {
    let schema = json!({
        "allOf": [
            {"properties": {"name": {"type": "string"}}, "required": ["name"]},
            {"properties": {"age": {"type": "integer", "minimum": 0}}, "required": ["age"]}
        ]
    });
    let sv = SchemaValue::new(json!({"name": "Alice", "age": 30}), schema);
    let merged = sv.all_of();
    assert!(merged.validate().is_ok());
}
