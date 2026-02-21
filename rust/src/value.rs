use std::fmt;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::validate::cached_regex;

/// A JSON value wrapped with an optional JSON Schema for runtime validation.
///
/// Schema propagates to children automatically: accessing a property
/// returns a `SchemaValue` carrying the sub-schema for that property.
#[derive(Clone, Debug)]
pub struct SchemaValue {
    pub(crate) data: Value,
    pub(crate) schema: Option<Arc<Value>>,
}

/// Static null for Index trait returns on miss.
static NULL: Value = Value::Null;

impl SchemaValue {
    /// Create with data and schema.
    pub fn new(data: Value, schema: Value) -> Self {
        Self {
            data,
            schema: Some(Arc::new(schema)),
        }
    }

    /// Create without schema (no validation).
    pub fn without_schema(data: Value) -> Self {
        Self {
            data,
            schema: None,
        }
    }

    /// Borrow the inner `serde_json::Value`.
    pub fn as_value(&self) -> &Value {
        &self.data
    }

    /// Borrow the schema if present.
    pub fn schema(&self) -> Option<&Value> {
        self.schema.as_deref()
    }

    /// Consume and return the inner `Value`, discarding schema.
    pub fn into_inner(self) -> Value {
        self.data
    }

    /// True if the inner value is null.
    pub fn is_null(&self) -> bool {
        self.data.is_null()
    }

    /// True if the inner value is a boolean.
    pub fn is_boolean(&self) -> bool {
        self.data.is_boolean()
    }

    /// True if the inner value is a number.
    pub fn is_number(&self) -> bool {
        self.data.is_number()
    }

    /// True if the inner value is a string.
    pub fn is_string(&self) -> bool {
        self.data.is_string()
    }

    /// True if the inner value is an array.
    pub fn is_array(&self) -> bool {
        self.data.is_array()
    }

    /// True if the inner value is an object.
    pub fn is_object(&self) -> bool {
        self.data.is_object()
    }

    /// Extract as bool.
    pub fn as_bool(&self) -> Option<bool> {
        self.data.as_bool()
    }

    /// Extract as i64.
    pub fn as_i64(&self) -> Option<i64> {
        self.data.as_i64()
    }

    /// Extract as u64.
    pub fn as_u64(&self) -> Option<u64> {
        self.data.as_u64()
    }

    /// Extract as f64.
    pub fn as_f64(&self) -> Option<f64> {
        self.data.as_f64()
    }

    /// Extract as string slice.
    pub fn as_str(&self) -> Option<&str> {
        self.data.as_str()
    }

    // --- Mutation with validation ---

    /// Set a value by key. Validates against schema before writing.
    ///
    /// Returns `Err(ValidationError)` if validation fails (data unchanged).
    /// On non-object data, returns `Err` with `NotAnObject` kind.
    pub fn set(&mut self, key: &str, value: Value) -> Result<(), crate::ValidationError> {
        if !self.data.is_object() {
            return Err(crate::ValidationError::root(
                crate::ErrorKind::NotAnObject,
                "cannot set key on non-object value",
            ));
        }

        // Validate against the resolved sub-schema for this key
        // (checks properties → patternProperties → additionalProperties)
        if let Some(sub) = sub_schema_for_key_from(self.schema.as_deref(), key) {
            crate::validate::check_field(key, &sub, &value)?;
        }

        // Borrow is now clear: validation done, safe to mutate
        self.data.as_object_mut().unwrap().insert(key.to_string(), value);
        Ok(())
    }

    /// Apply default values from schema, returning a new SchemaValue.
    ///
    /// For each property in `schema.properties` that has a `default` keyword
    /// and is missing from the data, the default is filled in.
    pub fn with_defaults(mut self) -> Self {
        if let Some(schema) = &self.schema {
            let schema_clone = schema.as_ref().clone();
            crate::defaults::apply_defaults(&mut self.data, &schema_clone);
        }
        self
    }

    /// Validate the entire data against the attached schema.
    ///
    /// Returns on the first validation error found. The error is wrapped
    /// in a `Vec` for forward compatibility with future multi-error collection.
    pub fn validate(&self) -> Result<(), Vec<crate::ValidationError>> {
        let schema = match &self.schema {
            Some(s) => s,
            None => return Ok(()),
        };
        crate::validate::check_value(schema, &self.data).map_err(|e| vec![e])
    }

    // --- Schema-propagating access ---

