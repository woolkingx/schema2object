# Conditionals — JSON Schema Draft-07 Spec Reference

> Covers `if`/`then`/`else` and `dependencies`. Pure specification — no implementation details.

---

## Chapter Role

| Field | Value |
|---|---|
| Input artifact | Conditional schema or dependency schema |
| Output artifact | Branch/dependency semantics independent of runtime projection |
| Owner | Draft-07 schema contract |
| Gate | Conditional behavior matches `if-then-else.json` and `dependencies.json` fixtures |
| Failure route | Return to `keywords.md` for fixture map or `composition.md` for boolean implication |

---

## if / then / else

New in Draft-07. These three keywords apply sub-schemas conditionally based on whether the instance satisfies `if`.

### How Each Keyword Behaves

**`if`** — evaluates a sub-schema against the instance. Produces a boolean result: matches or does not match. Does not constrain the instance by itself.

**`then`** — applied to the instance only when `if` matches. If `then` is absent, it is treated as passing (no constraint added).

**`else`** — applied to the instance only when `if` does not match. If `else` is absent, it is treated as passing (no constraint added).

`then` or `else` present without `if` — ignored. They have no effect.

### Truth Table

| `if` result | branch applied | branch result | instance valid? |
|---|---|---|---|
| matches | `then` | matches | yes |
| matches | `then` | fails | no |
| matches | `then` absent | — | yes |
| does not match | `else` | matches | yes |
| does not match | `else` | fails | no |
| does not match | `else` absent | — | yes |

### Missing Properties and `if`

`if` evaluates the sub-schema as written. It does not require any property to exist. If the sub-schema references a property that is absent from the instance, the sub-schema still evaluates — it may pass or fail depending on what it asserts.

A sub-schema of `{ "properties": { "country": { "const": "US" } } }` does not assert `country` is required. If `country` is absent, `properties` places no constraint on it, so `if` matches. To require the property's presence as part of the condition, include `required` inside the `if` sub-schema.

### Example

```json
{
  "if":   { "properties": { "country": { "const": "US" } }, "required": ["country"] },
  "then": { "properties": { "postal_code": { "pattern": "^[0-9]{5}(-[0-9]{4})?$" } }, "required": ["postal_code"] },
  "else": { "properties": { "postal_code": { "pattern": "^[A-Z][0-9][A-Z] [0-9][A-Z][0-9]$" } }, "required": ["postal_code"] }
}
```

- Instance with `"country": "US"` — `if` matches, `then` is applied.
- Instance with `"country": "CA"` — `if` does not match, `else` is applied.
- Instance without `country` — `if` does not match (because `required` inside `if` fails), `else` is applied.

---

## dependencies

`dependencies` applies to object instances. It activates when a named property is present. It has two distinct forms.

### Array Form (Property Dependency)

If property A is present in the instance, all properties listed in the array must also be present.

```json
{
  "dependencies": {
    "credit_card": ["billing_address", "billing_name"]
  }
}
```

- Instance with `credit_card` — `billing_address` and `billing_name` must also be present.
- Instance without `credit_card` — no constraint is added.

The array lists the names of required co-properties. It does not apply any schema to their values.

### Schema Form (Schema Dependency)

If property A is present in the instance, the entire instance must also satisfy the given sub-schema.

```json
{
  "dependencies": {
    "credit_card": {
      "required": ["billing_address"],
      "properties": {
        "billing_address": { "type": "string" }
      }
    }
  }
}
```

- Instance with `credit_card` — the whole instance must match the sub-schema (here: `billing_address` must be present and must be a string).
- Instance without `credit_card` — the sub-schema is not applied.

The sub-schema is validated against the whole instance, not just the triggering property.

### Both Forms in One Schema

A single `dependencies` object may mix array-form and schema-form entries:

```json
{
  "dependencies": {
    "credit_card": ["billing_address"],
    "subscription": { "required": ["email"], "properties": { "email": { "type": "string" } } }
  }
}
```

Each entry is evaluated independently when its trigger property is present.

### Directionality

Dependencies are not bidirectional. `"credit_card": ["billing_address"]` means: if `credit_card` is present, `billing_address` must be present. It does not mean: if `billing_address` is present, `credit_card` must be present. To enforce the reverse, add a separate entry.

### Out of Scope

`dependentRequired` (Draft 2019-09+) is the array form extracted into its own keyword. It is out of scope for Draft-07.

`dependentSchemas` (Draft 2019-09+) is the schema form extracted into its own keyword. It is out of scope for Draft-07.

In Draft-07, both forms live under `dependencies`.

---

## Implication (Pre-Draft-07 Pattern)

Before Draft-07 introduced `if`/`then`/`else`, conditional logic was expressed through boolean algebra using `anyOf` and `not`.

The concept is **implication**: `A → B` (A implies B) means: if A is true, then B must also be true. This is logically equivalent to `¬A ∨ B` — either A is false, or B is true.

In JSON Schema, this translates to:

```json
{
  "anyOf": [
    { "not": { <condition A> } },
    { <consequence B> }
  ]
}
```

If A does not match (the `not` branch passes), the overall schema passes. If A matches, then B must also match.

### Example

The equivalent of `if country = "US" then postal_code must match US format` expressed as implication:

```json
{
  "anyOf": [
    {
      "not": {
        "properties": { "country": { "const": "US" } },
        "required": ["country"]
      }
    },
    {
      "properties": { "postal_code": { "pattern": "^[0-9]{5}(-[0-9]{4})?$" } },
      "required": ["postal_code"]
    }
  ]
}
```

### Readability

This pattern becomes difficult to read when the condition is complex. The spec recommends using `$defs` with descriptive names to clarify intent:

```json
{
  "$defs": {
    "not-a-us-address": {
      "not": {
        "properties": { "country": { "const": "US" } },
        "required": ["country"]
      }
    },
    "us-postal-code": {
      "properties": { "postal_code": { "pattern": "^[0-9]{5}(-[0-9]{4})?$" } },
      "required": ["postal_code"]
    }
  },
  "anyOf": [
    { "$ref": "#/$defs/not-a-us-address" },
    { "$ref": "#/$defs/us-postal-code" }
  ]
}
```

In Draft-07, prefer `if`/`then`/`else` directly. This pattern is documented for understanding schemas written against earlier drafts.
