# JSON Types

JSON Schema's `type` keyword constrains which of JSON's six primitive types an instance may be.

---

## Chapter Role

| Field | Value |
|---|---|
| Input artifact | Draft-07 instance value and `type` keyword |
| Output artifact | Legal primitive type classification |
| Owner | Draft-07 schema contract |
| Gate | Type notes match `docs/draft-07/type.json` behavior |
| Failure route | Return to `keywords.md` for keyword-level implementation reference |

---

## The Six Types

| Type | JSON form | Description |
|---|---|---|
| `string` | `"hello"` | A Unicode text value |
| `number` | `1`, `1.5`, `-3e10` | Any numeric value, integer or float |
| `integer` | `1`, `42`, `1.0` | A number with no fractional part |
| `boolean` | `true`, `false` | A logical truth value |
| `array` | `[1, "a", null]` | An ordered sequence of values |
| `object` | `{"key": value}` | An unordered map of string keys to values |
| `null` | `null` | A null value |

---

## The `type` Keyword

`type` accepts a single string or an array of strings.

Single string form:

```json
{ "type": "string" }
```

Array form:

```json
{ "type": ["string", "null"] }
```

In the array form, the instance is valid if it matches **any** of the listed types.

Omitting `type` entirely means any JSON type is accepted.

---

## integer

`integer` is not a separate syntactic form in JSON. JSON has no integer literal — all numbers share the same syntax. The schema distinguishes integers by rule: a number is an integer if its fractional part is zero.

`1.0` is valid for `{"type": "integer"}`. Its fractional part is zero.

`1.1` is not valid for `{"type": "integer"}`. Its fractional part is non-zero.

```json
{ "type": "integer" }
```

Valid instances: `1`, `42`, `-7`, `1.0`

Invalid instances: `1.1`, `0.5`, `3.14`

---

## number

`number` accepts any numeric value — integers and floats alike.

```json
{ "type": "number" }
```

Valid instances: `1`, `1.5`, `-3`, `2e10`, `1.0`

---

## boolean

`boolean` is a distinct type. It is not a subtype of `number`. `true` and `false` are not aliases for `1` and `0`.

When `type` is `"integer"` or `"number"`, the values `true` and `false` are rejected.

```json
{ "type": "number" }
```

Invalid instances: `true`, `false`

---

## null

`null` is a distinct type. It is not the absence of a value. It is a value whose type is `null`.

```json
{ "type": "null" }
```

Valid instance: `null`

Invalid instances: `0`, `false`, `""`, `[]`

A field with value `null` is present. It satisfies a `required` constraint.

---

## Array Form of `type`

The array form allows a value to satisfy any one of multiple types. This is the standard pattern for an optional field that may also be `null`:

```json
{ "type": ["string", "null"] }
```

Valid instances: `"hello"`, `null`

Invalid instances: `1`, `true`, `[]`

The array may list any combination of the six type names. Order does not affect validation.
