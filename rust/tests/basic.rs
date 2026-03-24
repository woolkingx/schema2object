//! Integration tests: construction, access, Index, serialization, schema propagation, iterators.

use schema2object::ObjectTree;
use serde_json::{json, Value};

// --- Construction ---

#[test]
fn construct_with_schema() {
    let schema = json!({"type": "object", "properties": {"name": {"type": "string"}}});
    let sv = ObjectTree::new(json!({"name": "Alice"}), schema);
    assert!(sv.schema().is_some());
    assert!(sv.is_object());
}

#[test]
fn construct_without_schema() {
    let sv = ObjectTree::without_schema(json!([1, 2, 3]));
    assert!(sv.schema().is_none());
    assert!(sv.is_array());
}

#[test]
fn construct_from_value() {
    let sv: ObjectTree = json!(42).into();
    assert!(sv.is_number());
    assert!(sv.schema().is_none());
}

#[test]
fn into_inner_round_trip() {
    let original = json!({"x": [1, 2]});
    let sv = ObjectTree::without_schema(original.clone());
    let recovered: Value = sv.into();
    assert_eq!(recovered, original);
}

// --- Type checks ---

#[test]
fn type_checks_all_variants() {
    assert!(ObjectTree::without_schema(Value::Null).is_null());
    assert!(ObjectTree::without_schema(json!(true)).is_boolean());
    assert!(ObjectTree::without_schema(json!(42)).is_number());
    assert!(ObjectTree::without_schema(json!("hi")).is_string());
    assert!(ObjectTree::without_schema(json!([1])).is_array());
    assert!(ObjectTree::without_schema(json!({})).is_object());
}

#[test]
fn extractors() {
    assert_eq!(ObjectTree::without_schema(json!(true)).as_bool(), Some(true));
    assert_eq!(ObjectTree::without_schema(json!(42)).as_i64(), Some(42));
    assert_eq!(ObjectTree::without_schema(json!(42)).as_u64(), Some(42));
    assert_eq!(ObjectTree::without_schema(json!(3.14)).as_f64(), Some(3.14));
    assert_eq!(ObjectTree::without_schema(json!("hi")).as_str(), Some("hi"));
}

// --- Accessors ---

#[test]
fn get_hit_and_miss() {
    let sv = ObjectTree::without_schema(json!({"a": 1, "b": "two"}));
    assert_eq!(sv.get("a").unwrap().to_value(), json!(1));
    assert_eq!(sv.get("b").unwrap().to_value(), json!("two"));
    assert!(sv.get("missing").is_none());
}

#[test]
fn get_index_hit_and_miss() {
    let sv = ObjectTree::without_schema(json!([10, 20, 30]));
    assert_eq!(sv.get_index(0).unwrap().to_value(), json!(10));
    assert_eq!(sv.get_index(2).unwrap().to_value(), json!(30));
    assert!(sv.get_index(999).is_none());
}

// --- PartialEq ---

#[test]
fn eq_ignores_schema() {
    let a = ObjectTree::new(json!({"x": 1}), json!({"type": "object"}));
    let b = ObjectTree::without_schema(json!({"x": 1}));
    assert_eq!(a, b);
}

#[test]
fn eq_with_raw_value() {
    let sv = ObjectTree::without_schema(json!("hello"));
    assert_eq!(sv, json!("hello"));
}

// --- Display / Debug ---

#[test]
fn display_shows_json() {
    let sv = ObjectTree::without_schema(json!({"k": 42}));
    let s = sv.to_string();
    assert!(s.contains("42"));
}

// --- Serialize / Deserialize ---

#[test]
fn serialize_round_trip() {
    let data = json!({"name": "Alice", "scores": [1, 2, 3]});
    let sv = ObjectTree::without_schema(data.clone());
    let json_str = serde_json::to_string(&sv).unwrap();
    let recovered: ObjectTree = serde_json::from_str(&json_str).unwrap();
    assert_eq!(recovered, data);
}

// --- Schema propagation via get() ---

#[test]
fn get_propagates_sub_schema() {
    let schema = json!({
        "type": "object",
        "properties": {
            "user": {
                "type": "object",
                "properties": {
                    "name": {"type": "string", "maxLength": 50}
                }
            }
        }
    });
    let sv = ObjectTree::new(json!({"user": {"name": "Alice"}}), schema);
    let user = sv.get("user").unwrap();
    assert_eq!(user.schema().unwrap()["type"], json!("object"));

    let name = user.get("name").unwrap();
    assert_eq!(name.schema().unwrap()["type"], json!("string"));
    assert_eq!(name.schema().unwrap()["maxLength"], json!(50));
}

#[test]
fn get_missing_returns_none() {
    let sv = ObjectTree::without_schema(json!({"a": 1}));
    assert!(sv.get("missing").is_none());
}

#[test]
fn get_on_non_object_returns_none() {
    let sv = ObjectTree::without_schema(json!(42));
    assert!(sv.get("key").is_none());
}

// --- get_index ---

