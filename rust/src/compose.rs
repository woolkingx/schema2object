use serde_json::Value;

use crate::error::{ErrorKind, ValidationError};
use crate::validate::check_value;
use crate::SchemaValue;

/// Check if data matches a sub-schema (no error details needed).
fn matches(schema: &Value, data: &Value) -> bool {
    check_value(schema, data).is_ok()
}

impl SchemaValue {
    /// `oneOf` (XOR): select the unique matching branch from `schema.oneOf`.
    pub fn one_of(&self) -> Result<SchemaValue, ValidationError> {
        let subs = self.schema_array_keyword("oneOf");
        if subs.is_empty() {
            return Ok(self.clone());
        }

        let data = self.to_value();
        let matched: Vec<&Value> = subs.iter().filter(|s| matches(s, &data)).collect();
        match matched.len() {
            0 => Err(ValidationError::root(
                ErrorKind::OneOfNoneMatch,
                "oneOf: no branch matches",
            )),
            1 => Ok(SchemaValue::new(data, matched[0].clone())),
            n => Err(ValidationError::root(
                ErrorKind::OneOfMultipleMatch,
                format!("oneOf: expected 1 match, got {n}"),
            )),
        }
    }

    /// `anyOf` (OR): return all matching branches from `schema.anyOf`.
    pub fn any_of(&self) -> Result<Vec<SchemaValue>, ValidationError> {
        let subs = self.schema_array_keyword("anyOf");
        if subs.is_empty() {
            return Ok(vec![self.clone()]);
        }

        let data = self.to_value();
        let results: Vec<SchemaValue> = subs
            .iter()
            .filter(|s| matches(s, &data))
            .map(|s| SchemaValue::new(data.clone(), s.clone()))
            .collect();

        if results.is_empty() {
            Err(ValidationError::root(
                ErrorKind::AnyOfNoneMatch,
                "anyOf: no branch matches",
            ))
        } else {
            Ok(results)
        }
    }

    /// `allOf` (AND): merge all sub-schemas from `schema.allOf`.
    pub fn all_of(&self) -> SchemaValue {
        let schema = match self.schema() {
            Some(s) => s,
            None => return self.clone(),
        };

        let subs = match schema.get("allOf").and_then(|v| v.as_array()) {
            Some(arr) => arr,
            None => return self.clone(),
        };

        let mut merged_props = serde_json::Map::new();
        let mut merged_required: Vec<String> = Vec::new();
        let mut merged_items: Option<Value> = None;

        let sources: Vec<&Value> = std::iter::once(schema).chain(subs.iter()).collect();

        for src in sources {
            if let Some(props) = src.get("properties").and_then(|v| v.as_object()) {
                for (k, v) in props {
                    merged_props.insert(k.clone(), v.clone());
                }
            }
            if let Some(req) = src.get("required").and_then(|v| v.as_array()) {
                for r in req {
                    if let Some(s) = r.as_str() {
                        if !merged_required.contains(&s.to_string()) {
                            merged_required.push(s.to_string());
                        }
                    }
                }
            }
            if let Some(items) = src.get("items") {
                merged_items = Some(items.clone());
            }
        }

        let mut merged_schema = serde_json::Map::new();
        if !merged_props.is_empty() {
            merged_schema.insert("properties".to_string(), Value::Object(merged_props));
        }
        if !merged_required.is_empty() {
            merged_schema.insert(
                "required".to_string(),
                Value::Array(merged_required.into_iter().map(Value::String).collect()),
            );
        }
        if let Some(items) = merged_items {
            merged_schema.insert("items".to_string(), items);
        }

        SchemaValue::new(self.to_value(), Value::Object(merged_schema))
    }

    /// `not` (EXCEPT): true if data does NOT match the given or `schema.not` schema.
    pub fn not_of(&self, schema: Option<&Value>) -> bool {
        let target = schema.or_else(|| self.schema().and_then(|s| s.get("not")));
        match target {
            Some(s) => !matches(s, &self.to_value()),
            None => true,
        }
    }

    /// `if/then/else` (CASE WHEN): conditional branching.
    pub fn if_then(&self) -> SchemaValue {
        let schema = match self.schema() {
            Some(s) => s,
            None => return self.clone(),
        };

        let if_schema = match schema.get("if") {
            Some(s) => s,
            None => return self.clone(),
        };

        let branch = if matches(if_schema, &self.to_value()) {
            schema.get("then")
        } else {
            schema.get("else")
        };

        match branch {
            Some(b) => SchemaValue::new(self.to_value(), b.clone()),
            None => self.clone(),
        }
    }

    /// `project` (SELECT): keep only schema-defined properties.
    pub fn project(&self) -> Result<SchemaValue, ValidationError> {
        let schema = match self.schema() {
            Some(s) => s,
            None => return Ok(self.clone()),
        };

        let resolved = self.resolve_single_branch(schema)?;
        let resolved_ref = resolved.as_ref().unwrap_or(schema);

        let props = match resolved_ref.get("properties").and_then(|v| v.as_object()) {
            Some(p) => p,
            None => return Ok(self.clone()),
        };

        let data = self.to_value();
        let data_obj = match data.as_object() {
            Some(o) => o,
            None => return Ok(self.clone()),
        };

        let filtered: serde_json::Map<String, Value> = props
            .keys()
            .filter_map(|k| data_obj.get(k).map(|v| (k.clone(), v.clone())))
            .collect();

        Ok(SchemaValue::new(Value::Object(filtered), resolved_ref.clone()))
    }

    /// `contains` (EXISTS): true if any array element matches the schema.
    pub fn contains(&self, schema: Option<&Value>) -> bool {
        let target = schema.or_else(|| self.schema().and_then(|s| s.get("contains")));
        let target = match target {
            Some(s) => s,
            None => return false,
        };
        let data = self.to_value();
        let arr = match data.as_array() {
            Some(a) => a,
            None => return false,
        };
        arr.iter().any(|item| matches(target, item))
    }

    // --- Helpers ---

    fn schema_array_keyword(&self, keyword: &str) -> Vec<Value> {
        self.schema()
            .and_then(|s| s.get(keyword))
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default()
    }

    fn resolve_single_branch(&self, schema: &Value) -> Result<Option<Value>, ValidationError> {
        let data = self.to_value();
        for keyword in ["oneOf", "anyOf"] {
            if let Some(subs) = schema.get(keyword).and_then(|v| v.as_array()) {
                let matched: Vec<&Value> = subs.iter().filter(|s| matches(s, &data)).collect();
                match matched.len() {
                    0 => {}
                    1 => return Ok(Some(matched[0].clone())),
                    n => {
                        let method = keyword.replace("Of", "_of");
                        return Err(ValidationError::root(
                            ErrorKind::OneOfMultipleMatch,
                            format!(
                                "project: {keyword} has {n} matches, use {method}() first"
                            ),
                        ));
                    }
                }
            }
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_one_of_unique() {
        let schema = json!({
            "oneOf": [
                {"properties": {"type": {"const": "user"}, "name": {"type": "string"}}},
                {"properties": {"type": {"const": "bot"}, "id": {"type": "integer"}}}
            ]
        });
        let sv = SchemaValue::new(json!({"type": "user", "name": "Alice"}), schema);
        let branch = sv.one_of().unwrap();
        assert!(branch.schema().is_some());
    }
}
