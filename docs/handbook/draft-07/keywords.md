# Draft-07 Keywords — Implementation Reference

> Schema IS the class. Every keyword below is part of the class definition.
> External data is accepted through the JavaScript validation boundary before a cursor is used.
> This document is the implementation contract for the active JS runtime and projection reference for other languages.

---

## Chapter Role

| Field | Value |
|---|---|
| Input artifact | Draft-07 keyword or fixture question |
| Output artifact | Keyword legality and fixture coverage expectation |
| Owner | Active JavaScript validator behavior |
| Gate | `node js/tests/draft07_suite.mjs` covers the listed fixture files |
| Failure route | Return to `spec.json` for encoded contract or to `js/tests/draft07_suite.mjs` for proof |

---

## Type System

### `type`

Constrains the JSON type of the instance. Single string or array of strings.

Valid type names: `string`, `number`, `integer`, `boolean`, `array`, `object`, `null`

```json
{ "type": "integer" }
{ "type": ["string", "null"] }
```

**Critical rules:**
- `integer`: whole numbers only. `1.0` is valid (zero fractional part). `1.1` is not.
- `number`: integers and floats both valid.
- `boolean` is NOT a subtype of `integer` or `number`. `true`/`false` must be rejected when type is `integer` or `number`.
- `null` is a distinct type, not absence of value.
- Array form: instance is valid if it matches ANY of the listed types.

---

## Scalar Constraints

### `const`

Instance must be exactly equal to the given value. Deep equality — `{"foo": false}` ≠ `{"foo": 0}`.

```json
{ "const": 42 }
{ "const": "hello" }
{ "const": null }
```

### `enum`

Instance must equal one of the listed values. Deep equality on all types including objects and arrays.

```json
{ "enum": ["red", "green", "blue"] }
{ "enum": [1, "one", true, null] }
```

**Note:** `true` ≠ `1`, `false` ≠ `0` in enum comparison (strict type equality).

---

## Numeric Constraints

### `minimum` / `maximum`

Inclusive bounds. `x >= minimum`, `x <= maximum`.

### `exclusiveMinimum` / `exclusiveMaximum`

**Draft-07 form:** numeric values (not boolean). `x > exclusiveMinimum`, `x < exclusiveMaximum`.

```json
{ "type": "number", "minimum": 0, "exclusiveMaximum": 100 }
```

**Draft-04 difference:** in Draft-04, these were booleans paired with `minimum`/`maximum`. Draft-07 uses independent numeric values. The `docs/draft-07/` fixtures use Draft-07 form.

### `multipleOf`

Must be a positive number. Instance must be an integer multiple.

```json
{ "type": "number", "multipleOf": 0.01 }
```

Floating-point precision: use relative tolerance (e.g. `1e-9`) not exact equality.

---

## String Constraints

### `minLength` / `maxLength`

Length measured in **Unicode grapheme clusters**, not bytes or code points.

```json
{ "type": "string", "minLength": 2, "maxLength": 10 }
```

`"💩"` has length 1 (one grapheme). `"\u0041\u0301"` (á as two code points) has length 1.

### `pattern`

ECMA 262 regular expression. Match is a search (not anchored) — pattern `"p"` matches `"apple"`.

```json
{ "type": "string", "pattern": "^[0-9]{5}$" }
```

To anchor, use `^...$` explicitly.

---

## Array Constraints

### `items`

**Single schema form:** every element must match.
```json
{ "type": "array", "items": { "type": "number" } }
```

**Array of schemas form (Draft-07 tuple):** positional schemas. Index 0 schema applies to element 0, etc.
```json
{ "type": "array", "items": [{ "type": "number" }, { "type": "string" }] }
```

When `items` is an array, elements beyond the defined positions are controlled by `additionalItems`.

### `additionalItems`

Only active when `items` is an array (tuple form). Controls elements beyond the defined positions.
- `false` → no additional elements allowed
- schema → additional elements must match that schema
- absent or `true` → any additional elements allowed

### `contains`

At least one element must match the given schema. Empty array always fails.

```json
{ "type": "array", "contains": { "type": "number" } }
```

### `minItems` / `maxItems`

Constrain array length. Non-negative integers.

### `uniqueItems`

When `true`, all elements must be unique. Deep equality comparison including type (`true` ≠ `1`).

---

## Object Constraints

### `properties`

Defines schemas for named properties. A property that exists in the instance but not in `properties` is ignored by this keyword (not rejected — see `additionalProperties`).

```json
{
  "type": "object",
  "properties": {
    "name": { "type": "string" },
    "age":  { "type": "integer", "minimum": 0 }
  }
}
```

Properties not listed in `properties` are allowed by default.

### `required`

Array of property names that must be present. A property with value `null` IS present — it satisfies `required`.

```json
{ "required": ["name", "email"] }
```

### `additionalProperties`

Controls properties not matched by `properties` or `patternProperties`.
- `false` → no additional properties allowed
- schema → additional properties must match that schema
- absent or `true` → any additional properties allowed

**Scope rule:** `additionalProperties` only sees properties declared in the **same subschema**. It does not see across `allOf` boundaries.

### `patternProperties`

Maps regex patterns to schemas. A property whose name matches the pattern must satisfy the schema.

```json
{
  "patternProperties": {
    "^S_": { "type": "string" },
    "^I_": { "type": "integer" }
  }
}
```

