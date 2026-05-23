# Dialect and Vocabulary Declaration
Source: https://json-schema.org/understanding-json-schema/reference/schema

## Appendix Role

This file is upstream JSON Schema reference material. It does not own `schema2object` runtime behavior; use `../draft-07/` and `../runtime/` for local gates.

## $schema

The `$schema` keyword declares which dialect of JSON Schema the schema was written for. Its value is also the identifier for a meta-schema that can validate the schema itself.

- `$schema` applies to the entire document and must be at the root level.
- It does not apply to externally referenced (`$ref`) documents — those must declare their own `$schema`.
- If absent, implementations may make assumptions about which version to use. Recommended: always include `$schema`.

```json
"$schema": "http://json-schema.org/draft-07/schema#"
```

## Vocabularies

New in Draft 2019-09. Out of scope for Draft-07.

Before vocabularies existed, you could extend JSON Schema with custom keywords by copying the meta-schema for your target draft and assigning a custom URI. That custom URI is then used as the `$schema` value. Not all implementations support custom meta-schemas.

### Custom keyword interoperability

- **Annotation-only keywords** (e.g. `units`) — most interoperable; don't affect validation, always ignored safely.
- **Non-schema keywords** (e.g. `isEven`) — partial interoperability; validators that don't understand them skip them.
- **Keywords that apply subschemas or modify existing keywords** — least interoperable; schema becomes unusable without the custom implementation.
