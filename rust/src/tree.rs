/// ObjectTree — JSON Schema IS the object class.
/// Built from two ASTs: data + schema.

use crate::json::JsonNode;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug)]
pub struct ObjectTree {
    value: TreeValue,
    schema: JsonNode,
}

#[derive(Debug)]
enum TreeValue {
    Object(HashMap<String, ObjectTree>),
    Array(Vec<JsonNode>),
    String(String),
    Number(f64),
    Bool(bool),
    Null,
}

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub path: String,
    pub msg: String,
}

impl ValidationError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self { path: "$".to_string(), msg: msg.into() }
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.path, self.msg)
    }
}

impl std::error::Error for ValidationError {}

impl ObjectTree {
    pub fn new(data: JsonNode, schema: JsonNode) -> Result<Self, ValidationError> {
        Self::build(data, &schema, "$")
    }

    fn build(data: JsonNode, schema: &JsonNode, path: &str) -> Result<Self, ValidationError> {
        // Validate type
        Self::validate_type(&data, schema, path)?;

        let value = match data {
            JsonNode::Object(map) => {
                let props = schema.get("properties");
                let mut tree_map = HashMap::new();

                for (key, val) in map {
                    let child_path = format!("{}.{}", path, key);
                    let child_schema = props
                        .and_then(|p| p.get(&key))
                        .cloned()
                        .unwrap_or(JsonNode::Bool(true));

                    // Nested object → wrap as ObjectTree
                    let child = Self::build(val, &child_schema, &child_path)?;
                    tree_map.insert(key, child);
                }

                // Apply defaults for missing properties
                if let Some(JsonNode::Object(prop_map)) = props {
                    for (key, prop_schema) in prop_map {
                        if !tree_map.contains_key(key) {
                            if let Some(default) = prop_schema.get("default") {
                                let child_path = format!("{}.{}", path, key);
                                let child = Self::build(default.clone(), prop_schema, &child_path)?;
                                tree_map.insert(key.clone(), child);
                            }
                        }
                    }
                }

                // Check required
                if let Some(JsonNode::Array(req)) = schema.get("required") {
                    for r in req {
                        if let JsonNode::String(key) = r {
                            if !tree_map.contains_key(key) {
                                return Err(ValidationError {
                                    path: path.to_string(),
                                    msg: format!("missing required property '{}'", key),
                                });
                            }
                        }
                    }
                }

                TreeValue::Object(tree_map)
            }
            JsonNode::Array(arr) => TreeValue::Array(arr),
            JsonNode::String(s) => TreeValue::String(s),
            JsonNode::Number(n) => TreeValue::Number(n),
            JsonNode::Bool(b) => TreeValue::Bool(b),
            JsonNode::Null => TreeValue::Null,
        };

        Ok(ObjectTree { value, schema: schema.clone() })
    }

    fn validate_type(data: &JsonNode, schema: &JsonNode, path: &str) -> Result<(), ValidationError> {
        // boolean schema
        match schema {
            JsonNode::Bool(true) => return Ok(()),
            JsonNode::Bool(false) => return Err(ValidationError {
                path: path.to_string(),
                msg: "schema is false".to_string(),
            }),
            _ => {}
        }

        let type_val = match schema.get("type") {
            Some(t) => t,
            None => return Ok(()), // no type constraint
        };

        let type_str = match type_val.as_str() {
            Some(s) => s,
            None => return Ok(()),
        };

        let ok = match type_str {
            "object" => data.is_object(),
            "array" => data.is_array(),
            "string" => matches!(data, JsonNode::String(_)),
            "number" => matches!(data, JsonNode::Number(_)),
            "integer" => match data {
                JsonNode::Number(n) => *n == (*n as i64) as f64,
                _ => false,
            },
            "boolean" => matches!(data, JsonNode::Bool(_)),
            "null" => matches!(data, JsonNode::Null),
            _ => true,
        };

        if !ok {
            return Err(ValidationError {
                path: path.to_string(),
                msg: format!("expected {}, got {}", type_str, data_type_name(data)),
            });
        }

        // Constraint validation
        if let JsonNode::Number(n) = data {
            if let Some(JsonNode::Number(min)) = schema.get("minimum") {
                if n < min {
                    return Err(ValidationError {
                        path: path.to_string(),
                        msg: format!("must be >= {}", min),
                    });
                }
            }
            if let Some(JsonNode::Number(max)) = schema.get("maximum") {
                if n > max {
                    return Err(ValidationError {
                        path: path.to_string(),
                        msg: format!("must be <= {}", max),
                    });
                }
            }
        }

        if let JsonNode::String(s) = data {
            if let Some(JsonNode::Number(min)) = schema.get("minLength") {
                if (s.chars().count() as f64) < *min {
                    return Err(ValidationError {
                        path: path.to_string(),
                        msg: format!("length must be >= {}", min),
                    });
                }
            }
            if let Some(JsonNode::Number(max)) = schema.get("maxLength") {
                if (s.chars().count() as f64) > *max {
                    return Err(ValidationError {
                        path: path.to_string(),
                        msg: format!("length must be <= {}", max),
                    });
                }
            }
        }

        Ok(())
    }

