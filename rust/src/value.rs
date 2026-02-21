use std::collections::BTreeMap;
use std::fmt;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::validate::cached_regex;

#[derive(Clone, Debug)]
enum Node {
    Object(BTreeMap<String, SchemaValue>),
    Array(Vec<SchemaValue>),
    Scalar(Value),
}

/// Schema-defined object value. Data + schema are one identity.
#[derive(Clone, Debug)]
pub struct SchemaValue {
    node: Node,
    schema: Option<Arc<Value>>,
}

impl SchemaValue {
    /// Create with data and schema.
    pub fn new(data: Value, schema: Value) -> Self {
        let schema = Arc::new(schema);
        Self::from_value(data, Some(schema))
    }

    /// Create without schema (no validation).
    pub fn without_schema(data: Value) -> Self {
        Self::from_value(data, None)
    }

    fn from_value(data: Value, schema: Option<Arc<Value>>) -> Self {
        let node = match data {
            Value::Object(map) => {
                let mut out = BTreeMap::new();
                for (k, v) in map {
                    let sub_schema = sub_schema_for_key_from(schema.as_deref(), &k);
                    out.insert(k, SchemaValue::from_value(v, sub_schema));
                }
                Node::Object(out)
            }
            Value::Array(arr) => {
                let mut out = Vec::with_capacity(arr.len());
                for (idx, v) in arr.into_iter().enumerate() {
                    let sub_schema = sub_schema_for_index_from(schema.as_deref(), idx);
                    out.push(SchemaValue::from_value(v, sub_schema));
                }
                Node::Array(out)
            }
            v => Node::Scalar(v),
        };
        Self { node, schema }
    }

    /// Borrow the schema if present.
    pub fn schema(&self) -> Option<&Value> {
        self.schema.as_deref()
    }

    /// Return schema as plain JSON (full schema).
    pub fn get_schema(&self, path: Option<&str>) -> Option<Value> {
        let schema = self.schema.as_deref()?;
        if path.is_none() || path == Some("") {
            return Some(schema.clone());
        }
        let mut node = schema;
        for part in path.unwrap().split('.') {
            if part.is_empty() {
                continue;
            }
            let props = node.get("properties")?.as_object()?;
            node = props.get(part)?;
        }
        Some(node.clone())
    }

    /// Return x-* extensions from schema (optionally by dot path).
    pub fn get_extensions(&self, path: Option<&str>) -> BTreeMap<String, Value> {
        let mut out = BTreeMap::new();
        let Some(node) = self.get_schema(path) else { return out };
        let Some(obj) = node.as_object() else { return out };
        for (k, v) in obj {
            if k.starts_with("x-") {
                out.insert(k.clone(), v.clone());
            }
        }
        out
    }

    /// True if the inner value is null.
    pub fn is_null(&self) -> bool {
        matches!(self.node, Node::Scalar(Value::Null))
    }

    /// True if the inner value is a boolean.
    pub fn is_boolean(&self) -> bool {
        matches!(self.node, Node::Scalar(Value::Bool(_)))
    }

    /// True if the inner value is a number.
    pub fn is_number(&self) -> bool {
        matches!(self.node, Node::Scalar(Value::Number(_)))
    }

    /// True if the inner value is a string.
    pub fn is_string(&self) -> bool {
        matches!(self.node, Node::Scalar(Value::String(_)))
    }

    /// True if the inner value is an array.
    pub fn is_array(&self) -> bool {
        matches!(self.node, Node::Array(_))
    }

    /// True if the inner value is an object.
    pub fn is_object(&self) -> bool {
        matches!(self.node, Node::Object(_))
    }

    /// Extract as bool.
    pub fn as_bool(&self) -> Option<bool> {
        match &self.node {
            Node::Scalar(Value::Bool(b)) => Some(*b),
            _ => None,
        }
    }

    /// Extract as i64.
    pub fn as_i64(&self) -> Option<i64> {
        match &self.node {
            Node::Scalar(Value::Number(n)) => n.as_i64(),
            _ => None,
        }
    }

    /// Extract as u64.
    pub fn as_u64(&self) -> Option<u64> {
        match &self.node {
            Node::Scalar(Value::Number(n)) => n.as_u64(),
            _ => None,
        }
    }

