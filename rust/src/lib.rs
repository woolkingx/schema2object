//! # schema-value
//!
//! JSON Schema Draft-07 object wrapper for Rust.
//!
//! Structure maps to accessors, logic maps to methods.
//!
//! ```
//! use schema_value::SchemaValue;
//! use serde_json::json;
//!
//! let schema = json!({
//!     "type": "object",
//!     "properties": {
//!         "name": {"type": "string"},
//!         "age": {"type": "integer", "minimum": 0}
//!     }
//! });
//!
//! let mut sv = SchemaValue::new(json!({"name": "Alice"}), schema);
//! assert_eq!(sv["name"], json!("Alice"));
//! ```

pub mod error;
pub mod value;
pub mod validate;
pub mod compose;
pub mod defaults;

pub use error::{ErrorKind, ValidationError};
pub use value::SchemaValue;
