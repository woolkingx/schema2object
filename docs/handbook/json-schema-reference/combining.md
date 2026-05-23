# Boolean JSON Schema Combination
Source: https://json-schema.org/understanding-json-schema/reference/combining

## Appendix Role

This file is upstream JSON Schema reference material. It does not own `schema2object` runtime behavior; use `../draft-07/` and `../runtime/` for local gates.

Keywords for combining schemas using boolean algebra. Each keyword evaluates the instance against multiple subschemas simultaneously.

## allOf

Instance must be valid against **all** subschemas (AND).

```json
{ "allOf": [{ "type": "string" }, { "maxLength": 5 }] }
```

`allOf` cannot be used for OO-style inheritance. Instances must independently satisfy every subschema. See `additionalProperties` scope rule — it cannot see across `allOf` boundaries.

## anyOf

Instance must be valid against **at least one** subschema (OR).

```json
{ "anyOf": [{ "type": "string", "maxLength": 5 }, { "type": "number", "minimum": 0 }] }
```

## oneOf

Instance must be valid against **exactly one** subschema (XOR). Both matching = invalid. Zero matching = invalid.

```json
{ "oneOf": [{ "type": "number", "multipleOf": 5 }, { "type": "number", "multipleOf": 3 }] }
```

Note: `oneOf` requires verifying every subschema, which can increase processing time. Prefer `anyOf` where possible.

## not

Instance must **not** be valid against the given schema.

```json
{ "not": { "type": "string" } }
```

## Properties of Schema Composition

### Illogical schemas

Schemas that can never be satisfied are legal but will always fail:

```json
{ "allOf": [{ "type": "string" }, { "type": "number" }] }
```

### Factoring schemas

Common constraints can be factored out. These two schemas are equivalent:

```json
{ "oneOf": [{ "type": "number", "multipleOf": 5 }, { "type": "number", "multipleOf": 3 }] }
```

```json
{ "type": "number", "oneOf": [{ "multipleOf": 5 }, { "multipleOf": 3 }] }
```