    /// Get a child by object key, propagating the sub-schema.
    ///
    /// Returns `None` if the data is not an object or the key doesn't exist.
    pub fn get(&self, key: &str) -> Option<SchemaValue> {
        let obj = self.data.as_object()?;
        let child_data = obj.get(key)?.clone();
        let child_schema = self.sub_schema_for_key(key);
        Some(SchemaValue {
            data: child_data,
            schema: child_schema,
        })
    }

    /// Get a child by array index, propagating the items schema.
    ///
    /// Returns `None` if the data is not an array or the index is out of bounds.
    pub fn get_index(&self, idx: usize) -> Option<SchemaValue> {
        let arr = self.data.as_array()?;
        let child_data = arr.get(idx)?.clone();
        let child_schema = self.sub_schema_for_index(idx);
        Some(SchemaValue {
            data: child_data,
            schema: child_schema,
        })
    }

    /// Navigate a dot-separated path (e.g., `"user.profile.name"`).
    ///
    /// Array indices can be used: `"users.0.name"`.
    pub fn path(&self, dot_path: &str) -> Option<SchemaValue> {
        let mut current = self.clone();
        for segment in dot_path.split('.') {
            if let Ok(idx) = segment.parse::<usize>() {
                current = current.get_index(idx)?;
            } else {
                current = current.get(segment)?;
            }
        }
        Some(current)
    }

    /// Number of entries (object) or elements (array). Returns 0 for scalars.
    pub fn len(&self) -> usize {
        match &self.data {
            Value::Object(map) => map.len(),
            Value::Array(arr) => arr.len(),
            _ => 0,
        }
    }

    /// True if null, empty object, or empty array.
    pub fn is_empty(&self) -> bool {
        match &self.data {
            Value::Null => true,
            Value::Object(map) => map.is_empty(),
            Value::Array(arr) => arr.is_empty(),
            _ => false,
        }
    }

    /// Iterate over object entries as `(key, SchemaValue)` pairs.
    ///
    /// Returns an empty iterator for non-objects.
    pub fn entries(&self) -> impl Iterator<Item = (&str, SchemaValue)> {
        let map = self.data.as_object();
        let schema = &self.schema;
        map.into_iter()
            .flat_map(|m| m.iter())
            .map(move |(k, v)| {
                let child_schema = sub_schema_for_key_from(schema.as_deref(), k);
                (
                    k.as_str(),
                    SchemaValue {
                        data: v.clone(),
                        schema: child_schema,
                    },
                )
            })
    }

    /// Iterate over array elements as `SchemaValue`.
    ///
    /// Returns an empty iterator for non-arrays.
    pub fn elements(&self) -> impl Iterator<Item = SchemaValue> + '_ {
        let arr = self.data.as_array();
        arr.into_iter().flat_map(|a| a.iter()).enumerate().map(
            move |(idx, v)| {
                let child_schema = self.sub_schema_for_index(idx);
                SchemaValue {
                    data: v.clone(),
                    schema: child_schema,
                }
            },
        )
    }

    /// Extract the sub-schema for an object key.
    fn sub_schema_for_key(&self, key: &str) -> Option<Arc<Value>> {
        sub_schema_for_key_from(self.schema.as_deref(), key)
    }

    /// Extract the sub-schema for an array index.
    fn sub_schema_for_index(&self, idx: usize) -> Option<Arc<Value>> {
        let schema = self.schema.as_deref()?;
        let items = schema.get("items")?;
        if let Some(arr) = items.as_array() {
            // Tuple validation: items[idx]
            if let Some(item_schema) = arr.get(idx) {
                return Some(Arc::new(item_schema.clone()));
            }
            // Fall through to additionalItems
            if let Some(additional) = schema.get("additionalItems") {
                if additional.is_object() {
                    return Some(Arc::new(additional.clone()));
                }
            }
            return None;
        }
        // Single schema for all items
        if items.is_object() {
            Some(Arc::new(items.clone()))
        } else {
            None
        }
    }
}

