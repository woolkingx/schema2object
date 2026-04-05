# schema2object

**JSON Schema as header file. Like C's `.h`, like TypeScript's type system — but at runtime, cross-language, and removable.**

| | TypeScript | C `.h` | schema2object |
|--|-----------|--------|---------------|
| Type definition | `.ts` file, erased at compile | `.h` file, compile-time only | JSON Schema, lives at runtime |
| Validation | compile-time only, runtime naked | compile-time only | construction IS validation |
| Cross-language | JS only | C/C++ only | JSON Schema = language-agnostic |
| Removable | no, build step required | no, compilation required | yes — mature code runs without it, zero overhead |
| Self-describing | no | no | schema is machine-readable documentation |

Not a validator. Not a hint. The schema *is* the class — data is the instance.

JSON came from JavaScript objects. Serialization stripped the methods.
`schema2object` restores them: bind a Draft-07 schema to a runtime object and the schema enforces itself — invalid data cannot become an instance, invalid writes cannot reach storage, and Draft-07 logic keywords become callable methods.

**Use it as scaffolding**: develop with full schema validation, then remove it when mature — the contracts are already baked into your code. [bashjsast](https://codeberg.org/woolkingx/bashjsast) demonstrates this: built with schema2object, shipped as raw dict. 840x faster, zero regression.

## Core Semantics

These are internal mechanisms of the class, not external validation steps:

- **Instantiation enforces the schema.** `ObjectTree(data, schema)` raises if `data` is not a valid instance of `schema`. This is not validation — it is what it means to instantiate a class.
- **Assignment enforces the field contract.** Writing to a field checks the field's schema before storage. The class rejects what does not fit.
- **`to_dict()` projects the schema.** Serialization returns schema-defined fields only. Extra runtime fields exist but are outside the class definition.
- **Unknown fields are permitted at runtime.** The instance can carry state beyond the schema, as any object can. It just does not appear in `to_dict()`.
- **Schema defaults are the initial value.** A field missing from raw data reads its schema `default` transparently — `tree.port` returns `8080` without any explicit `$withDefaults()` call. `Object.keys(tree)` and `in` include default-only fields. `$withDefaults()` materializes defaults into raw data for serialization to schema-unaware systems.

## Draft-07 Logic → Methods

Draft-07 combinators are the class's behavioral logic, not query operations on external data:

| Draft-07 | JS method | Python method | Semantics |
|----------|-----------|---------------|-----------|
| `oneOf` | `$oneOf()` | `one_of()` | XOR — re-bind to the single matching sub-schema |
| `anyOf` | `$anyOf()` | `any_of()` | OR — re-bind to all matching sub-schemas |
| `allOf` | `$allOf()` | `all_of()` | AND — re-bind to merged schema |
| `not` | `$notOf()` | `not_of()` | NOT — true if instance does not match |
| `if/then/else` | `$ifThen()` | `if_then()` | CASE WHEN — re-bind to the matching branch |
| `properties` | `$project()` | `project()` | SELECT — keep only schema-defined fields |
| `contains` | `$contains()` | `contains()` | EXISTS — true if any array element matches |
| `default` | `$withDefaults()` | `with_defaults()` | materialize schema defaults into raw data |

> **JS naming convention**: All ObjectTree API methods use `$` prefix (`$value`, `$schema`, `$toDict()`, etc.). Data properties have no prefix (`tree.name`, `tree.email`). This avoids collision when schema property names match API method names.

## Implementations

| Language | Directory | Draft-07 suite |
|----------|-----------|----------------|
| JavaScript | [js/](./js/) | 922/922 |
| Python | [python/](./python/) | 922/922 |
| Rust | [rust/](./rust/) | 922/922 |

## Philosophy

See [schema-driven-development](https://codeberg.org/woolkingx/schema-driven-development) for the full methodology.

## License

MIT License