Patterns are unanchored (match anywhere in the key name). Use `^...$` to anchor.
A property matched by `patternProperties` is excluded from `additionalProperties` checking.

### `dependencies` (Draft-07)

Two forms:

**Array form (property dependency):** if key A is present, keys B and C must also be present.
```json
{ "dependencies": { "credit_card": ["billing_address"] } }
```

**Schema form (schema dependency):** if key A is present, the whole instance must also match the given schema.
```json
{ "dependencies": { "credit_card": { "required": ["billing_address"] } } }
```

`dependentRequired` (Draft 2019-09+) is the array form split into its own keyword.

### `minProperties` / `maxProperties`

Constrain the number of properties. Non-negative integers.

### `propertyNames`

Schema applied to every property **name** (key), not value. Since keys are always strings, this is typically a string-constraint schema.

```json
{ "propertyNames": { "pattern": "^[A-Za-z_][A-Za-z0-9_]*$" } }
```

---

## Logic / Composition Keywords

These are the class's behavioral logic. They do not merge schemas — they apply schemas independently.

### `allOf`

Instance must be valid against **all** subschemas (AND).

```json
{ "allOf": [{ "type": "string" }, { "maxLength": 5 }] }
```

Does not imply inheritance. Each subschema is checked independently.

### `anyOf`

Instance must be valid against **at least one** subschema (OR).

```json
{ "anyOf": [{ "type": "string" }, { "type": "number" }] }
```

### `oneOf`

Instance must be valid against **exactly one** subschema (XOR). Both matching = invalid. Zero matching = invalid.

```json
{ "oneOf": [{ "multipleOf": 5 }, { "multipleOf": 3 }] }
```

### `not`

Instance must **not** be valid against the given schema.

```json
{ "not": { "type": "string" } }
```

---

## Conditional Keywords (new in Draft-07)

### `if` / `then` / `else`

Truth table:

| `if` matches | result |
|---|---|
| yes, `then` matches | valid |
| yes, `then` fails | invalid |
| no, `else` matches | valid |
| no, `else` fails | invalid |
| no `then`/`else` defined | valid |

```json
{
  "if":   { "properties": { "country": { "const": "US" } } },
  "then": { "properties": { "zip": { "pattern": "^[0-9]{5}$" } } },
  "else": { "properties": { "zip": { "pattern": "^[A-Z][0-9][A-Z]" } } }
}
```

`if`/`then`/`else` without each other: `then`/`else` without `if` are ignored. `if` without `then`/`else` always passes.

---

## Annotation Keywords

Not enforced by Draft-07 validation. Carried as metadata on the class definition.

| Keyword | Purpose |
|---|---|
| `title` | Human-readable name |
| `description` | Human-readable description |
| `default` | Default value for schema-aware reads and explicit materialization |
| `examples` | Example valid values |
| `$comment` | Schema author notes, invisible to applications |
| `readOnly` | Hint: field should not be written |
| `writeOnly` | Hint: field should not be read |

**`default` is the runtime exception**: it is not used by Draft-07 validation, but `schema2object` uses it for lazy schema-aware reads and `$withDefaults()` materialization. Cursor construction does not mutate raw data.

---

## Scope: Draft-07 vs Later Drafts

This project targets **Draft-07**. Keywords introduced after Draft-07 are out of scope:

| Keyword | Introduced | Status |
|---|---|---|
| `dependentRequired` | 2019-09 | out of scope (use `dependencies`) |
| `dependentSchemas` | 2019-09 | out of scope |
| `prefixItems` | 2020-12 | out of scope (use `items` array form) |
| `unevaluatedProperties` | 2019-09 | out of scope |
| `unevaluatedItems` | 2019-09 | out of scope |

The `draft-07/` test suite fixtures are the ground truth. If a keyword appears in those fixtures, it must be implemented. If it does not appear, it is out of scope.

---

## Implementation Checklist

Every keyword must pass 100% of its fixture file in `draft-07/`.

| Fixture file | Keywords covered |
|---|---|
| `type.json` | `type` |
| `const.json` | `const` |
| `enum.json` | `enum` |
| `minimum.json`, `maximum.json` | `minimum`, `maximum` |
| `exclusiveMinimum.json`, `exclusiveMaximum.json` | `exclusiveMinimum`, `exclusiveMaximum` |
| `multipleOf.json` | `multipleOf` |
| `minLength.json`, `maxLength.json` | `minLength`, `maxLength` |
| `pattern.json` | `pattern` |
| `items.json`, `additionalItems.json` | `items`, `additionalItems` |
| `contains.json` | `contains` |
| `minItems.json`, `maxItems.json` | `minItems`, `maxItems` |
| `uniqueItems.json` | `uniqueItems` |
| `properties.json` | `properties` |
| `required.json` | `required` |
| `additionalProperties.json` | `additionalProperties` |
| `patternProperties.json` | `patternProperties` |
| `dependencies.json` | `dependencies` |
| `minProperties.json`, `maxProperties.json` | `minProperties`, `maxProperties` |
| `allOf.json` | `allOf` |
| `anyOf.json` | `anyOf` |
| `oneOf.json` | `oneOf` |
| `not.json` | `not` |
| `if-then-else.json` | `if`, `then`, `else` |
| `default.json` | `default` |