    // ─── Access ──────────────────────────────────────────────────────────────

    pub fn get(&self, key: &str) -> Option<&ObjectTree> {
        match &self.value {
            TreeValue::Object(map) => map.get(key),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match &self.value { TreeValue::String(s) => Some(s), _ => None }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match &self.value { TreeValue::Number(n) => Some(*n), _ => None }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match &self.value {
            TreeValue::Number(n) => {
                let i = *n as i64;
                if (i as f64) == *n { Some(i) } else { None }
            }
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match &self.value { TreeValue::Bool(b) => Some(*b), _ => None }
    }

    pub fn is_null(&self) -> bool {
        matches!(&self.value, TreeValue::Null)
    }

    pub fn schema(&self) -> &JsonNode {
        &self.schema
    }

    // ─── Composition methods (runtime) ──────────────────────────────────────

    /// oneOf — exactly one sub-schema must match
    pub fn one_of(&self) -> Result<ObjectTree, ValidationError> {
        let subs = self.schema.get("oneOf").and_then(|v| match v {
            JsonNode::Array(a) => Some(a),
            _ => None,
        }).ok_or_else(|| ValidationError {
            path: "$".to_string(), msg: "schema has no oneOf".to_string(),
        })?;
        let data = self.to_value();
        let matches: Vec<&JsonNode> = subs.iter()
            .filter(|s| Self::build(data.clone(), s, "$").is_ok()).collect();
        match matches.len() {
            1 => Self::build(data, matches[0], "$"),
            n => Err(ValidationError {
                path: "$".to_string(),
                msg: format!("oneOf: expected 1 match, got {}", n),
            }),
        }
    }

    /// anyOf — all matching sub-schemas
    pub fn any_of(&self) -> Result<Vec<ObjectTree>, ValidationError> {
        let subs = self.schema.get("anyOf").and_then(|v| match v {
            JsonNode::Array(a) => Some(a),
            _ => None,
        }).ok_or_else(|| ValidationError {
            path: "$".to_string(), msg: "schema has no anyOf".to_string(),
        })?;
        let data = self.to_value();
        let results: Vec<ObjectTree> = subs.iter()
            .filter_map(|s| Self::build(data.clone(), s, "$").ok()).collect();
        if results.is_empty() {
            Err(ValidationError { path: "$".to_string(), msg: "anyOf: no branch matches".to_string() })
        } else { Ok(results) }
    }

    /// allOf — deep-merge all sub-schemas, re-bind
    pub fn all_of(&self) -> Result<ObjectTree, ValidationError> {
        let subs = match self.schema.get("allOf") {
            Some(JsonNode::Array(a)) => a,
            _ => return Ok(ObjectTree { value: self.rebuild_value(), schema: self.schema.clone() }),
        };
        let merged = subs.iter().fold(JsonNode::Object(HashMap::new()), |acc, s| deep_merge_nodes(acc, s));
        Self::build(self.to_value(), &merged, "$")
    }

    /// notOf — true if data does NOT match 'not' schema
    pub fn not_of(&self) -> bool {
        match self.schema.get("not") {
            Some(not_schema) => Self::build(self.to_value(), not_schema, "$").is_err(),
            None => true,
        }
    }

    /// ifThen — if data matches 'if', bind to 'then'; else bind to 'else'
    pub fn if_then(&self) -> Result<ObjectTree, ValidationError> {
        let if_schema = match self.schema.get("if") {
            Some(s) => s,
            None => return Ok(ObjectTree { value: self.rebuild_value(), schema: self.schema.clone() }),
        };
        let data = self.to_value();
        let branch = if Self::build(data.clone(), if_schema, "$").is_ok() {
            self.schema.get("then")
        } else {
            self.schema.get("else")
        };
        match branch {
            Some(b) => Self::build(data, b, "$"),
            None => Ok(ObjectTree { value: self.rebuild_value(), schema: self.schema.clone() }),
        }
    }

    /// project — keep only schema-defined properties
    pub fn project(&self) -> Result<ObjectTree, ValidationError> {
        match &self.value {
            TreeValue::Object(map) => {
                let props = self.schema.get("properties");
                let filtered: HashMap<String, ObjectTree> = match props {
                    Some(JsonNode::Object(prop_map)) => {
                        map.iter()
                            .filter(|(k, _)| prop_map.contains_key(k.as_str()))
                            .map(|(k, v)| (k.clone(), ObjectTree { value: v.rebuild_value(), schema: v.schema.clone() }))
                            .collect()
                    }
                    _ => map.iter().map(|(k, v)| (k.clone(), ObjectTree { value: v.rebuild_value(), schema: v.schema.clone() })).collect(),
                };
                Ok(ObjectTree {
                    value: TreeValue::Object(filtered),
                    schema: self.schema.clone(),
                })
            }
            _ => Err(ValidationError { path: "$".to_string(), msg: "project: data must be object".to_string() }),
        }
    }

    /// withDefaults — fill missing fields from schema defaults (deep)
    pub fn with_defaults(&self) -> Result<ObjectTree, ValidationError> {
        let data = self.to_value();
        Self::build(data, &self.schema, "$")
    }

    /// contains — true if any array element matches 'contains' schema
    pub fn contains(&self) -> Option<bool> {
        let arr = match &self.value {
            TreeValue::Array(a) => a,
            _ => return None,
        };
        let cs = self.schema.get("contains")?;
        Some(arr.iter().any(|item| Self::build(item.clone(), cs, "$").is_ok()))
    }

    /// get_schema — navigate to sub-schema by dot path
    pub fn get_schema(&self, path: &str) -> Option<&JsonNode> {
        if path.is_empty() { return Some(&self.schema); }
        let mut current = &self.schema;
        for part in path.split('.') {
            current = current.get("properties")?.get(part)?;
        }
        Some(current)
    }

    /// get_extensions — all x-* keys from schema
    pub fn get_extensions(&self) -> HashMap<String, JsonNode> {
        let mut m = HashMap::new();
        if let JsonNode::Object(obj) = &self.schema {
            for (k, v) in obj {
                if k.starts_with("x-") {
                    m.insert(k.clone(), v.clone());
                }
            }
        }
        m
    }

    // ─── Internal helpers ─────────────────────────────────────────────────

    fn rebuild_value(&self) -> TreeValue {
        match &self.value {
            TreeValue::Object(map) => TreeValue::Object(
                map.iter().map(|(k, v)| (k.clone(), ObjectTree { value: v.rebuild_value(), schema: v.schema.clone() })).collect()
            ),
            TreeValue::Array(a) => TreeValue::Array(a.clone()),
            TreeValue::String(s) => TreeValue::String(s.clone()),
            TreeValue::Number(n) => TreeValue::Number(*n),
            TreeValue::Bool(b) => TreeValue::Bool(*b),
            TreeValue::Null => TreeValue::Null,
        }
    }

    /// Export to plain JsonNode (like JS $value)
    pub fn to_value(&self) -> JsonNode {
        match &self.value {
            TreeValue::Object(map) => {
                let mut out = HashMap::new();
                for (k, v) in map {
                    out.insert(k.clone(), v.to_value());
                }
                JsonNode::Object(out)
            }
            TreeValue::Array(arr) => JsonNode::Array(arr.clone()),
            TreeValue::String(s) => JsonNode::String(s.clone()),
            TreeValue::Number(n) => JsonNode::Number(*n),
            TreeValue::Bool(b) => JsonNode::Bool(*b),
            TreeValue::Null => JsonNode::Null,
        }
    }
}

impl std::ops::Index<&str> for ObjectTree {
    type Output = ObjectTree;
    fn index(&self, key: &str) -> &ObjectTree {
        static NULL_TREE: std::sync::LazyLock<ObjectTree> = std::sync::LazyLock::new(|| ObjectTree {
            value: TreeValue::Null,
            schema: JsonNode::Bool(true),
        });
        self.get(key).unwrap_or(&NULL_TREE)
    }
}

impl fmt::Display for ObjectTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_value())
    }
}

fn deep_merge_nodes(mut base: JsonNode, overlay: &JsonNode) -> JsonNode {
    match (&mut base, overlay) {
        (JsonNode::Object(ref mut a), JsonNode::Object(b)) => {
            for (k, v) in b {
                let merged = match a.get(k) {
                    Some(existing) => deep_merge_nodes(existing.clone(), v),
                    None => v.clone(),
                };
                a.insert(k.clone(), merged);
            }
            base
        }
        _ => overlay.clone(),
    }
}

fn data_type_name(data: &JsonNode) -> &'static str {
    match data {
        JsonNode::Object(_) => "object",
        JsonNode::Array(_) => "array",
        JsonNode::String(_) => "string",
        JsonNode::Number(_) => "number",
        JsonNode::Bool(_) => "boolean",
        JsonNode::Null => "null",
    }
}
