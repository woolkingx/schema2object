# Python Projection Guide

Python does not own an active `ObjectTree` runtime in the current `schema2object` language posture. The active dynamic runtime posture belongs to JavaScript. Python should project schema into Python-native model and validation shapes.

Read the local owner docs first:

- `../index.html`
- `../runtime/javascript.md`
- `../runtime/schema2object-usage.md`

For the broader methodology, see:

- https://codeberg.org/woolkingx/schema-driven-development

## Chapter Role

| Field | Value |
|---|---|
| Input artifact | JSON Schema that a Python consumer must accept or emit |
| Output artifact | Pydantic, dataclass, TypedDict, or validator-consumer projection |
| Owner | Python consumer project, not this repository runtime |
| Gate | Python shape traces back to schema and does not implement a second ObjectTree runtime |
| Failure route | Return to the schema contract; do not let Python models become source of truth |

## First Principle

`schema.json` is truth. Python models are projection. `dict` data belongs at the boundary. Core logic should use a clear Python-native shape.

```text
schema.json -> Pydantic / dataclass / TypedDict / validator consumer
```

## Python Posture

| Schema role | Python shape |
|---|---|
| input schema / SDU | Pydantic model, dataclass, TypedDict, or validated dict |
| rule schema / PCI | rule model, config object, or explicit transition function input |
| output schema / PDU | Pydantic model, dataclass, TypedDict, or validated dict |
| executor | function that transforms input plus rule into output |
| service boundary | JSON decode / encode and schema validation |

Avoid building a second JavaScript-style object runtime in Python. `obj.foo`, dynamic schema-bound methods, and general-purpose `ObjectTree` cursors are not the natural Python posture for this project.

## Recommended Shape

```text
wire JSON
  -> validate against schema.json
  -> parse into Pydantic / dataclass / TypedDict boundary
  -> execute explicit Python transition
  -> serialize output
  -> validate output contract
```

## Allowed Development Modes

| Mode | Use when | Gate |
|---|---|---|
| Pydantic projection | API or service boundary needs runtime validation | model fields trace back to schema |
| dataclass projection | internal typed data with low validation needs | construction follows validated boundary data |
| TypedDict projection | static typing and dict compatibility are enough | mypy/pyright shape matches schema |
| validator consumer | lightweight script or ingestion step | schema remains the contract |

## Conformance Rule

Python code must not become the schema source of truth.

```text
schema.json changes -> Python projection changes -> examples/tests update
Python model changes without schema change -> suspicious until explained
```

Gate:

```text
input validates
rule applies
output validates
Python projection does not invent schema truth
```