/// Free function for sub-schema extraction (used by iterators that can't borrow self).
fn sub_schema_for_key_from(schema: Option<&Value>, key: &str) -> Option<Arc<Value>> {
    let schema = schema?;
    // 1. properties[key]
    if let Some(prop_schema) = schema
        .get("properties")
        .and_then(|p| p.as_object())
        .and_then(|p| p.get(key))
    {
        return Some(Arc::new(prop_schema.clone()));
    }
    // 2. patternProperties
    if let Some(patterns) = schema.get("patternProperties").and_then(|p| p.as_object()) {
        for (pattern, pat_schema) in patterns {
            if let Ok(re) = cached_regex(pattern) {
                if re.is_match(key) {
                    return Some(Arc::new(pat_schema.clone()));
                }
            }
        }
    }
    // 3. additionalProperties (if schema object)
    if let Some(additional) = schema.get("additionalProperties") {
        if additional.is_object() {
            return Some(Arc::new(additional.clone()));
        }
    }
    None
}

// --- From / Into ---

impl From<Value> for SchemaValue {
    fn from(value: Value) -> Self {
        Self::without_schema(value)
    }
}

impl From<SchemaValue> for Value {
    fn from(sv: SchemaValue) -> Self {
        sv.data
    }
}

// --- Display ---

impl fmt::Display for SchemaValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.data)
    }
}

// --- PartialEq (compare data only, ignore schema) ---

impl PartialEq for SchemaValue {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl PartialEq<Value> for SchemaValue {
    fn eq(&self, other: &Value) -> bool {
        &self.data == other
    }
}

// --- Index traits ---

impl std::ops::Index<&str> for SchemaValue {
    type Output = Value;

    /// Quick access by key. Returns `&Value::Null` on miss (never panics).
    fn index(&self, key: &str) -> &Value {
        match &self.data {
            Value::Object(map) => map.get(key).unwrap_or(&NULL),
            _ => &NULL,
        }
    }
}

impl std::ops::Index<usize> for SchemaValue {
    type Output = Value;

