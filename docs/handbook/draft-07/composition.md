# Schema Composition Keywords

> Spec reference for `allOf`, `anyOf`, `oneOf`, `not`. No implementation details.

---

## Chapter Role

| Field | Value |
|---|---|
| Input artifact | A schema using boolean composition keywords |
| Output artifact | Correct composition semantics and boundary rules |
| Owner | Draft-07 schema contract |
| Gate | Composition behavior matches `allOf`, `anyOf`, `oneOf`, and `not` fixtures |
| Failure route | Return to `keywords.md` for fixture map or `structure.md` for ObjectTree binding |

---

## Overview

Composition keywords apply boolean algebra to sub-schemas. Each sub-schema is evaluated independently against the full instance. These keywords do not merge schemas, inherit fields, or share keyword scope across boundaries.

| Keyword | Logic | Cardinality | Passes when |
|---------|-------|-------------|-------------|
| `allOf` | AND   | array of schemas | instance satisfies ALL sub-schemas |
| `anyOf` | OR    | array of schemas | instance satisfies AT LEAST ONE sub-schema |
| `oneOf` | XOR   | array of schemas | instance satisfies EXACTLY ONE sub-schema |
| `not`   | NOT   | single schema    | instance does NOT satisfy the sub-schema |

---

## `allOf`

Instance must satisfy every sub-schema in the array. The sub-schemas are independent — each checks the full instance from scratch.

```json
{
  "allOf": [
    { "type": "string" },
    { "minLength": 2 },
    { "maxLength": 10 }
  ]
}
```

An instance that fails any one sub-schema is invalid. An instance that satisfies all is valid.

### allOf is not inheritance

`allOf` does not merge sub-schemas into one combined schema. It evaluates each sub-schema separately. This has a critical consequence for `additionalProperties`.

`additionalProperties` only sees properties declared in its own sub-schema. It cannot see across `allOf` boundaries:

```json
{
  "allOf": [
    {
      "properties": { "name": { "type": "string" } },
      "additionalProperties": false
    },
    {
      "properties": { "age": { "type": "integer" } }
    }
  ]
}
```

This schema rejects any instance that has `age`. The first sub-schema declares `additionalProperties: false` and knows only about `name`. It does not see `age`, which is declared in the second sub-schema. From the first sub-schema's perspective, `age` is an additional property — and is rejected.

You cannot extend a closed schema (`additionalProperties: false`) using `allOf`. The closed sub-schema has no knowledge of properties added in sibling sub-schemas.

---

## `anyOf`

Instance must satisfy at least one sub-schema. Satisfying more than one is also valid.

```json
{
  "anyOf": [
    { "type": "string" },
    { "type": "number" }
  ]
}
```

Valid: `"hello"` (matches first), `42` (matches second), `3.14` (matches second).
Invalid: `true`, `null`, `[]`, `{}`.

---

## `oneOf`

Instance must satisfy exactly one sub-schema. Zero matches is invalid. Two or more matches is also invalid.

```json
{
  "oneOf": [
    { "multipleOf": 5 },
    { "multipleOf": 3 }
  ]
}
```

| Instance | Multiple of 5 | Multiple of 3 | Result |
|----------|:---:|:---:|--------|
| `10`  | yes | no  | valid (exactly one) |
| `9`   | no  | yes | valid (exactly one) |
| `15`  | yes | yes | **invalid** (both match) |
| `7`   | no  | no  | **invalid** (none match) |

`oneOf` enforces mutual exclusion between sub-schemas at the instance level.

---

## `not`

Instance must not satisfy the given sub-schema. Takes a single schema, not an array.

```json
{ "not": { "type": "string" } }
```

Valid: any non-string. Invalid: any string.

```json
{ "not": { "enum": ["forbidden", "banned"] } }
```

Valid: any value not in the enum. Invalid: `"forbidden"` or `"banned"`.

`not: {}` rejects everything. `not: false` accepts everything (negation of the always-failing schema).

---

## Factoring

A constraint at the outer schema applies to every branch of a composition keyword. These two schemas are equivalent:

```json
{ "type": "number", "oneOf": [{ "multipleOf": 5 }, { "multipleOf": 3 }] }
```

```json
{
  "oneOf": [
    { "type": "number", "multipleOf": 5 },
    { "type": "number", "multipleOf": 3 }
  ]
}
```

In both forms the instance must be a number before the `oneOf` branches are evaluated. The first form expresses the common constraint once at the outer level. The second duplicates it inside each branch. The validation result is identical.

Factoring applies to all composition keywords. Any keyword at the outer schema level is an additional independent constraint the instance must satisfy, regardless of the composition result.

---

## Combining Composition Keywords

Composition keywords combine like any other constraints — all must be satisfied simultaneously.

```json
{
  "allOf": [
    { "type": "object" }
  ],
  "anyOf": [
    { "required": ["name"] },
    { "required": ["id"] }
  ],
  "not": {
    "required": ["forbidden_field"]
  }
}
```

The instance must be an object (from `allOf`), must have `name` or `id` (from `anyOf`), and must not have `forbidden_field` (from `not`). All three conditions apply independently.

---

## The `additionalProperties` Boundary Rule

This rule applies across all composition keywords, not only `allOf`.

Each sub-schema in a composition is evaluated as a self-contained unit. `additionalProperties` computes "additional" relative to `properties` and `patternProperties` declared **in the same schema object**. It has no visibility into sibling sub-schemas, parent schemas, or other branches of a composition.

Correct mental model: each sub-schema object is a closed namespace. `additionalProperties` can only reference what is declared inside that same object.

```json
{
  "oneOf": [
    {
      "properties": { "type": { "const": "circle" }, "radius": {} },
      "additionalProperties": false
    },
    {
      "properties": { "type": { "const": "rect" }, "width": {}, "height": {} },
      "additionalProperties": false
    }
  ]
}
```

Each branch closes itself. A `circle` instance with `radius` is valid against the first branch and invalid against the second (because `radius` is additional from the second branch's perspective). This is the intended pattern — `additionalProperties: false` inside `oneOf` branches successfully discriminates between shapes.

The mistake is placing `additionalProperties: false` in a parent schema and expecting `allOf` sub-schemas to contribute their properties to its scope. They do not.

---

## Truth Table Summary

| Keyword | Sub-schemas that must pass | Sub-schemas that must fail | Behavior on empty array |
|---------|---------------------------|---------------------------|------------------------|
| `allOf` | all                        | none                       | valid (vacuously true)  |
| `anyOf` | at least one               | —                          | invalid                 |
| `oneOf` | exactly one                | all others                 | invalid                 |
| `not`   | — (single schema)          | the given schema           | n/a                     |
