# Annotations and Comments
Source: https://json-schema.org/understanding-json-schema/reference/annotations
       https://json-schema.org/understanding-json-schema/reference/comments
       https://json-schema.org/understanding-json-schema/reference/metadata

## Appendix Role

This file is upstream JSON Schema reference material. It does not own `schema2object` runtime behavior; use `../draft-07/` and `../runtime/` for local gates.

## Annotation keywords

Annotations are not used for validation. They describe parts of a schema for human readers and tooling.

| Keyword | Purpose | Draft |
|---|---|---|
| `title` | Short human-readable name | — |
| `description` | Longer explanation | — |
| `default` | Default value — documentation only, not auto-fill | — |
| `examples` | Array of example valid values | Draft 6 |
| `readOnly` | Hint: value should not be modified | Draft 7 |
| `writeOnly` | Hint: value may be set but not read back | Draft 7 |
| `deprecated` | Hint: value is discouraged and may be removed | Draft 2019-09 |

### default

> "This value is not used to fill in missing values during the validation process."

`default` expresses that if a value is missing, it is semantically equivalent to the default value being present. It is documentation of intent, not an instruction to auto-fill. The value of `default` should validate against its containing schema, but this is not required.

## $comment

New in Draft 7. Strictly for schema author notes. Value must be a string. Implementations must not attach any meaning or behavior to it, and may strip it at any time. Invisible to applications using the schema.

```json
{
  "$comment": "Created by John Doe",
  "type": "object",
  "properties": {
    "country": { "$comment": "TODO: add enum of countries" }
  }
}
```
