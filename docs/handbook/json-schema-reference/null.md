# Null
Source: https://json-schema.org/understanding-json-schema/reference/null

## Appendix Role

This file is upstream JSON Schema reference material. It does not own `schema2object` runtime behavior; use `../draft-07/` and `../runtime/` for local gates.

The `null` type has only one acceptable value: `null`.

```json
{ "type": "null" }
```

Valid: `null`
Invalid: `false`, `0`, `""`, absent value

`null` is not equivalent to an absent property. A property with value `null` IS present and satisfies `required`. See object `required` for details.
