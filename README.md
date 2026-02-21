# schema2object

JSON Schema Draft-07 as object definition.
Structure maps to attributes/properties, logic maps to methods.

JSON came from JavaScript objects. Serialization stripped the methods.
`schema2object` restores them — across languages, with the same API.

## Implementations

| Language | Directory | Status |
|----------|-----------|--------|
| Python   | [python/](./python/) | stable |
| Rust     | [rust/](./rust/) | stable |
| JavaScript | [js/](./js/) | planned |

## Core API (all implementations)

Each Draft-07 logical operator maps to a method:

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
