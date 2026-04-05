/// schema2object — Rust
/// JSON Schema IS the object class.
/// Compile-time: proc-macro generates typed structs from schema.
/// Runtime: self-contained JSON parser + ObjectTree for dynamic use.

mod json;
mod tree;

pub use json::JsonNode;
pub use tree::{ObjectTree, ValidationError};
pub use schema2object_macro::schema;
