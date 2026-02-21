//! Integration tests: Draft-07 validation keywords end-to-end.

use schema_value::{ErrorKind, SchemaValue};
use serde_json::json;

fn assert_valid(schema: serde_json::Value, data: serde_json::Value) {
    let sv = SchemaValue::new(data, schema);
    assert!(sv.validate().is_ok(), "expected valid but got: {:?}", sv.validate());
}

fn assert_invalid(schema: serde_json::Value, data: serde_json::Value, expected_kind: ErrorKind) {
    let sv = SchemaValue::new(data, schema);
    let errs = sv.validate().unwrap_err();
    assert!(
        errs.iter().any(|e| e.kind == expected_kind),
        "expected {:?} but got: {:?}",
        expected_kind,
        errs
    );
}

// --- type ---

#[test]
fn type_string_pass_fail() {
    let schema = json!({"type": "string"});
    assert_valid(schema.clone(), json!("hello"));
    assert_invalid(schema, json!(42), ErrorKind::TypeError);
}

#[test]
fn type_integer_pass_fail() {
    let schema = json!({"type": "integer"});
    assert_valid(schema.clone(), json!(42));
    assert_invalid(schema.clone(), json!(3.14), ErrorKind::TypeError);
    // bool is not integer
    assert_invalid(schema, json!(true), ErrorKind::TypeError);
}

#[test]
fn type_number_pass_fail() {
    let schema = json!({"type": "number"});
    assert_valid(schema.clone(), json!(3.14));
    assert_valid(schema.clone(), json!(42));
    assert_invalid(schema, json!("not"), ErrorKind::TypeError);
}

#[test]
fn type_boolean_pass_fail() {
    let schema = json!({"type": "boolean"});
    assert_valid(schema.clone(), json!(true));
    assert_invalid(schema, json!(1), ErrorKind::TypeError);
}

#[test]
fn type_null_pass_fail() {
    let schema = json!({"type": "null"});
    assert_valid(schema.clone(), json!(null));
    assert_invalid(schema, json!(0), ErrorKind::TypeError);
}

#[test]
fn type_union() {
    let schema = json!({"type": ["string", "null"]});
    assert_valid(schema.clone(), json!("hello"));
    assert_valid(schema.clone(), json!(null));
    assert_invalid(schema, json!(42), ErrorKind::TypeError);
}

// --- const / enum ---

#[test]
fn const_keyword() {
    let schema = json!({"const": "fixed"});
    assert_valid(schema.clone(), json!("fixed"));
    assert_invalid(schema, json!("other"), ErrorKind::ConstMismatch);
}

#[test]
fn enum_keyword() {
    let schema = json!({"enum": ["a", "b", "c"]});
    assert_valid(schema.clone(), json!("b"));
    assert_invalid(schema, json!("d"), ErrorKind::EnumMismatch);
}

// --- Numeric constraints ---

#[test]
fn minimum_maximum() {
    let schema = json!({"type": "number", "minimum": 0, "maximum": 100});
    assert_valid(schema.clone(), json!(0));
    assert_valid(schema.clone(), json!(100));
    assert_valid(schema.clone(), json!(50));
    assert_invalid(schema.clone(), json!(-1), ErrorKind::Minimum);
    assert_invalid(schema, json!(101), ErrorKind::Maximum);
}

#[test]
fn exclusive_min_max() {
    let schema = json!({"type": "number", "exclusiveMinimum": 0, "exclusiveMaximum": 10});
    assert_valid(schema.clone(), json!(1));
    assert_valid(schema.clone(), json!(9));
    assert_invalid(schema.clone(), json!(0), ErrorKind::ExclusiveMinimum);
    assert_invalid(schema, json!(10), ErrorKind::ExclusiveMaximum);
}

#[test]
fn multiple_of() {
    let schema = json!({"type": "number", "multipleOf": 3});
    assert_valid(schema.clone(), json!(9));
    assert_valid(schema.clone(), json!(0));
    assert_invalid(schema, json!(10), ErrorKind::MultipleOf);
}

// --- String constraints ---

#[test]
fn min_max_length() {
    let schema = json!({"type": "string", "minLength": 2, "maxLength": 5});
    assert_valid(schema.clone(), json!("ab"));
    assert_valid(schema.clone(), json!("abcde"));
    assert_invalid(schema.clone(), json!("a"), ErrorKind::MinLength);
    assert_invalid(schema, json!("abcdef"), ErrorKind::MaxLength);
}