    /// Quick access by array index. Returns `&Value::Null` on out-of-bounds.
    fn index(&self, idx: usize) -> &Value {
        match &self.data {
            Value::Array(arr) => arr.get(idx).unwrap_or(&NULL),
            _ => &NULL,
        }
    }
}

// --- Serialize / Deserialize (delegates to data) ---

impl Serialize for SchemaValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.data.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for SchemaValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let data = Value::deserialize(deserializer)?;
        Ok(Self::without_schema(data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_new_and_accessors() {
        let schema = json!({"type": "object"});
        let sv = SchemaValue::new(json!({"name": "Alice"}), schema.clone());
        assert!(sv.is_object());
        assert!(!sv.is_null());
        assert!(sv.schema().is_some());
    }

    #[test]
    fn test_without_schema() {
        let sv = SchemaValue::without_schema(json!(42));
        assert!(sv.schema().is_none());
        assert_eq!(sv.as_i64(), Some(42));
    }

    #[test]
    fn test_from_value() {
        let sv: SchemaValue = json!({"x": 1}).into();
        assert!(sv.is_object());
        assert!(sv.schema().is_none());
    }

    #[test]
    fn test_into_value() {
        let sv = SchemaValue::without_schema(json!("hello"));
        let v: Value = sv.into();
        assert_eq!(v, json!("hello"));
    }

    #[test]
    fn test_index_str() {
        let sv = SchemaValue::without_schema(json!({"a": 1, "b": "two"}));
        assert_eq!(sv["a"], json!(1));
        assert_eq!(sv["b"], json!("two"));
        assert_eq!(sv["missing"], Value::Null);
    }

    #[test]
    fn test_index_usize() {
        let sv = SchemaValue::without_schema(json!([10, 20, 30]));
        assert_eq!(sv[0], json!(10));
        assert_eq!(sv[2], json!(30));
        assert_eq!(sv[99], Value::Null);
    }

    #[test]
    fn test_index_on_wrong_type() {
        let sv = SchemaValue::without_schema(json!(42));
        assert_eq!(sv["key"], Value::Null);
        assert_eq!(sv[0], Value::Null);
    }

    #[test]
    fn test_partial_eq() {
        let a = SchemaValue::new(json!({"x": 1}), json!({"type": "object"}));
        let b = SchemaValue::without_schema(json!({"x": 1}));
        assert_eq!(a, b); // schema ignored in comparison
    }

    #[test]
    fn test_partial_eq_with_value() {
        let sv = SchemaValue::without_schema(json!(42));
        assert_eq!(sv, json!(42));
    }

    #[test]
    fn test_display() {
        let sv = SchemaValue::without_schema(json!({"name": "Alice"}));
        let s = sv.to_string();
        assert!(s.contains("Alice"));
    }

    #[test]
    fn test_clone() {
        let sv = SchemaValue::new(json!({"x": 1}), json!({"type": "object"}));
        let sv2 = sv.clone();
        assert_eq!(sv, sv2);
        assert!(sv2.schema().is_some());
    }

    #[test]
    fn test_serialize() {
        let sv = SchemaValue::without_schema(json!({"a": 1}));
        let s = serde_json::to_string(&sv).unwrap();
        assert_eq!(s, r#"{"a":1}"#);
    }

    #[test]
    fn test_deserialize() {
        let sv: SchemaValue = serde_json::from_str(r#"{"a":1}"#).unwrap();
        assert_eq!(sv["a"], json!(1));
        assert!(sv.schema().is_none());
    }

    #[test]
    fn test_type_checks() {
        assert!(SchemaValue::without_schema(Value::Null).is_null());
        assert!(SchemaValue::without_schema(json!(true)).is_boolean());
        assert!(SchemaValue::without_schema(json!(3.14)).is_number());
        assert!(SchemaValue::without_schema(json!("hi")).is_string());
        assert!(SchemaValue::without_schema(json!([1])).is_array());
        assert!(SchemaValue::without_schema(json!({})).is_object());
    }

    // --- Phase 3: set() and validate() tests ---

    #[test]
    fn test_set_pass() {
        let schema = json!({
            "type": "object",
            "properties": {
                "age": {"type": "integer", "minimum": 0}
            }
        });
        let mut sv = SchemaValue::new(json!({}), schema);
        assert!(sv.set("age", json!(25)).is_ok());
        assert_eq!(sv["age"], json!(25));
    }

    #[test]
    fn test_set_type_fail() {
        let schema = json!({
            "type": "object",
            "properties": {
                "age": {"type": "integer"}
            }
        });
        let mut sv = SchemaValue::new(json!({}), schema);
        let err = sv.set("age", json!("not_int"));
        assert!(err.is_err());
        // Data unchanged on failure
        assert_eq!(sv["age"], Value::Null);
    }

    #[test]
    fn test_set_constraint_fail() {
        let schema = json!({
            "type": "object",
            "properties": {
                "age": {"type": "integer", "minimum": 0}
            }
        });
        let mut sv = SchemaValue::new(json!({}), schema);
        assert!(sv.set("age", json!(-1)).is_err());
    }

    #[test]
    fn test_set_no_schema() {
        let mut sv = SchemaValue::without_schema(json!({}));
        assert!(sv.set("anything", json!("ok")).is_ok());
        assert_eq!(sv["anything"], json!("ok"));
    }

    #[test]
    fn test_set_on_non_object() {
        let mut sv = SchemaValue::without_schema(json!(42));
        assert!(sv.set("key", json!(1)).is_err());
    }

    #[test]
    fn test_validate_pass() {
        let schema = json!({
            "type": "object",
            "properties": {
                "name": {"type": "string"}
            },
            "required": ["name"]
        });
        let sv = SchemaValue::new(json!({"name": "Alice"}), schema);
        assert!(sv.validate().is_ok());
    }

    #[test]
    fn test_validate_fail() {
        let schema = json!({
            "type": "object",
            "required": ["name"]
        });
        let sv = SchemaValue::new(json!({}), schema);
        assert!(sv.validate().is_err());
    }

    #[test]
    fn test_validate_no_schema() {
        let sv = SchemaValue::without_schema(json!("anything"));
        assert!(sv.validate().is_ok());
    }

    // --- Phase 2: Schema propagation tests ---

    #[test]
    fn test_get_object_key() {
        let sv = SchemaValue::without_schema(json!({"user": {"name": "Alice"}}));
        let user = sv.get("user").unwrap();
        assert!(user.is_object());
        assert_eq!(user["name"], json!("Alice"));
    }

    #[test]
    fn test_get_missing_key() {
        let sv = SchemaValue::without_schema(json!({"a": 1}));
        assert!(sv.get("missing").is_none());
    }

    #[test]
    fn test_get_on_non_object() {
        let sv = SchemaValue::without_schema(json!(42));
        assert!(sv.get("key").is_none());
    }

    #[test]
    fn test_get_propagates_schema() {
        let schema = json!({
            "type": "object",
            "properties": {
                "user": {
                    "type": "object",
                    "properties": {
                        "name": {"type": "string"}
                    }
                }
            }
        });
        let sv = SchemaValue::new(json!({"user": {"name": "Alice"}}), schema);
        let user = sv.get("user").unwrap();
        assert!(user.schema().is_some());
        let user_schema = user.schema().unwrap();
        assert_eq!(user_schema["type"], json!("object"));

        // Sub-schema propagates further
        let name = user.get("name").unwrap();
        assert!(name.schema().is_some());
        assert_eq!(name.schema().unwrap()["type"], json!("string"));
    }

    #[test]
    fn test_get_index() {
        let sv = SchemaValue::without_schema(json!([10, 20, 30]));
        let second = sv.get_index(1).unwrap();
        assert_eq!(second, json!(20));
        assert!(sv.get_index(99).is_none());
    }

    #[test]
    fn test_get_index_propagates_items_schema() {
        let schema = json!({
            "type": "array",
            "items": {"type": "integer", "minimum": 0}
        });
        let sv = SchemaValue::new(json!([1, 2, 3]), schema);
        let elem = sv.get_index(0).unwrap();
        assert!(elem.schema().is_some());
        assert_eq!(elem.schema().unwrap()["type"], json!("integer"));
    }

    #[test]
    fn test_path_simple() {
        let sv = SchemaValue::without_schema(json!({
            "user": {"profile": {"name": "Alice"}}
        }));
        let name = sv.path("user.profile.name").unwrap();
        assert_eq!(name, json!("Alice"));
    }

    #[test]
    fn test_path_with_array_index() {
        let sv = SchemaValue::without_schema(json!({
            "users": [{"name": "Alice"}, {"name": "Bob"}]
        }));
        let bob = sv.path("users.1.name").unwrap();
        assert_eq!(bob, json!("Bob"));
    }

    #[test]
    fn test_path_missing() {
        let sv = SchemaValue::without_schema(json!({"a": 1}));
        assert!(sv.path("a.b.c").is_none());
    }

    #[test]
    fn test_len_and_is_empty() {
        let obj = SchemaValue::without_schema(json!({"a": 1, "b": 2}));
        assert_eq!(obj.len(), 2);
        assert!(!obj.is_empty());

        let arr = SchemaValue::without_schema(json!([1, 2, 3]));
        assert_eq!(arr.len(), 3);

        let empty = SchemaValue::without_schema(json!({}));
        assert!(empty.is_empty());
        assert_eq!(empty.len(), 0);

        let null = SchemaValue::without_schema(Value::Null);
        assert!(null.is_empty());

        let scalar = SchemaValue::without_schema(json!(42));
        assert_eq!(scalar.len(), 0);
        assert!(!scalar.is_empty()); // scalars are not "empty"
    }

    #[test]
    fn test_entries() {
        let sv = SchemaValue::without_schema(json!({"a": 1, "b": 2}));
        let entries: Vec<_> = sv.entries().collect();
        assert_eq!(entries.len(), 2);
        // entries come from BTreeMap-like order in serde_json
        let keys: Vec<&str> = entries.iter().map(|(k, _)| *k).collect();
        assert!(keys.contains(&"a"));
        assert!(keys.contains(&"b"));
    }

    #[test]
    fn test_entries_propagate_schema() {
        let schema = json!({
            "type": "object",
            "properties": {
                "x": {"type": "integer"},
                "y": {"type": "string"}
            }
        });
        let sv = SchemaValue::new(json!({"x": 1, "y": "hi"}), schema);
        for (key, child) in sv.entries() {
            assert!(child.schema().is_some(), "child '{key}' should have schema");
        }
    }

    #[test]
    fn test_elements() {
        let sv = SchemaValue::without_schema(json!([10, 20, 30]));
        let elems: Vec<_> = sv.elements().collect();
        assert_eq!(elems.len(), 3);
        assert_eq!(elems[0], json!(10));
        assert_eq!(elems[2], json!(30));
    }

    #[test]
    fn test_elements_propagate_schema() {
        let schema = json!({
            "type": "array",
            "items": {"type": "integer"}
        });
        let sv = SchemaValue::new(json!([1, 2]), schema);
        for elem in sv.elements() {
            assert!(elem.schema().is_some());
        }
    }

    #[test]
    fn test_entries_on_non_object() {
        let sv = SchemaValue::without_schema(json!(42));
        assert_eq!(sv.entries().count(), 0);
    }

    #[test]
    fn test_elements_on_non_array() {
        let sv = SchemaValue::without_schema(json!({"a": 1}));
        assert_eq!(sv.elements().count(), 0);
    }
}
