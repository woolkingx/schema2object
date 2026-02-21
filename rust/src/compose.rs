use std::sync::Arc;

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
    ///
    /// Returns a new `SchemaValue` bound to the matching sub-schema.
    /// Errors if zero or more than one branch matches.
    pub fn one_of(&self) -> Result<SchemaValue, ValidationError> {
        let subs = self.schema_array_keyword("oneOf");
        if subs.is_empty() {
            return Ok(self.clone());
        }

        let matched: Vec<&Value> = subs.iter().filter(|s| matches(s, &self.data)).collect();
        match matched.len() {
            0 => Err(ValidationError::root(
                ErrorKind::OneOfNoneMatch,
                "oneOf: no branch matches",
            )),
            1 => Ok(SchemaValue {
                data: self.data.clone(),
                schema: Some(Arc::new(matched[0].clone())),
            }),
            n => Err(ValidationError::root(
                ErrorKind::OneOfMultipleMatch,
                format!("oneOf: expected 1 match, got {n}"),
            )),
        }
    }

    /// `anyOf` (OR): return all matching branches from `schema.anyOf`.
    ///
    /// Each returned `SchemaValue` is bound to its matching sub-schema.
    /// Errors if no branch matches.
    pub fn any_of(&self) -> Result<Vec<SchemaValue>, ValidationError> {
        let subs = self.schema_array_keyword("anyOf");
        if subs.is_empty() {
            return Ok(vec![self.clone()]);
        }

        let results: Vec<SchemaValue> = subs
            .iter()
            .filter(|s| matches(s, &self.data))
            .map(|s| SchemaValue {
                data: self.data.clone(),
                schema: Some(Arc::new(s.clone())),
            })
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
    ///
    /// Merges `properties` and `required` from all branches and the parent schema.
    /// Returns a new `SchemaValue` bound to the merged schema.
    pub fn all_of(&self) -> SchemaValue {
        let schema = match &self.schema {
            Some(s) => s.as_ref(),
            None => return self.clone(),
        };

        let subs = match schema.get("allOf").and_then(|v| v.as_array()) {
            Some(arr) => arr,
            None => return self.clone(),
        };

        let mut merged_props = serde_json::Map::new();
        let mut merged_required: Vec<String> = Vec::new();
        let mut merged_items: Option<Value> = None;

        // Collect from parent schema first, then allOf branches
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

        SchemaValue {
            data: self.data.clone(),
            schema: Some(Arc::new(Value::Object(merged_schema))),
        }
    }

    /// `not` (EXCEPT): true if data does NOT match the given or `schema.not` schema.
    pub fn not_of(&self, schema: Option<&Value>) -> bool {
        let target = schema.or_else(|| {
            self.schema
                .as_deref()
                .and_then(|s| s.get("not"))
        });
        match target {
            Some(s) => !matches(s, &self.data),
            None => true,
        }
    }

    /// `if/then/else` (CASE WHEN): conditional branching.
    ///
    /// Evaluates `schema.if` against data. If it matches, returns a SchemaValue
    /// bound to `schema.then`; otherwise, bound to `schema.else`.
    /// Returns self if no `if` keyword exists.
    pub fn if_then(&self) -> SchemaValue {
        let schema = match &self.schema {
            Some(s) => s.as_ref(),
            None => return self.clone(),
        };

        let if_schema = match schema.get("if") {
            Some(s) => s,
            None => return self.clone(),
        };

        let branch = if matches(if_schema, &self.data) {
            schema.get("then")
        } else {
            schema.get("else")
        };

        match branch {
            Some(b) => SchemaValue {
                data: self.data.clone(),
                schema: Some(Arc::new(b.clone())),
            },
            None => self.clone(),
        }
    }

    /// `project` (SELECT): keep only schema-defined properties.
    ///
    /// For `oneOf`/`anyOf` with exactly one match, auto-resolves the branch.
    pub fn project(&self) -> Result<SchemaValue, ValidationError> {
        let schema = match &self.schema {
            Some(s) => s.as_ref(),
            None => return Ok(self.clone()),
        };

        // Try auto-resolving oneOf/anyOf with single match
        let resolved = self.resolve_single_branch(schema)?;
        let resolved_ref = resolved.as_ref().unwrap_or(schema);

        let props = match resolved_ref.get("properties").and_then(|v| v.as_object()) {
            Some(p) => p,
            None => return Ok(self.clone()),
        };

        let data = match self.data.as_object() {
            Some(o) => o,
            None => return Ok(self.clone()),
        };

        let filtered: serde_json::Map<String, Value> = props
            .keys()
            .filter_map(|k| data.get(k).map(|v| (k.clone(), v.clone())))
            .collect();

        Ok(SchemaValue {
            data: Value::Object(filtered),
            schema: Some(Arc::new(resolved_ref.clone())),
        })
    }

    /// `contains` (EXISTS): true if any array element matches the schema.
    ///
    /// Uses `schema.contains` if no explicit schema is provided.
    pub fn contains(&self, schema: Option<&Value>) -> bool {
        let target = schema.or_else(|| {
            self.schema
                .as_deref()
                .and_then(|s| s.get("contains"))
        });
        let target = match target {
            Some(s) => s,
            None => return false,
        };
        let arr = match self.data.as_array() {
            Some(a) => a,
            None => return false,
        };
        arr.iter().any(|item| matches(target, item))
    }

    // --- Helpers ---

    /// Extract an array keyword from the attached schema.
    fn schema_array_keyword(&self, keyword: &str) -> Vec<Value> {
        self.schema
            .as_deref()
            .and_then(|s| s.get(keyword))
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default()
    }

    /// Try to resolve oneOf/anyOf to a single branch for projection.
    fn resolve_single_branch(
        &self,
        schema: &Value,
    ) -> Result<Option<Value>, ValidationError> {
        for keyword in ["oneOf", "anyOf"] {
            if let Some(subs) = schema.get(keyword).and_then(|v| v.as_array()) {
                let matched: Vec<&Value> =
                    subs.iter().filter(|s| self::matches(s, &self.data)).collect();
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

    // --- oneOf ---

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

    #[test]
    fn test_one_of_no_match() {
        let schema = json!({
            "oneOf": [
                {"properties": {"x": {"const": 1}}},
                {"properties": {"x": {"const": 2}}}
            ]
        });
        let sv = SchemaValue::new(json!({"x": 99}), schema);
        assert!(sv.one_of().is_err());
    }

    #[test]
    fn test_one_of_multiple_match() {
        let schema = json!({
            "oneOf": [
                {"properties": {"x": {"type": "integer"}}},
                {"properties": {"x": {"type": "number"}}}
            ]
        });
        let sv = SchemaValue::new(json!({"x": 42}), schema);
        assert!(sv.one_of().is_err());
    }

    #[test]
    fn test_one_of_no_schema() {
        let sv = SchemaValue::without_schema(json!({"x": 1}));
        let result = sv.one_of().unwrap();
        assert_eq!(result, sv);
    }

    // --- anyOf ---

    #[test]
    fn test_any_of_multiple() {
        let schema = json!({
            "anyOf": [
                {"properties": {"x": {"type": "integer"}}},
                {"properties": {"x": {"type": "number"}}}
            ]
        });
        let sv = SchemaValue::new(json!({"x": 42}), schema);
        let branches = sv.any_of().unwrap();
        assert_eq!(branches.len(), 2);
    }

    #[test]
    fn test_any_of_no_match() {
        let schema = json!({
            "anyOf": [
                {"properties": {"x": {"const": 1}}},
                {"properties": {"x": {"const": 2}}}
            ]
        });
        let sv = SchemaValue::new(json!({"x": 99}), schema);
        assert!(sv.any_of().is_err());
    }

    #[test]
    fn test_any_of_no_schema() {
        let sv = SchemaValue::without_schema(json!({"x": 1}));
        let result = sv.any_of().unwrap();
        assert_eq!(result.len(), 1);
    }

    // --- allOf ---

    #[test]
    fn test_all_of_merge() {
        let schema = json!({
            "allOf": [
                {"properties": {"name": {"type": "string"}}, "required": ["name"]},
                {"properties": {"age": {"type": "integer"}}, "required": ["age"]}
            ]
        });
        let sv = SchemaValue::new(json!({"name": "Alice", "age": 30}), schema);
        let merged = sv.all_of();
        let ms = merged.schema().unwrap();
        // Merged schema has both properties
        assert!(ms.get("properties").unwrap().get("name").is_some());
        assert!(ms.get("properties").unwrap().get("age").is_some());
        // Merged required
        let req = ms.get("required").unwrap().as_array().unwrap();
        assert_eq!(req.len(), 2);
    }

    #[test]
    fn test_all_of_no_allof() {
        let sv = SchemaValue::without_schema(json!({"x": 1}));
        let result = sv.all_of();
        assert_eq!(result, sv);
    }

    #[test]
    fn test_all_of_merge_required_no_duplicates() {
        let schema = json!({
            "required": ["a"],
            "allOf": [
                {"required": ["a", "b"]},
                {"required": ["b", "c"]}
            ]
        });
        let sv = SchemaValue::new(json!({"a": 1, "b": 2, "c": 3}), schema);
        let merged = sv.all_of();
        let req = merged.schema().unwrap().get("required").unwrap().as_array().unwrap();
        assert_eq!(req.len(), 3); // a, b, c — no duplicates
    }

    // --- not_of ---

    #[test]
    fn test_not_of_holds() {
        let schema = json!({"not": {"properties": {"x": {"const": 1}}}});
        let sv = SchemaValue::new(json!({"x": 2}), schema);
        assert!(sv.not_of(None));
    }

    #[test]
    fn test_not_of_fails() {
        let schema = json!({"not": {"properties": {"x": {"const": 1}}}});
        let sv = SchemaValue::new(json!({"x": 1}), schema);
        assert!(!sv.not_of(None));
    }

    #[test]
    fn test_not_of_explicit_schema() {
        let sv = SchemaValue::without_schema(json!(42));
        assert!(sv.not_of(Some(&json!({"type": "string"}))));
        assert!(!sv.not_of(Some(&json!({"type": "integer"}))));
    }

    // --- if_then ---

    #[test]
    fn test_if_then_match() {
        let schema = json!({
            "if": {"properties": {"role": {"const": "admin"}}},
            "then": {"properties": {"level": {"minimum": 5}}},
            "else": {"properties": {"level": {"maximum": 4}}}
        });
        let sv = SchemaValue::new(json!({"role": "admin", "level": 10}), schema);
        let result = sv.if_then();
        let rs = result.schema().unwrap();
        assert!(rs.get("properties").unwrap().get("level").unwrap().get("minimum").is_some());
    }

    #[test]
    fn test_if_then_no_match() {
        let schema = json!({
            "if": {"properties": {"role": {"const": "admin"}}},
            "then": {"properties": {"level": {"minimum": 5}}},
            "else": {"properties": {"level": {"maximum": 4}}}
        });
        let sv = SchemaValue::new(json!({"role": "user", "level": 2}), schema);
        let result = sv.if_then();
        let rs = result.schema().unwrap();
        assert!(rs.get("properties").unwrap().get("level").unwrap().get("maximum").is_some());
    }

    #[test]
    fn test_if_then_no_if() {
        let schema = json!({"then": {"properties": {"x": {"type": "integer"}}}});
        let sv = SchemaValue::new(json!({"x": 1}), schema);
        let result = sv.if_then();
        assert_eq!(result, sv);
    }

    // --- project ---

    #[test]
    fn test_project_filters() {
        let schema = json!({
            "properties": {
                "name": {"type": "string"},
                "age": {"type": "integer"}
            }
        });
        let sv = SchemaValue::new(json!({"name": "Alice", "age": 30, "extra": "x"}), schema);
        let projected = sv.project().unwrap();
        let data = projected.as_value().as_object().unwrap();
        assert!(data.contains_key("name"));
        assert!(data.contains_key("age"));
        assert!(!data.contains_key("extra"));
    }

    #[test]
    fn test_project_no_properties() {
        let schema = json!({"type": "object"});
        let sv = SchemaValue::new(json!({"a": 1}), schema);
        let result = sv.project().unwrap();
        assert_eq!(result, sv);
    }

    #[test]
    fn test_project_auto_resolve_oneof() {
        let schema = json!({
            "oneOf": [
                {"properties": {"type": {"const": "a"}, "x": {"type": "integer"}}},
                {"properties": {"type": {"const": "b"}, "y": {"type": "string"}}}
            ]
        });
        let sv = SchemaValue::new(json!({"type": "a", "x": 1, "extra": true}), schema);
        let projected = sv.project().unwrap();
        let data = projected.as_value().as_object().unwrap();
        assert!(data.contains_key("type"));
        assert!(data.contains_key("x"));
        assert!(!data.contains_key("extra"));
    }

    // --- contains ---

    #[test]
    fn test_contains_match() {
        let schema = json!({
            "type": "array",
            "contains": {"const": 42}
        });
        let sv = SchemaValue::new(json!([1, 42, 3]), schema);
        assert!(sv.contains(None));
    }

    #[test]
    fn test_contains_no_match() {
        let schema = json!({
            "type": "array",
            "contains": {"const": 99}
        });
        let sv = SchemaValue::new(json!([1, 2, 3]), schema);
        assert!(!sv.contains(None));
    }

    #[test]
    fn test_contains_explicit_schema() {
        let sv = SchemaValue::without_schema(json!([1, 2, 3]));
        assert!(sv.contains(Some(&json!({"const": 2}))));
        assert!(!sv.contains(Some(&json!({"const": 99}))));
    }

    #[test]
    fn test_contains_not_array() {
        let sv = SchemaValue::without_schema(json!({"a": 1}));
        assert!(!sv.contains(Some(&json!({"const": 1}))));
    }
}
