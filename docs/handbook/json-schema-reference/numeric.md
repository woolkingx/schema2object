# Numeric Types
Source: https://json-schema.org/understanding-json-schema/reference/numeric

## Appendix Role

This file is upstream JSON Schema reference material. It does not own `schema2object` runtime behavior; use `../draft-07/` and `../runtime/` for local gates.

Two numeric types: `integer` and `number`. They share the same validation keywords.

## integer

Integral numbers only. JSON does not distinguish integers from floats syntactically — `1` and `1.0` represent the same value. JSON Schema considers both integers. Numbers with a zero fractional part are valid integers; numbers with a non-zero fractional part are not.

## number

Any numeric value — integers and floating-point numbers both valid.

## multipleOf

Must be a positive number. Instance must be an integer multiple of the given value.

```json
{ "type": "number", "multipleOf": 0.01 }
```

Note: the JSON specification defines numerical precision independently of IEEE 754. Implementations should use relative tolerance (e.g. `1e-9`) for floating-point comparisons.

## minimum / maximum

Inclusive bounds. `x >= minimum`, `x <= maximum`.

## exclusiveMinimum / exclusiveMaximum

**Draft-07 form:** numeric values. `x > exclusiveMinimum`, `x < exclusiveMaximum`.

```json
{ "type": "number", "minimum": 0, "exclusiveMaximum": 100 }
```

**Draft-04 form (different):** boolean values paired with `minimum`/`maximum`. `"exclusiveMaximum": true` means the paired `maximum` is exclusive. This form is out of scope for Draft-07.
