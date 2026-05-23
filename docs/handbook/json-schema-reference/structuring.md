# Modular JSON Schema Combination (Structuring)
Source: https://json-schema.org/understanding-json-schema/structuring

## Appendix Role

This file is upstream JSON Schema reference material. It does not own `schema2object` runtime behavior; use `../draft-07/` and `../runtime/` for local gates.

## Schema Identification

Schema documents are identified by non-relative URIs. A schema does not require an identifier, but needs one to be referenced from another schema.

### URI terminology

- **URI** / non-relative URI — full URI with scheme (`https://`), may include fragment (`#foo`)
- **relative reference** — partial URI without scheme, may include fragment
- **URI-reference** — either of the above
- **absolute URI** — full URI with scheme, no fragment

Implementations generally do not fetch schemas over the network. Instead, schemas are loaded into an internal database and retrieved by URI identifier.

### Retrieval URI

The URI used to fetch a schema. Anonymous schemas (passed directly to an implementation) have no retrieval URI.

### Base URI

The base URI determines how relative references within the schema are resolved. Determined by:
1. `$id` keyword (if present) — resolved against the retrieval URI
2. Otherwise, the retrieval URI itself

## $id

Sets the base URI of the schema. Value is a URI-reference without a fragment.

In Draft 4, `$id` is just `id` (no dollar sign).

Recommended: always use an absolute URI to avoid resolution ambiguity with anonymous schemas.

```json
{ "$id": "https://example.com/schemas/address" }
```

## JSON Pointer

A JSON Pointer (RFC 6901) identifies a subschema by path:

```
https://example.com/schemas/address#/properties/street_address
```

`/properties/street_address` means: find key `properties`, then within that find key `street_address`.

## $anchor

A named anchor identifies a subschema without a path. Declared with the `$anchor` keyword.

```
https://example.com/schemas/address#street_address
```

Anchors must start with a letter, followed by letters, digits, `-`, `_`, `:`, or `.`.

In Draft 4-7, anchors are declared the same way but use `id` instead of `$id`.

## $ref

References another schema. Value is a URI-reference resolved against the schema's base URI. The referenced schema is then applied to the instance.

```json
{
  "properties": {
    "shipping_address": { "$ref": "/schemas/address" },
    "billing_address":  { "$ref": "/schemas/address" }
  }
}
```

**Draft 4-7 behavior:** when an object contains `$ref`, the entire object is treated as a reference. Any other keywords in that object are ignored.

## $defs

Standardized location for subschemas intended for reuse within the current document. Referenced via JSON Pointer:

```json
{
  "$defs": { "name": { "type": "string" } },
  "properties": {
    "first_name": { "$ref": "#/$defs/name" },
    "last_name":  { "$ref": "#/$defs/name" }
  }
}
```

**Draft-07:** uses `definitions` (not `$defs`). `$defs` is Draft 2019-09+.

## Recursion

`$ref` may point to the schema itself (`"$ref": "#"`), enabling self-referential structures.

```json
{
  "type": "object",
  "properties": {
    "name": { "type": "string" },
    "children": { "type": "array", "items": { "$ref": "#" } }
  }
}
```

Mutual loops (`A → B → A`) are explicitly disallowed — they cause infinite resolution loops.

## Bundling

Multiple schema documents can be bundled into a single Compound Schema Document using `$id` in a subschema to create embedded schemas. Each embedded schema is an independent Schema Resource.

In Draft 4-7, a subschema `$id` only represents a base URI change — not an independent Schema Resource. All schemas in a bundled Draft-07 document must use the same dialect.
