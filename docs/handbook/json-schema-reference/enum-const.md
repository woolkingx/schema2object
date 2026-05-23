# Enumerated and Constant Values
Source: https://json-schema.org/understanding-json-schema/reference/generic
       https://json-schema.org/understanding-json-schema/reference/enum
       https://json-schema.org/understanding-json-schema/reference/const

## Appendix Role

This file is upstream JSON Schema reference material. It does not own `schema2object` runtime behavior; use `../draft-07/` and `../runtime/` for local gates.

## enum

The `enum` keyword restricts a value to a fixed set of values. It must be an array with at least one element, where each element is unique.

```json
{ "enum": ["red", "amber", "green"] }
```

Enums can mix types, including `null`:

```json
{ "enum": ["red", "amber", "green", null, 42] }
```

`enum` can be used without a `type` keyword — the value is valid if it matches any entry regardless of type.

## const

New in Draft 6. Restricts a value to a single fixed value.

```json
{ "properties": { "country": { "const": "United States of America" } } }
```

`const` is equivalent to an `enum` with a single entry. Deep equality applies — `{"foo": false}` ≠ `{"foo": 0}`.
