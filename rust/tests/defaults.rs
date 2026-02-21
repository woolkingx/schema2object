//! Integration tests: default value filling.

use schema_value::SchemaValue;
use serde_json::json;

#[test]
fn simple_defaults() {
    let schema = json!({
        "type": "object",
        "properties": {
            "status": {"type": "string", "default": "pending"},
            "priority": {"type": "integer", "default": 0}
        }
    });
    let sv = SchemaValue::new(json!({}), schema).with_defaults();
    assert_eq!(sv.get("status").unwrap().to_value(), json!("pending"));
    assert_eq!(sv.get("priority").unwrap().to_value(), json!(0));
}

#[test]
fn existing_values_not_overwritten() {
    let schema = json!({
        "type": "object",
        "properties": {
            "status": {"type": "string", "default": "pending"}
        }
    });
    let sv = SchemaValue::new(json!({"status": "active"}), schema).with_defaults();
    assert_eq!(sv.get("status").unwrap().to_value(), json!("active"));
}

#[test]
fn nested_defaults() {
    let schema = json!({
        "type": "object",
        "properties": {
            "config": {
                "type": "object",
                "properties": {
                    "retries": {"type": "integer", "default": 3},
                    "timeout": {"type": "integer", "default": 5000}
                }
            }
        }
    });
    let sv = SchemaValue::new(json!({"config": {}}), schema).with_defaults();
    assert_eq!(sv.path("config.retries").unwrap().to_value(), json!(3));
    assert_eq!(sv.path("config.timeout").unwrap().to_value(), json!(5000));
}

#[test]
fn nested_partial_existing() {
    let schema = json!({
        "type": "object",
        "properties": {
            "config": {
                "type": "object",
                "properties": {
                    "retries": {"type": "integer", "default": 3},
                    "timeout": {"type": "integer", "default": 5000}
                }
            }
        }
    });
    let sv = SchemaValue::new(json!({"config": {"retries": 10}}), schema).with_defaults();
    assert_eq!(sv.path("config.retries").unwrap().to_value(), json!(10)); // kept
    assert_eq!(sv.path("config.timeout").unwrap().to_value(), json!(5000)); // filled
}

#[test]
fn no_default_no_fill() {
    let schema = json!({
        "type": "object",
        "properties": {
            "name": {"type": "string"},
            "age": {"type": "integer", "default": 0}
        }
    });
    let sv = SchemaValue::new(json!({}), schema).with_defaults();
    assert_eq!(sv.get("age").unwrap().to_value(), json!(0));
    // "name" has no default, stays absent
    assert!(sv.get("name").is_none());
}

#[test]
fn without_schema_no_op() {
    let sv = SchemaValue::without_schema(json!({})).with_defaults();
    assert!(sv.is_empty());
}

#[test]
fn non_object_data_unchanged() {
    let schema = json!({"properties": {"x": {"default": 1}}});
    let sv = SchemaValue::new(json!(42), schema).with_defaults();
    assert_eq!(sv, json!(42));
}

#[test]
fn mixed_defaults_and_existing() {
    let schema = json!({
        "type": "object",
        "properties": {
            "a": {"type": "integer", "default": 10},
            "b": {"type": "string", "default": "hello"},
            "c": {"type": "boolean"}
        }
    });
    let sv = SchemaValue::new(json!({"b": "world"}), schema).with_defaults();
    assert_eq!(sv.get("a").unwrap().to_value(), json!(10));       // filled
    assert_eq!(sv.get("b").unwrap().to_value(), json!("world"));  // kept
    assert!(sv.get("c").is_none());       // no default in schema
}

// --- Defaults + validation combined ---

#[test]
fn defaults_then_validate() {
    let schema = json!({
        "type": "object",
        "properties": {
            "mode": {"type": "string", "default": "auto", "enum": ["auto", "manual"]},
            "retries": {"type": "integer", "default": 3, "minimum": 0}
        },
        "required": ["mode", "retries"]
    });
    // Empty data would fail required, but after defaults it should pass
    let sv = SchemaValue::new(json!({}), schema).with_defaults();
    assert!(sv.validate().is_ok());
    assert_eq!(sv.get("mode").unwrap().to_value(), json!("auto"));
    assert_eq!(sv.get("retries").unwrap().to_value(), json!(3));
}
