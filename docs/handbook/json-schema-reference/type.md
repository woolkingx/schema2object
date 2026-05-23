# Type-specific Keywords
Source: https://json-schema.org/understanding-json-schema/reference/type

## Appendix Role

This file is upstream JSON Schema reference material. It does not own `schema2object` runtime behavior; use `../draft-07/` and `../runtime/` for local gates.

The `type` keyword can take two forms:

1. **A single string** — one of: `array`, `boolean`, `integer`, `number`, `null`, `object`, `string`. The instance is valid only when it matches that type.

2. **An array of strings** — the instance is valid if it matches *any* of the listed types.

```json
{ "type": ["number", "string"] }
```

## Type-to-keyword mapping

| Type | Specific Keywords |
|---|---|
| `array` | `items`, `additionalItems`, `minItems`, `maxItems`, `uniqueItems` |
| `number` | `minimum`, `maximum`, `exclusiveMinimum`, `exclusiveMaximum`, `multipleOf` |
| `object` | `required`, `properties`, `additionalProperties`, `patternProperties`, `minProperties`, `maxProperties`, `dependencies` |
| `string` | `minLength`, `maxLength`, `pattern`, `format` |

## format

The `format` keyword conveys semantic information for values that are difficult to describe using JSON Schema alone (e.g. dates encoded as strings). By default, `format` is an annotation only and does not affect validation.

Implementations may optionally enable `format` as an assertion. When enabled, validation fails if the value does not conform to the format's specification.

### Built-in formats (Draft-07)

**Dates and times** (RFC 3339 §5.6):
- `"date-time"` — e.g. `2018-11-13T20:20:39+00:00`
- `"time"` *(new in Draft 7)* — e.g. `20:20:39+00:00`
- `"date"` *(new in Draft 7)* — e.g. `2018-11-13`

**Email:** `"email"`, `"idn-email"` *(Draft 7)*

**Hostnames:** `"hostname"`, `"idn-hostname"` *(Draft 7)*

**IP addresses:** `"ipv4"`, `"ipv6"`

**Resource identifiers:** `"uri"`, `"uri-reference"` *(Draft 6)*, `"iri"` *(Draft 7)*, `"iri-reference"` *(Draft 7)*, `"uri-template"` *(Draft 6)*, `"json-pointer"` *(Draft 6)*, `"relative-json-pointer"` *(Draft 7)*

**Regular expressions:** `"regex"` *(Draft 7)* — ECMA 262 dialect