#[test]
fn get_index_propagates_items_schema() {
    let schema = json!({
        "type": "array",
        "items": {"type": "integer", "minimum": 0}
    });
    let sv = ObjectTree::new(json!([1, 2, 3]), schema);
    let elem = sv.get_index(1).unwrap();
    assert_eq!(elem.to_value(), json!(2));
    assert_eq!(elem.schema().unwrap()["type"], json!("integer"));
}

#[test]
fn get_index_oob_returns_none() {
    let sv = ObjectTree::without_schema(json!([1]));
    assert!(sv.get_index(99).is_none());
}

// --- path() ---

#[test]
fn path_deep_traversal() {
    let sv = ObjectTree::without_schema(json!({
        "a": {"b": {"c": 42}}
    }));
    assert_eq!(sv.path("a.b.c").unwrap().to_value(), json!(42));
}

#[test]
fn path_with_array_index() {
    let sv = ObjectTree::without_schema(json!({
        "users": [{"name": "Alice"}, {"name": "Bob"}]
    }));
    assert_eq!(sv.path("users.1.name").unwrap().to_value(), json!("Bob"));
}

#[test]
fn path_missing_returns_none() {
    let sv = ObjectTree::without_schema(json!({"a": 1}));
    assert!(sv.path("a.b.c").is_none());
}

// --- len / is_empty ---

#[test]
fn len_and_empty() {
    let obj = ObjectTree::without_schema(json!({"a": 1, "b": 2}));
    assert_eq!(obj.len(), 2);
    assert!(!obj.is_empty());

    let arr = ObjectTree::without_schema(json!([1, 2, 3]));
    assert_eq!(arr.len(), 3);

    let empty_obj = ObjectTree::without_schema(json!({}));
    assert!(empty_obj.is_empty());

    let null = ObjectTree::without_schema(Value::Null);
    assert!(null.is_empty());
}

// --- entries / elements ---

#[test]
fn entries_iterates_object() {
    let sv = ObjectTree::without_schema(json!({"x": 1, "y": 2}));
    let entries: Vec<_> = sv.entries().collect();
    assert_eq!(entries.len(), 2);
}

#[test]
fn entries_propagate_schema() {
    let schema = json!({
        "type": "object",
        "properties": {
            "x": {"type": "integer"},
            "y": {"type": "string"}
        }
    });
    let sv = ObjectTree::new(json!({"x": 1, "y": "hi"}), schema);
    for (_, child) in sv.entries() {
        assert!(child.schema().is_some());
    }
}

#[test]
fn elements_iterates_array() {
    let sv = ObjectTree::without_schema(json!([10, 20, 30]));
    let elems: Vec<_> = sv.elements().collect();
    assert_eq!(elems.len(), 3);
    assert_eq!(elems[1].to_value(), json!(20));
}

#[test]
fn elements_propagate_items_schema() {
    let schema = json!({"type": "array", "items": {"type": "integer"}});
    let sv = ObjectTree::new(json!([1, 2]), schema);
    for elem in sv.elements() {
        assert!(elem.schema().is_some());
    }
}

#[test]
fn entries_on_non_object_empty() {
    let sv = ObjectTree::without_schema(json!(42));
    assert_eq!(sv.entries().count(), 0);
}

#[test]
fn elements_on_non_array_empty() {
    let sv = ObjectTree::without_schema(json!({"a": 1}));
    assert_eq!(sv.elements().count(), 0);
}

// --- set() ---

#[test]
fn set_validates_then_writes() {
    let schema = json!({
        "type": "object",
        "properties": {
            "age": {"type": "integer", "minimum": 0}
        }
    });
    let mut sv = ObjectTree::new(json!({}), schema);
    assert!(sv.set("age", json!(25)).is_ok());
    assert_eq!(sv.get("age").unwrap().to_value(), json!(25));

    // Constraint violation
    assert!(sv.set("age", json!(-1)).is_err());
    assert_eq!(sv.get("age").unwrap().to_value(), json!(25)); // unchanged

    // Type violation
    assert!(sv.set("age", json!("old")).is_err());
    assert_eq!(sv.get("age").unwrap().to_value(), json!(25)); // still unchanged
}

#[test]
fn set_on_non_object_fails() {
    let mut sv = ObjectTree::without_schema(json!(42));
    assert!(sv.set("key", json!(1)).is_err());
}

// --- validate() ---

#[test]
fn validate_full_object() {
    let schema = json!({
        "type": "object",
        "properties": {
            "name": {"type": "string"},
            "age": {"type": "integer", "minimum": 0}
        },
        "required": ["name", "age"]
    });
    let good = ObjectTree::new(json!({"name": "Alice", "age": 30}), schema.clone());
    assert!(good.validate().is_ok());

    let bad = ObjectTree::new(json!({"name": "Alice"}), schema);
    assert!(bad.validate().is_err());
}

#[test]
fn validate_without_schema_always_ok() {
    let sv = ObjectTree::without_schema(json!("anything"));
    assert!(sv.validate().is_ok());
}
