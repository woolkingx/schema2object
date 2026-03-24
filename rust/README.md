# schema2object (Rust)

JSON Schema Draft-07 as object definition in Rust.

## Core Idea

Schema defines the class; data is the instance. Access returns schema-aware objects,
mutation validates, and Draft-07 logic maps to methods.

## Install

```bash
cargo add schema2object
```

## Quick Example

```rust
use schema2object::ObjectTree;
use serde_json::json;

let schema = json!({
  "type": "object",
  "properties": {
    "name": {"type": "string"},
    "age":  {"type": "integer", "minimum": 0, "x-docs": "Age in years"}
  }
});

let mut user = ObjectTree::new(json!({"name": "Alice"}), schema);
user.set("age", json!(30)).unwrap();

let name = user.get("name").unwrap();
assert_eq!(name.to_value(), json!("Alice"));

// Explicit schema access
let age_schema = user.get_schema(Some("age")).unwrap();
assert_eq!(age_schema["minimum"], json!(0));
let x = user.get_extensions(Some("age"));
assert_eq!(x.get("x-docs").unwrap(), "Age in years");
```

## API Notes

- `get/get_index/path` return `ObjectTree`
- `set/set_index` validate on write
- `to_dict()` returns schema-defined fields only
- `to_value()` returns full data (including unknown fields)
- `get_schema(path)` reads schema (dot path supported)
- `get_extensions(path)` reads `x-*` extensions

## Tests

```bash
cargo test
```
