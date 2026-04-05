# schema2object — Rust

JSON Schema IS the object class. Zero dependencies.

## Two-Layer Architecture

### Compile-time: Proc-Macro

Schema → typed struct. Construction IS validation.

```rust
use schema2object::schema;

#[schema("user.schema.json")]
struct User;

let user = User::new(
    "Alice".to_string(),    // name (required)
    Some(30),               // age (optional, default 0)
    "alice@example.com".to_string(),
    None,                   // address (optional)
).unwrap();

user.name   // String — compile-time typed
user.age    // i64 — default applied
user.email  // String
```

What the macro generates:
- Typed struct with fields from `schema.properties`
- Nested sub-structs for object properties
- `new()` — constructor with constraint validation (minimum/maximum/enum/const/length)
- `validate()` — same as new, returns `Result`
- `to_json()` — JSON string output
- `to_dict()` — `HashMap<String, String>`
- `schema()` — raw schema JSON string
- `get_extensions()` — all `x-*` keys

### Runtime: ObjectTree

Dynamic access when schema isn't known at compile time.

```rust
use schema2object::{JsonNode, ObjectTree};

let schema = JsonNode::from_file("user.schema.json").unwrap();
let data = JsonNode::parse(r#"{"name": "Alice", "age": 30}"#).unwrap();
let user = ObjectTree::new(data, schema).unwrap();

user["name"].as_str()  // Some("Alice")
user["age"].as_i64()   // Some(30)
```

Runtime methods: `one_of()`, `any_of()`, `all_of()`, `if_then()`, `not_of()`, `contains()`, `project()`, `with_defaults()`, `get_schema()`, `get_extensions()`, `to_value()`.

## Test

```bash
cargo test          # 17 tests
cargo run --example macro_usage
cargo run --example usage
```

## Files

```
src/json.rs                    # JSON parser (~320 lines)
src/tree.rs                    # ObjectTree runtime (~430 lines)
src/lib.rs                     # re-exports
schema2object-macro/src/lib.rs # proc-macro (~640 lines)
```

Total: ~1400 lines, zero external dependencies.
