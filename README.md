# schema2object

**JSON Schema Draft-07 is the object class definition.**

Not a validator. Not a hint. The schema *is* the class — data is the instance.

JSON came from JavaScript objects. Serialization stripped the methods.
`schema2object` restores them: bind a Draft-07 schema to a runtime object and the schema enforces itself — invalid data cannot become an instance, invalid writes cannot reach storage, and Draft-07 logic keywords become callable methods.

## Core Semantics

These are internal mechanisms of the class, not external validation steps:

- **Instantiation enforces the schema.** `ObjectTree(data, schema)` raises if `data` is not a valid instance of `schema`. This is not validation — it is what it means to instantiate a class.
- **Assignment enforces the field contract.** Writing to a field checks the field's schema before storage. The class rejects what does not fit.
- **`to_dict()` projects the schema.** Serialization returns schema-defined fields only. Extra runtime fields exist but are outside the class definition.
- **Unknown fields are permitted at runtime.** The instance can carry state beyond the schema, as any object can. It just does not appear in `to_dict()`.

## Draft-07 Logic → Methods

Draft-07 combinators are the class's behavioral logic, not query operations on external data:

| Draft-07 | JS method | Python method | Semantics |
|----------|-----------|---------------|-----------|
| `oneOf` | `oneOf()` | `one_of()` | XOR — re-bind to the single matching sub-schema |
| `anyOf` | `anyOf()` | `any_of()` | OR — re-bind to all matching sub-schemas |
| `allOf` | `allOf()` | `all_of()` | AND — re-bind to merged schema |
| `not` | `notOf()` | `not_of()` | NOT — true if instance does not match |
| `if/then/else` | `ifThen()` | `if_then()` | CASE WHEN — re-bind to the matching branch |
| `properties` | `project()` | `project()` | SELECT — keep only schema-defined fields |
| `contains` | `contains()` | `contains()` | EXISTS — true if any array element matches |
| `default` | `withDefaults()` | `with_defaults()` | fill missing fields from schema defaults |

## Implementations

| Language | Directory | Draft-07 suite |
|----------|-----------|----------------|
| JavaScript | [js/](./js/) | 922/922 |
| Python | [python/](./python/) | 922/922 |
| Rust | [rust/](./rust/) | 922/922 |

## Philosophy

See [schema-driven-development](https://github.com/woolkingx/schema-driven-development) for the full methodology.

## License

MIT License
