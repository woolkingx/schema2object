# Object
Source: https://json-schema.org/understanding-json-schema/reference/object

## Appendix Role

This file is upstream JSON Schema reference material. It does not own `schema2object` runtime behavior; use `../draft-07/` and `../runtime/` for local gates.

Objects are the mapping type in JSON. Keys must always be strings.

## properties

Maps property names to their schemas. Any property that doesn't match any key in `properties` is ignored by this keyword (not rejected — see `additionalProperties`).

```json
{
  "type": "object",
  "properties": {
    "number": { "type": "number" },
    "street_name": { "type": "string" }
  }
}
```

By default, leaving out properties is valid, and providing additional properties is valid.

## patternProperties

Maps regex patterns to schemas. A property whose name matches the pattern must satisfy the schema. Patterns are unanchored — use `^...$` to anchor.

```json
{
  "patternProperties": {
    "^S_": { "type": "string" },
    "^I_": { "type": "integer" }
  }
}
```

Properties matched by `patternProperties` are excluded from `additionalProperties` checking.

## additionalProperties

Controls properties not matched by `properties` or `patternProperties`.
- `false` — no additional properties allowed
- schema — additional properties must match that schema
- absent or `true` — any additional properties allowed

**Scope rule:** `additionalProperties` only recognizes properties declared in the **same subschema**. It cannot see across `allOf` boundaries.

### Extending closed schemas

Using `additionalProperties: false` with `allOf` fails because `additionalProperties` cannot see properties from the other subschema. Workaround: move `additionalProperties` to the extending schema and redeclare the base properties.

## unevaluatedProperties

New in Draft 2019-09. Out of scope for Draft-07.

Similar to `additionalProperties` but can see across `allOf`/`anyOf`/`oneOf` boundaries.

## required

Array of property names that must be present. A property with value `null` IS present — it satisfies `required`. In Draft 4, must contain at least one string.

```json
{ "required": ["name", "email"] }
```

## propertyNames

New in Draft 6. Schema applied to every property **name** (key), not value. Since keys are always strings, this is typically a string-constraint schema.

```json
{ "propertyNames": { "pattern": "^[A-Za-z_][A-Za-z0-9_]*$" } }
```

## minProperties / maxProperties

Constrain the number of properties. Non-negative integers.

```json
{ "type": "object", "minProperties": 2, "maxProperties": 3 }
```
