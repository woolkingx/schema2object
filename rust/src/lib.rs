//! # schema2object (Rust)
//!
//! JSON Schema Draft-07 object definition for Rust.
//!
//! Schema defines the class. Data is the instance.
//!
//! ```
//! use schema2object::ObjectTree;
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
//! let mut sv = ObjectTree::new(json!({"name": "Alice"}), schema);
//! sv.set("age", json!(30)).unwrap();
//! assert_eq!(sv.get("name").unwrap().to_value(), json!("Alice"));
//! ```

pub mod error;
pub mod value;
pub mod validate;
pub mod compose;
pub mod defaults;

pub use error::{ErrorKind, ValidationError};
pub use value::ObjectTree;