    /// Extract as f64.
    pub fn as_f64(&self) -> Option<f64> {
        match &self.node {
            Node::Scalar(Value::Number(n)) => n.as_f64(),
            _ => None,
        }
    }

    /// Extract as string slice.
    pub fn as_str(&self) -> Option<&str> {
        match &self.node {
            Node::Scalar(Value::String(s)) => Some(s.as_str()),
            _ => None,
        }
    }

    // --- Mutation with validation ---

    /// Set a value by key. Validates against schema before writing.
    pub fn set(&mut self, key: &str, value: Value) -> Result<(), crate::ValidationError> {
        let Node::Object(map) = &mut self.node else {
            return Err(crate::ValidationError::root(
                crate::ErrorKind::NotAnObject,
                "cannot set key on non-object value",
            ));
        };

        if let Some(sub) = sub_schema_for_key_from(self.schema.as_deref(), key) {
            crate::validate::check_field(key, &sub, &value)?;
            map.insert(key.to_string(), SchemaValue::from_value(value, Some(sub)));
        } else {
            map.insert(key.to_string(), SchemaValue::from_value(value, None));
        }
        Ok(())
    }

    /// Set array element by index. Validates against items schema if present.
    pub fn set_index(&mut self, idx: usize, value: Value) -> Result<(), crate::ValidationError> {
        let Node::Array(arr) = &mut self.node else {
            return Err(crate::ValidationError::root(
                crate::ErrorKind::NotAnObject,
                "cannot set index on non-array value",
            ));
        };
        if let Some(sub) = sub_schema_for_index_from(self.schema.as_deref(), idx) {
            crate::validate::check_field(&format!("[{idx}]"), &sub, &value)?;
            if idx < arr.len() {
                arr[idx] = SchemaValue::from_value(value, Some(sub));
            } else {
                arr.push(SchemaValue::from_value(value, Some(sub)));
            }
        } else if idx < arr.len() {
            arr[idx] = SchemaValue::from_value(value, None);
        } else {
            arr.push(SchemaValue::from_value(value, None));
        }
        Ok(())
    }

    /// Apply default values from schema, returning a new SchemaValue.
    pub fn with_defaults(self) -> Self {
        let Some(schema) = &self.schema else { return self };
        let mut data = self.to_value();
        crate::defaults::apply_defaults(&mut data, schema.as_ref());
        SchemaValue::from_value(data, Some(schema.clone()))
    }

    /// Validate the entire data against the attached schema.
    pub fn validate(&self) -> Result<(), Vec<crate::ValidationError>> {
        let schema = match &self.schema {
            Some(s) => s,
            None => return Ok(()),
        };
        crate::validate::check_value(schema, &self.to_value()).map_err(|e| vec![e])
    }

    // --- Access ---

    /// Get a child by object key.
    pub fn get(&self, key: &str) -> Option<&SchemaValue> {
        match &self.node {
            Node::Object(map) => map.get(key),
            _ => None,
        }
    }

    /// Get a child by object key (mutable).
    pub fn get_mut(&mut self, key: &str) -> Option<&mut SchemaValue> {
        match &mut self.node {
            Node::Object(map) => map.get_mut(key),
            _ => None,
        }
    }

    /// Get a child by array index.
    pub fn get_index(&self, idx: usize) -> Option<&SchemaValue> {
        match &self.node {
            Node::Array(arr) => arr.get(idx),
            _ => None,
        }
    }

    /// Navigate a dot-separated path (e.g., "user.profile.name").
    pub fn path(&self, dot_path: &str) -> Option<&SchemaValue> {
        let mut current = self;
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
        match &self.node {
            Node::Object(map) => map.len(),
            Node::Array(arr) => arr.len(),
            _ => 0,
        }
    }

    /// True if null, empty object, or empty array.
    pub fn is_empty(&self) -> bool {
        match &self.node {
            Node::Scalar(Value::Null) => true,
            Node::Object(map) => map.is_empty(),
            Node::Array(arr) => arr.is_empty(),
            _ => false,
        }
    }

