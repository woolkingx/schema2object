# Changelog

## [0.5.0] - 2026-03-30

### Changed (JS)
- **Lazy defaults** — schema `default` values are the initial value of any field. Reading a field missing from raw data returns its schema default transparently. `Object.keys(tree)` and `in` include default-only fields. Applies at all nesting levels.
- **Array cursor** — array properties wrap as a lazy Proxy cursor instead of eager `.map()`. Per-index ObjectTree instances are created on first access and cached for stable identity (`arr[0] === arr[0]`). Mutations invalidate only the affected index. `for...of`, spread, and destructuring work correctly.
- **`$withDefaults()` semantics** — materializes defaults into raw data. Recursively fills nested objects and array items.
- **`$value` includes defaults** — includes schema-default fields and expands array item defaults.

### Internal (JS)
- **ctx pipeline** — internal rewrite: C-style ctx struct + pure fn pipeline replaces class methods. External API unchanged.
- **Removed graph scheduler** — simplified execution model.

### Fixed (JS)
- **Nested remote `$ref`** — correctly resolves nested `$ref` in remote schemas.

### Note
- Python and Rust implementations have not been updated for 0.5.0 changes (lazy defaults, array cursor, ctx pipeline). They remain at 0.4.x semantics.

## [0.4.1] - 2026-03-28

### Breaking Changes (JS)
- **Filesystem resolver no longer accepts HTTP URIs** — `$ref` with `http://` scheme now throws `TypeError` when resolver is a directory path string. Use function or object-map resolver for HTTP schemas.
- **Resolver no longer retries with original `$ref`** — only the normalized URI is passed to resolver. Resolver must handle the canonical form.

### Fixed
- **Loader `#idMap` lookup** (JS) — removed incorrect fallback that searched fragment-bearing URIs against `$id` map
- **Test runner** — switched from filesystem resolver to function resolver for Draft-07 remote `$ref` tests, correctly exercising the resolver callback interface

### Removed
- `#fileExists` helper, `.json` extension auto-append, path traversal guard — these were runtime concerns that don't belong in schema tooling

## [0.4.0] - 2026-03-21

### Breaking Changes (JS)
- **Unified `#value` storage** — removed `#isObjectNode()` and dual `#value`/`#data` fields. Every schema node is an object; `type`/`properties` are just attributes, not node classification. Schemas without `type` or `properties` (e.g. pure `anyOf`/`oneOf`) now correctly build ObjectTree.
- **Removed file path as schema parameter** — schema must be a plain object. Read and parse files yourself; the lib doesn't do I/O.

### Fixed
- **Nested wrapping for schemeless nodes** (JS) — schemas without `type: "object"` or `properties` now correctly wrap nested data as ObjectTree
- **Primitive data storage** (JS) — `ObjectTree("hello", { type: "string" })` now correctly stores and returns the value

### Changed
- **`#defineProperties` guard** (JS) — handles `null`/boolean schemas without crashing
- **README** — rewritten constructor docs, added "Loading Schema from File" section

## [0.3.0] - 2026-03-17

### Breaking Changes (JS only)
- **`$` prefix on all ObjectTree API** — `value` → `$value`, `toDict()` → `$toDict()`, `oneOf()` → `$oneOf()`, etc. Data properties keep no prefix (`tree.name`). Prevents collision when schema property names match API method names.

### Added
- **`toJSON()` protocol** (JS) — `JSON.stringify(tree)` now works correctly, recursively unwraps nested ObjectTree instances
- **Invalid schema guard** (JS) — `validateType` rejects `null`/non-object schemas with clear error message

### Fixed
- **Eager nested wrapping** (JS) — nested objects eagerly wrapped as ObjectTree at construction, aligned with Python/Rust. Mutation at any depth validates against sub-schema.
- **Default preservation** (JS) — constructor overlays data onto defaults instead of replacing; `set $value` also preserves defaults
- **anyOf/oneOf sub-schema wrapping** (JS) — removed `isObj` guard; any object value is wrapped regardless of schema shape
- **`$project()`** (JS) — uses constructor instead of direct `#data` assignment, ensuring nested wrapping

### Changed
- **`configurable: false`** (JS) — property descriptors block `delete` to prevent validation bypass

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
