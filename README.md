# schema2object

JSON Schema Draft-07 **is the object class definition**.
Structure maps to properties, constraints are enforced on mutation, and Draft-07 logic maps to methods.

JSON came from JavaScript objects. Serialization stripped the methods.
`schema2object` restores them — across languages, with the same core semantics.

## Implementations

| Language | Directory | Status |
|----------|-----------|--------|
| Python   | [python/](./python/) | stable |
| Rust     | [rust/](./rust/) | stable |
| JavaScript | [js/](./js/) | stable |

## Core Semantics (all implementations)

- **Schema defines the class**; data is the instance
- **Mutation validates** (invalid writes fail)
- **Unknown fields may exist at runtime** (language-native behavior)
- **`to_dict()` exports schema-defined fields only**
- **Schema extensions live in `x-*`** (e.g. `x-docs`, `x-tests`)

## Draft-07 Logic → Methods

| Draft-07 | Method | Semantics |
|----------|--------|-----------|
| `oneOf` | `one_of()` | XOR — exactly one branch |
| `anyOf` | `any_of()` | OR — all matching branches |
| `allOf` | `all_of()` | AND — merged schema |
| `not` | `not_of()` | NOT — exclusion check |
| `if/then/else` | `if_then()` | CASE WHEN — conditional branch |
| `properties` | `project()` | SELECT — schema-defined fields only |
| `contains` | `contains()` | EXISTS — array element check |

## Philosophy

See [schema-driven-development](https://github.com/woolkingx/schema-driven-development) for the full methodology.

## License

MIT License