    /// Iterate over object entries as `(key, SchemaValue)` pairs.
    pub fn entries(&self) -> impl Iterator<Item = (&str, &SchemaValue)> {
        match &self.node {
            Node::Object(map) => map.iter().map(|(k, v)| (k.as_str(), v)).collect::<Vec<_>>().into_iter(),
            _ => Vec::new().into_iter(),
        }
    }

    /// Iterate over array elements as `SchemaValue`.
    pub fn elements(&self) -> impl Iterator<Item = &SchemaValue> {
        match &self.node {
            Node::Array(arr) => arr.iter().collect::<Vec<_>>().into_iter(),
            _ => Vec::new().into_iter(),
        }
    }

    /// Full data (includes unknown fields).
    pub fn to_value(&self) -> Value {
        match &self.node {
            Node::Scalar(v) => v.clone(),
            Node::Array(arr) => Value::Array(arr.iter().map(|v| v.to_value()).collect()),
            Node::Object(map) => {
                let mut out = serde_json::Map::new();
                for (k, v) in map {
                    out.insert(k.clone(), v.to_value());
                }
                Value::Object(out)
            }
        }
    }

    /// Schema-defined data only (unknown fields excluded).
    pub fn to_dict(&self) -> Value {
        match &self.node {
            Node::Scalar(v) => v.clone(),
            Node::Array(arr) => Value::Array(arr.iter().map(|v| v.to_dict()).collect()),
            Node::Object(map) => {
                let Some(schema) = &self.schema else {
                    return self.to_value();
                };
                let props = match schema.get("properties").and_then(|p| p.as_object()) {
                    Some(p) => p,
                    None => return Value::Object(serde_json::Map::new()),
                };
                let mut out = serde_json::Map::new();
                for key in props.keys() {
                    if let Some(v) = map.get(key) {
                        out.insert(key.clone(), v.to_dict());
                    }
                }
                Value::Object(out)
            }
        }
    }

}

/// Free function for sub-schema extraction (used by iterators that can't borrow self).
fn sub_schema_for_key_from(schema: Option<&Value>, key: &str) -> Option<Arc<Value>> {
    let schema = schema?;
    if let Some(prop_schema) = schema
        .get("properties")
        .and_then(|p| p.as_object())
        .and_then(|p| p.get(key))
    {
        return Some(Arc::new(prop_schema.clone()));
    }
    if let Some(patterns) = schema.get("patternProperties").and_then(|p| p.as_object()) {
        for (pattern, pat_schema) in patterns {
            if let Ok(re) = cached_regex(pattern) {
                if re.is_match(key) {
                    return Some(Arc::new(pat_schema.clone()));
                }
            }
        }
    }
    if let Some(additional) = schema.get("additionalProperties") {
        if additional.is_object() {
            return Some(Arc::new(additional.clone()));
        }
    }
    None
}

fn sub_schema_for_index_from(schema: Option<&Value>, idx: usize) -> Option<Arc<Value>> {
    let schema = schema?;
    let items = schema.get("items")?;
    if let Some(arr) = items.as_array() {
        if let Some(item_schema) = arr.get(idx) {
            return Some(Arc::new(item_schema.clone()));
        }
        if let Some(additional) = schema.get("additionalItems") {
            if additional.is_object() {
                return Some(Arc::new(additional.clone()));
            }
        }
        return None;
    }
    if items.is_object() {
        Some(Arc::new(items.clone()))
    } else {
        None
    }
}

// --- From / Into ---

impl From<Value> for SchemaValue {
    fn from(value: Value) -> Self {
        Self::without_schema(value)
    }
}

impl From<SchemaValue> for Value {
    fn from(sv: SchemaValue) -> Self {
        sv.to_value()
    }
}

// --- Display ---

impl fmt::Display for SchemaValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_value())
    }
}

// --- PartialEq (compare data only, ignore schema) ---

impl PartialEq for SchemaValue {
    fn eq(&self, other: &Self) -> bool {
        self.to_value() == other.to_value()
    }
}

impl PartialEq<Value> for SchemaValue {
    fn eq(&self, other: &Value) -> bool {
        &self.to_value() == other
    }
}

// --- Serialize / Deserialize (delegates to data) ---

impl Serialize for SchemaValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_value().serialize(serializer)
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
