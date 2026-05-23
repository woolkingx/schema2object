# Array
Source: https://json-schema.org/understanding-json-schema/reference/array

## Appendix Role

This file is upstream JSON Schema reference material. It does not own `schema2object` runtime behavior; use `../draft-07/` and `../runtime/` for local gates.

Arrays are used for ordered elements. Each element may be of a different type.

Two usage modes:
- **List validation** — arbitrary length, each item matches the same schema
- **Tuple validation** — fixed length, each position has its own schema

## items (list validation)

Single schema form: every element must match.

```json
{ "type": "array", "items": { "type": "number" } }
```

Empty array is always valid.

## items (tuple validation — Draft-07)

In Draft 4–2019-09, `items` as an array of schemas performs tuple validation. Position `i` applies schema `items[i]`.

```json
{
  "type": "array",
  "items": [{ "type": "number" }, { "type": "string" }, { "enum": ["Street", "Avenue"] }]
}
```

Note: In Draft 2020-12, this was renamed to `prefixItems`. In Draft-07, use the array form of `items`.

## additionalItems

Only active when `items` is an array (tuple form). Controls elements beyond the defined positions.
- `false` — no additional elements allowed
- schema — additional elements must match that schema
- absent or `true` — any additional elements allowed

In Draft 6–2019-09, `additionalItems` is ignored if there is no tuple-form `items` in the same schema.

## contains

New in Draft 6. At least one element must match the given schema. Does not require all elements to match. Empty array always fails.

```json
{ "type": "array", "contains": { "type": "number" } }
```

## minContains / maxContains

New in Draft 2019-09. Out of scope for Draft-07.

## minItems / maxItems

Constrain array length. Non-negative integers.

## uniqueItems

When `true`, all elements must be unique. Deep equality including type (`true` ≠ `1`).

## unevaluatedItems

New in Draft 2019-09. Out of scope for Draft-07.
