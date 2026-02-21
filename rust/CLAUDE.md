# schema2object (Rust)

## Core Philosophy

**Schema IS the object definition.** Not "value + external validation + behavior layer" — one object where data, constraints, and behavior are inseparable.

Like Python's ObjectTree: you give it a schema, it **becomes** that object. Validation is intrinsic — the object inherently knows its constraints and rejects what doesn't belong, not because an external validator checks, but because that's what the object **is**.

## Design Principles

- **struct + impl = complete object.** A single `SchemaObject` struct that does everything: holds data, understands its schema, validates on mutation, provides composition methods. No separation into "data layer / validation layer / behavior layer."
- **Validation is internal behavior, not external checking.** `obj.set("age", "thirty")` fails because the object doesn't accept it — same as Python's `obj.age = "thirty"` raising TypeError. The object defines itself through its schema.
- **Schema propagation is identity propagation.** `obj.get("user")` returns another `SchemaObject` that IS the user object defined by the sub-schema. Not a raw Value, not a wrapper — a schema-defined object.
- **Self-contained implementation.** Implements what `serde_json::Value` does for data and what `jsonschema` does for validation, but as one unified object. Dependencies (serde_json, etc.) are internal storage details, not the architecture.

## Rename: schema-value → schema2object

Previous name `schema-value` reflected a "value wrapper" mindset. The correct identity is `schema2object` — schema becomes object definition. This aligns with the Python implementation and the project's cross-language vision.

## Current State (v0.1 — to be rewritten)

Current implementation treats SchemaValue as a value wrapper:
- `Index<&str>` returns `&Value` (breaks schema propagation)
- Validation lives in separate `validate.rs` as standalone engine
- Composition methods in separate `compose.rs`
- Object has no unified identity — it's a bag of parts

### Current Architecture (reference only)

```
src/
├── lib.rs        (30L)   Re-exports
├── error.rs      (100L)  ValidationError + ErrorKind enum (25 variants)
├── value.rs      (769L)  SchemaValue struct
├── validate.rs   (831L)  Standalone Draft-07 validation engine
├── compose.rs    (534L)  Composition methods
└── defaults.rs   (112L)  Default filling

tests/
├── basic.rs, validation.rs, composition.rs, defaults.rs
├── real_schemas.rs, draft07_suite.rs
└── draft7/          (30 files — official test suite data, keep)
```

Draft-07 test suite pass rate: 98.4% (666/677).
Known gaps: float/int equivalence in const/enum (serde_json limitation, 8 cases), $ref resolution (3 cases).

## Rewrite Direction

The new `SchemaObject` should:

1. **All access returns `SchemaObject`** — `get()`, `Index`, iteration all return schema-aware objects, never raw `Value`
2. **Mutation IS validation** — `set()` doesn't "validate then write", it's the object asserting its own definition
3. **Composition methods are self-transformation** — `one_of()` returns a new `SchemaObject` bound to the matching branch; the object reshapes itself
4. **`impl` carries all behavior** — no standalone validation engine, no separate composition module; everything is method on the struct
5. **Trait implementations make it a Rust-native object** — `Index`, `IntoIterator`, `Display`, `Serialize/Deserialize`, etc.

## Commands

```bash
# From rust/ directory
cargo check                                              # Type check
cargo test                                               # Run all tests
cargo test --test draft07_suite -- --nocapture           # Draft-07 suite with details
```

## Reminders

- Draft-07 test data in `tests/draft7/` — keep as-is for rewrite validation
- Sub-schema extraction priority: `properties[key]` → `patternProperties` (regex match) → `additionalProperties`
- Boolean schemas: `true` accepts all, `false` rejects all. Empty `{}` also accepts all