#[test]
fn pattern() {
    let schema = json!({"type": "string", "pattern": "^[a-z]+$"});
    assert_valid(schema.clone(), json!("hello"));
    assert_invalid(schema, json!("Hello"), ErrorKind::Pattern);
}

// --- Array constraints ---

#[test]
fn min_max_items() {
    let schema = json!({"type": "array", "minItems": 1, "maxItems": 3});
    assert_valid(schema.clone(), json!([1]));
    assert_valid(schema.clone(), json!([1, 2, 3]));
    assert_invalid(schema.clone(), json!([]), ErrorKind::MinItems);
    assert_invalid(schema, json!([1, 2, 3, 4]), ErrorKind::MaxItems);
}

#[test]
fn unique_items() {
    let schema = json!({"type": "array", "uniqueItems": true});
    assert_valid(schema.clone(), json!([1, 2, 3]));
    assert_invalid(schema, json!([1, 2, 1]), ErrorKind::UniqueItems);
}

// --- Object constraints ---

#[test]
fn required_properties() {
    let schema = json!({
        "type": "object",
        "required": ["name", "age"]
    });
    assert_valid(schema.clone(), json!({"name": "Alice", "age": 30}));
    assert_invalid(schema, json!({"name": "Alice"}), ErrorKind::Required);
}

#[test]
fn additional_properties_false() {
    let schema = json!({
        "type": "object",
        "properties": {"x": {"type": "integer"}},
        "additionalProperties": false
    });
    assert_valid(schema.clone(), json!({"x": 1}));
    assert_invalid(schema, json!({"x": 1, "y": 2}), ErrorKind::AdditionalProperties);
}

#[test]
fn additional_properties_schema() {
    let schema = json!({
        "type": "object",
        "properties": {"x": {"type": "integer"}},
        "additionalProperties": {"type": "string"}
    });
    assert_valid(schema.clone(), json!({"x": 1, "extra": "ok"}));
    // Extra property validated against additionalProperties schema — fails TypeError
    assert_invalid(schema, json!({"x": 1, "extra": 42}), ErrorKind::TypeError);
}

#[test]
fn min_max_properties() {
    let schema = json!({"type": "object", "minProperties": 1, "maxProperties": 2});
    assert_valid(schema.clone(), json!({"a": 1}));
    assert_valid(schema.clone(), json!({"a": 1, "b": 2}));
    assert_invalid(schema.clone(), json!({}), ErrorKind::MinProperties);
    assert_invalid(schema, json!({"a": 1, "b": 2, "c": 3}), ErrorKind::MaxProperties);
}

#[test]
fn pattern_properties() {
    let schema = json!({
        "type": "object",
        "patternProperties": {
            "^S_": {"type": "string"},
            "^I_": {"type": "integer"}
        },
        "additionalProperties": false
    });
    assert_valid(schema.clone(), json!({"S_name": "Alice", "I_age": 30}));
    // S_name matched by patternProperties ^S_, value 42 fails type check
    assert_invalid(schema.clone(), json!({"S_name": 42}), ErrorKind::TypeError);
}

// --- Dependencies ---

#[test]
fn dependent_required() {
    let schema = json!({
        "type": "object",
        "properties": {
            "credit_card": {"type": "string"},
            "billing_address": {"type": "string"}
        },
        "dependencies": {
            "credit_card": ["billing_address"]
        }
    });
    assert_valid(schema.clone(), json!({"credit_card": "1234", "billing_address": "123 St"}));
    assert_valid(schema.clone(), json!({"billing_address": "123 St"}));
    assert_invalid(schema, json!({"credit_card": "1234"}), ErrorKind::DependentRequired);
}

// --- Nested validation ---

#[test]
fn nested_object_validation() {
    let schema = json!({
        "type": "object",
        "properties": {
            "address": {
                "type": "object",
                "properties": {
                    "zip": {"type": "string", "pattern": "^\\d{5}$"}
                },
                "required": ["zip"]
            }
        }
    });
    assert_valid(schema.clone(), json!({"address": {"zip": "12345"}}));
    assert_invalid(schema, json!({"address": {"zip": "abc"}}), ErrorKind::Pattern);
}

#[test]
fn nested_array_item_validation() {
    let schema = json!({
        "type": "object",
        "properties": {
            "tags": {
                "type": "array",
                "items": {"type": "string", "minLength": 1},
                "minItems": 1
            }
        }
    });
    assert_valid(schema.clone(), json!({"tags": ["rust", "json"]}));
    // Empty string fails minLength on items validation
    assert_invalid(schema, json!({"tags": [""]}), ErrorKind::MinLength);
}
