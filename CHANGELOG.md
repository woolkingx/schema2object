# Changelog

Current repository posture: JavaScript is the only active runtime implementation. Python and Rust entries below are historical release notes unless they explicitly describe projection documentation.

## Chapter Role

| Field | Value |
|---|---|
| Input artifact | A release or historical behavior question |
| Output artifact | Historical release record |
| Owner | Project history |
| Gate | Current architecture truth remains in `index.html`, runtime docs, and usage guide |
| Failure route | Return to the owning current chapter before treating history as present-day contract |

## [0.7.0] - 2026-05-23

### Changed (JS)
- **Four-layer runtime split** — `validate()` is the normative gate, `ObjectTree.from()` is the external boundary entry, and `new ObjectTree()` is the internal Proxy cursor over already-legal data.
- **Promoted v0.7.0 to mainline** — `js/schema2object.mjs` now contains the v0.7.0 implementation. Experimental side files for v0.7.1/v0.7.2 were removed; git history preserves them.
- **Simplified JS surface** — the `js/` implementation directory now keeps a single canonical `schema2object.mjs` file.

### Tests (JS)
- `node js/tests/validate.mjs`: 34/34
- `node js/tests/draft07_suite.mjs`: 922/922

## [0.6.1] - 2026-04-28

### Fixed (JS)
- **Cross-document defaults** — `_applyDefaults` now forwards the resolved sub-loader through cross-`$ref` recursion so multi-document defaults materialize with the correct resolver context.

### Changed (Docs)
- **Relocated JS docs** — moved `schema2object-api.json` and `schema2object-usage.md` under the JS documentation surface at the time of the release.
- **Updated usage notes** — clarified lazy reads, construction-time validation, `$withDefaults()` materialization, and cross-`$ref` defaults behavior.

## [0.6.0] - 2026-04-19

### Changed (JS)
- **Pointer/proxy/lazy performance baseline** — optimized the v0.5.x runtime model around direct schema lookup, cache lookup, default lookup, and lazy cursor access.
- **Loader reuse** — added `WeakMap<schema, Loader>` reuse for `resolver = null`, reducing repeated constructor setup cost.
- **Default value cache** — cached default lookup by `(loader, schema, key)` with a sentinel for missing defaults.
- **Tree operation extraction** — extracted `_treeGet`, `_treeSet`, and `_treeDescriptor` with inline cache-hit fast paths.

### Internal (JS)
- Added direct `charCodeAt` pass-through checks and `_isSafeCacheLoader` guard for cache-safe loader reuse.
- Cleaned old runtime version files into backup storage during the release branch preparation.

## [0.5.2] - 2026-04-15

### Changed (JS)
- **Observer Schema Context** — `ObjectTree._observer` hook signature extended with 5th parameter: `fn(op, path, key, val, schema)`. The `schema` parameter provides the schema node for the accessed property, enabling:
  - Filter properties by `x-observable` or other extensions
  - Format-aware serialization based on `schema.format`
  - Type-aware logging with `schema.type` and `schema.description`
  - Reactive bindings with full schema metadata in event payload
- **Backward compatible** — existing observer hooks with 4-parameter signature continue to work (5th parameter ignored if unused)

### Added (JS)
- **Observer examples** — `js/examples/observer_schema_context.mjs` demonstrates schema-context observation from `js/examples/schema.json`.

### Tests (JS)
- All existing tests pass: 922/922 Draft-07 + 19/19 validate tests

## [0.5.1] - 2026-04-15

### Added (JS)
- **Dot Key Support** — nested property paths via dot notation syntax. `obj["a.b.c"] = value` automatically creates intermediate objects and validates against nested schema. Simplifies deep property updates without manual object traversal.
  - Auto-creates intermediate objects when undefined/null
  - Full schema validation on final property
  - Works with existing observer hooks
- **`ObjectTree._observer`** — static hook point for property access observation. Set to `fn(op, path, key, val)` to observe all get/set across every ObjectTree instance. `null` disables (zero cost). Structural equivalent of Objective-C KVO — every schema-driven object is natively observable without wrapper or Proxy layer.
  - `get` emits after value resolution (primitive, object wrap, array cursor)
  - `set` emits after validation and write
  - Cached object/array re-access does not re-emit (cache hit returns before hook)

### Tests (JS)
- 5 new dot key support tests in `test_dotkey.mjs`: basic nested set, multiple paths, overwrite, type validation, auto-create intermediate
- 6 observer tests in `validate.mjs`: get primitive, get object, get array, set, null-disable, cached-no-re-emit
- Total: 922/922 Draft-07 (100%)

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
- **Historical Rust codegen** — generated static `.rs` file: runtime source + typed structs from JSON Schema. This is no longer an active repository runtime.
  - Full draft-07 suite (922/922) + API tests (44) at the time of that release.
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

- Historical three-language implementation: JS, Python, Rust
- JSON Schema Draft-07 as object class definition
- Historical API posture: `ObjectTree` construction validates; mutation validates; `to_dict()` projects schema fields only
- Draft-07 combinators as methods: `oneOf`, `anyOf`, `allOf`, `notOf`, `ifThen`, `project`, `contains`, `withDefaults`
