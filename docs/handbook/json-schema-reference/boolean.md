# Boolean
Source: https://json-schema.org/understanding-json-schema/reference/boolean

## Appendix Role

This file is upstream JSON Schema reference material. It does not own `schema2object` runtime behavior; use `../draft-07/` and `../runtime/` for local gates.

The `boolean` type matches only two values: `true` and `false`.

Values that *evaluate* to true or false — such as `1`, `0`, `""`, `null` — are **not** accepted. Boolean is not a subtype of integer or number.

```json
{ "type": "boolean" }
```

Valid: `true`, `false`
Invalid: `1`, `0`, `"true"`, `null`
