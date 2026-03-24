# Changelog

## [0.2.0] - 2026-03-14

### Added
- **Rust codegen** (`rust/codegen/`) — generates static `.rs` file: runtime source + typed structs from JSON Schema
  - `python rust/codegen/codegen.py schema.json --out src/generated.rs`
  - Full draft-07 suite (922/922) + API tests (44)
- **`docs/`** — architecture docs: `schema-as-object.md`, `schema2object-api.json`, `structure.md`, `types.md`, etc.
- **JS `validate()`** — standalone validation export without constructing ObjectTree
- **Rust `schema()`/`get_schema()`** — now use `schema_to_dict`, aligned with JS/Python

### Fixed
- **Internal re-bind Loader propagation** (JS + Python + Rust) — `oneOf`/`anyOf`/`allOf`/`ifThen`/`project`/`withDefaults` now pass Loader instance internally; sub-schemas no longer lose `$definitions` context
- **Rust `get_extensions()`** — empty path now calls `self.schema()` via `schema_to_dict`, consistent with JS/Python
- **`multipleOf` precision** — relative tolerance aligned across all three implementations
- **Resolver propagation** — `ifThen`/`project`/`withDefaults` correctly propagate resolver (JS + Python)
- **`required` ordering** — cross-language alignment

### Changed
- Rust rewritten as single file (`schema2object.rs`), aligned with JS/Python structure
- Python rewritten as single file (`schema2object.py`), full draft-07 suite
- JS full Draft-07 `$ref` resolution with flexible resolver API

## [0.1.0] - initial

- Three-language implementation: JS, Python, Rust
- JSON Schema Draft-07 as object class definition
- `ObjectTree` construction validates; mutation validates; `to_dict()` projects schema fields only
- Draft-07 combinators as methods: `oneOf`, `anyOf`, `allOf`, `notOf`, `ifThen`, `project`, `contains`, `withDefaults`
