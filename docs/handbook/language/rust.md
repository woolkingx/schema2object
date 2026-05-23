# Rust Projection Guide

Rust is not the canonical dynamic `ObjectTree` runtime in the current `schema2object` language posture. Rust should project schema into native static shapes.

Read the local owner docs first:

- `../index.html`
- `../runtime/javascript.md`
- `../runtime/schema2object-usage.md`

For the broader methodology, see:

- https://codeberg.org/woolkingx/schema-driven-development

## Chapter Role

| Field | Value |
|---|---|
| Input artifact | JSON Schema that a Rust consumer must accept or emit |
| Output artifact | Generated or handwritten `types.rs` plus typed executor projection |
| Owner | Rust consumer project, not this repository runtime |
| Gate | Typed executor traces back to schema and does not implement a second ObjectTree runtime |
| Failure route | Return to the schema contract; do not let `types.rs` become source of truth |

## First Principle

`schema.json` is truth. `types.rs` is projection. Dynamic JSON belongs at the boundary. Core logic uses typed `Input -> Rule -> Output`.

```text
schema.json -> generated or handwritten types.rs -> typed executor -> conformance tests
```

## Rust Posture

| Schema role | Rust shape |
|---|---|
| input schema / SDU | `struct Input` |
| rule schema / PCI | `struct Rule` or enum transition declaration |
| output schema / PDU | `struct Output` |
| executor | `fn execute(input: Input, rule: Rule) -> Result<Output, Error>` |
| service boundary | serde JSON decode / encode and schema validation |

The Rust core should not walk arbitrary `serde_json::Value` as its primary model. `Value` is acceptable at ingress, egress, diagnostics, and tooling boundaries.

## Recommended Shape

```text
wire JSON
  -> validate against schema.json
  -> deserialize into generated or handwritten types.rs
  -> execute typed transition
  -> serialize output
  -> validate output contract
```

## Allowed Development Modes

| Mode | Use when | Gate |
|---|---|---|
| handwritten `types.rs` | early architecture or small schema subset | conformance tests prove schema and types agree |
| generated `types.rs` | stable schema subset exists | generated output is reproducible |
| dynamic `serde_json::Value` | boundary, plugin, migration, or debug surface | value does not own core transition truth |

## Minimum Rust Subset

Start with a small schema subset:

- `type: object`
- `properties`
- `required`
- `enum`
- `const`
- `array`
- `$ref`
- `oneOf` with discriminator
- `additionalProperties: false`

Reject or hand-review complex `if/then/else`, unconstrained `oneOf`, `patternProperties`, and open maps until the projection rule is explicit.

## Conformance Rule

Rust code must not become the schema source of truth.

```text
schema.json changes -> projection changes -> conformance tests update
types.rs changes without schema change -> suspicious until explained
```

Gate:

```text
input validates
rule applies
output validates
typed executor does not invent schema truth
```
