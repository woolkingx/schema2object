# schema2object Language Quick Start

JSON Schema is an Object DSL. Write a class definition in it. Use it directly as an object.

Repository posture: JavaScript is the only active runtime owned here. Other language sections are projection guidance for carrying the same schema truth into native tools.

## Chapter Role

| Field | Value |
|---|---|
| Input artifact | Schema truth that needs to move into another ecosystem |
| Output artifact | Projection route, not runtime package instructions |
| Owner | `docs/handbook/language/` |
| Gate | Projection does not invent schema truth or claim active runtime ownership |
| Failure route | Return to `../runtime/schema2object-usage.md` for JS usage, or to the language-specific projection chapter for native shape details |

---

## Step 1: Write the Schema

One file. State + behavior logic + test intent + semantic context — all in one DSL.

**schemas/user.schema.json**:
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "User",
  "type": "object",

  "properties": {
    "email": {"type": "string", "format": "email"},
    "age":   {"type": "integer", "minimum": 0},
    "roles": {
      "type": "array",
      "items": {"enum": ["admin", "user", "guest"]},
      "minItems": 1,
      "uniqueItems": true
    }
  },
  "required": ["email", "roles"],

  "x-methods": {
    "is_adult": {
      "properties": {"age": {"minimum": 18}}
    },
    "is_admin": {
      "properties": {"roles": {"contains": {"const": "admin"}}}
    }
  },

  "x-tests": {
    "is_adult": ["min_boundary", "below_min", "null", "valid_normal"],
    "is_admin": ["valid_normal", "valid_edge", "missing"]
  },

  "x-docs": {
    "intent":   "Core user entity for authentication and authorization.",
    "category": "auth",
    "notes":    "Email is immutable after creation."
  }
}
```

---

## Step 2: Project it into the Native Language Shape

`schema2object` does not require every language to implement `ObjectTree`.

```text
same schema truth
different language-native projection
```

| Language | Projection |
|---|---|
| JS/MJS | Active dynamic schema runtime |
| Rust | Projection note: `schema.json -> types.rs -> typed executor` |
| Python | Projection note: `schema.json -> Pydantic / dataclass / TypedDict / validator consumer` |
| TypeScript | Projection note: `schema.json -> generated types + Ajv validator` |

Rust, Python, and TypeScript are guide-only projection surfaces:

- `rust.md`
- `python.md`
- `typescript-ajv.md`

This repository owns the language references:

- `../index.html`
- `../runtime/javascript.md`
- `../runtime/schema2object-usage.md`

The broader methodology lives in Schema-Driven Development:

- https://codeberg.org/woolkingx/schema-driven-development

---

## Step 3: CI/CD — Derive Everything Else

Once the schema is complete, CI/CD derives types, docs, and runs contract tests.

**.github/workflows/schema-validation.yml**:
```yaml
name: Schema Validation

on:
  pull_request:
    paths: ["schemas/**"]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Validate schema syntax
        run: |
          npm install -g ajv-cli
          ajv compile -s schemas/*.json

      - name: Validate examples
        run: |
          ajv validate -s schemas/user.schema.json -d examples/user.valid.json

      - name: Run tests
        run: |
          npm test
```

Adapt those steps to the owning project. In this repository, the JavaScript canonical runtime gate is:

```bash
node js/tests/draft07_suite.mjs
```

---

## What Each Layer Does

```
schemas/user.schema.json    →  class definition (single source of truth)
        ↓
  properties                →  state (fields + constraints)
  x-methods                 →  behavior specs
  x-tests                   →  test intent
  x-docs                    →  semantic context

        ↓ derived by toolchain + LLM

  JS runtime                →  active dynamic object surface
  Rust projection           →  types.rs + typed executor
  Python projection         →  Pydantic / dataclass / TypedDict
  CI/CD                     →  validators, docs, contract tests
```

---

## Next Steps

- `../index.html` — handbook overview and language posture
- `../runtime/javascript.md` — JavaScript runtime surface
- `../runtime/schema2object-usage.md` — JavaScript runtime guide
- `rust.md` — Rust Projection Guide
- `python.md` — Python Projection Guide
- https://codeberg.org/woolkingx/schema-driven-development — broader SDD methodology
