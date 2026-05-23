# ObjectTree Structure

> Derived strictly from the JSON Schema specification. This is the design contract for the active JavaScript ObjectTree runtime and projection reference for other languages.

---

## Chapter Role

| Field | Value |
|---|---|
| Input artifact | Schema tree plus JSON data instance |
| Output artifact | ObjectTree binding model and serialization/access contract |
| Owner | Active JavaScript runtime, with Draft-07 as schema source |
| Gate | `ObjectTree.from()` owns external data entry; `new ObjectTree()` remains an internal cursor |
| Failure route | Return to `runtime/schema2object-usage.md` for operational usage or `runtime/javascript.md` for API surface |

---

## What JSON Schema Is

From the spec:

> "JSON Schema is a declarative format for describing the structure of other data."

A schema is itself JSON. It is data, not a program. It describes what other data should look like. JSON Schema defines six primitive types: `object`, `array`, `string`, `number`, `integer`, `boolean`, `null`.

`ObjectTree` takes this one step further: the schema does not sit beside the data describing it from outside. The schema and the data are bound into one identity. The schema IS the class. The data is the instance of that class.

---

## Instance and Schema — One Identity

```
ObjectTree.from(data, schema)
```

- `data` — the JSON instance (any of the six types)
- `schema` — the class definition (a JSON Schema object)

These are not two separate objects that interact at call time. They are one thing. Once bound, every access, write, and read goes through the schema.

The schema is also an ObjectTree. Schema keywords are accessible as fields on the schema node, mirroring the same dot-access as data.

---

## The Six Instance Types

An ObjectTree node holds exactly one of these at runtime:

| Type | JSON form | Notes |
|---|---|---|
| `object` | `{"key": value, ...}` | keys always strings |
| `array` | `[v1, v2, ...]` | ordered, elements may differ in type |
| `string` | `"text"` | Unicode |
| `number` | `1`, `1.5`, `1e10` | integer or float |
| `integer` | `1`, `42` | whole number; `1.0` qualifies, `1.1` does not |
| `boolean` | `true`, `false` | distinct from 0/1 |
| `null` | `null` | distinct type, not absence |

`type` may be a single string or an array of strings. Array form means the instance is valid if it matches any of the listed types.

---

## Schema Keywords as Class Members

Every keyword in a schema is a member of the class definition. They fall into four categories:

### 1. Type constraint
`type` — restricts which JSON type the instance may be.

### 2. Value constraints (per type)

**Object:** `properties`, `required`, `additionalProperties`, `patternProperties`, `minProperties`, `maxProperties`, `dependencies`, `propertyNames`

**Array:** `items`, `additionalItems`, `contains`, `minItems`, `maxItems`, `uniqueItems`

**String:** `minLength`, `maxLength`, `pattern`, `format`

**Number/Integer:** `minimum`, `maximum`, `exclusiveMinimum`, `exclusiveMaximum`, `multipleOf`

**Any type:** `const`, `enum`

### 3. Logic / composition
`allOf`, `anyOf`, `oneOf`, `not`, `if`, `then`, `else`

These are boolean algebra over sub-schemas. They do not merge schemas or add properties — each sub-schema is evaluated independently against the same instance.

### 4. Annotations
`title`, `description`, `default`, `examples`, `readOnly`, `writeOnly`, `$comment`

From the spec: annotations are **not used for validation**. They add human-readable or tooling metadata to the schema. They do not constrain what instances are valid.

`default` specifically: *"not used to fill in missing values during the validation process."* It is documentation of intent, not an auto-fill instruction.

---

## Instantiation

```
ObjectTree.from(data, schema)
```

The schema defines what data is a valid instance. External data enters through `ObjectTree.from(data, schema)` or `validate(data, schema)`. The internal `new ObjectTree(data, schema)` constructor is a cursor over already-legal or staged data.

**Empty schema `{}`** — accepts any valid JSON. Always succeeds.

**Boolean schema `true`** — equivalent to `{}`, accepts everything.

**Boolean schema `false`** — accepts nothing. Always fails.

**`$ref`** — must be resolved before instantiation proceeds. A `$ref` in the schema replaces itself with the referenced schema. The instance is then checked against the resolved schema.

### Sub-schema propagation

When data is an object or array, each child inherits its own sub-schema:

- Object key `k` → sub-schema from `schema.properties[k]`
- Array element at index `i` → sub-schema from `schema.items` (single schema form) or `schema.items[i]` (tuple form)

Children are themselves ObjectTree instances, carrying their sub-schema. The schema tree mirrors the data tree.

---

## Properties Keyword — the Class Member Map

`properties` is the central keyword for object instances. It maps field names to their sub-schemas:

```json
{
  "type": "object",
  "properties": {
    "name": { "type": "string" },
    "age":  { "type": "integer", "minimum": 0 }
  }
}
```

From the spec: *"Any property that doesn't match any of the property names in `properties` is ignored by this keyword."*

This means:
- Properties not listed in `properties` are allowed at runtime
- They are unknown to the class definition
- `$toDict()` projects only schema-defined fields — unknown fields are excluded from serialization

`required` lists which properties must be present. Absent from `required` = optional. Not listed in `properties` at all = unknown, allowed but untyped.

---

## additionalProperties and patternProperties

`additionalProperties` applies to keys not matched by `properties` or `patternProperties`. Default: allowed.

`patternProperties` maps regex patterns to sub-schemas. A key matching a pattern must satisfy that sub-schema.

**Scope rule from spec:** `additionalProperties` only recognizes properties declared in the **same subschema**. It cannot see across `allOf` boundaries.

---

## Array: items and additionalItems

Two modes:

**List validation** — `items` is a single schema. Every element must match.

**Tuple validation** — `items` is an array of schemas. Position `i` applies schema `items[i]`. Elements beyond the defined positions are controlled by `additionalItems`.

`contains` — at least one element must match. Does not apply to all elements.

---

## Composition Keywords

`allOf`, `anyOf`, `oneOf`, `not` operate on the instance as a whole:

- `allOf` — instance must satisfy ALL sub-schemas (AND)
- `anyOf` — instance must satisfy AT LEAST ONE sub-schema (OR)
- `oneOf` — instance must satisfy EXACTLY ONE sub-schema (XOR)
- `not` — instance must NOT satisfy the sub-schema

From the spec on `allOf`: *"Instances must independently be valid against 'all of' the schemas."* This is not inheritance. Each sub-schema is checked in full.

These keywords expose as methods on ObjectTree that re-bind the instance to the resolved sub-schema view:

| Keyword | Method | Returns |
|---|---|---|
| `oneOf` | `$oneOf()` | ObjectTree re-bound to the matching sub-schema |
| `anyOf` | `$anyOf()` | list of ObjectTree, one per matching sub-schema |
| `allOf` | `$allOf()` | ObjectTree re-bound to merged schema view |
| `not` | `$notOf()` | bool |
| `if/then/else` | `$ifThen()` | ObjectTree re-bound to the matching branch |

Re-binding means: same data, different schema lens. The original instance is not modified.

---

## if / then / else

New in Draft-07. Conditional sub-schema application:

| `if` matches | outcome |
|---|---|
| yes, `then` present and matches | valid |
| yes, `then` present and fails | invalid |
| no, `else` present and matches | valid |
| no, `else` present and fails | invalid |
| `then`/`else` absent | valid (if is treated as passing) |

`then` or `else` without `if` — ignored by spec.

---

## Serialization: $toDict()

`$toDict()` returns the **schema-defined projection** of the instance:

- For object nodes with `properties`: returns only keys defined in `properties` that exist in data
- For array nodes: returns all elements (no projection)
- For scalar nodes: returns the value directly

Unknown fields — present in data, absent from `properties` — are excluded. This is intentional. The class definition determines what the serialized form contains.

`$value` / full access — returns all data including unknown fields and lazy defaults as a deterministic snapshot.

---

## $ref and definitions

Schema is a JSON object. `$ref` is a value inside that object — a JSON Pointer path that names another object node within the same schema tree.

```json
{
  "properties": {
    "address": { "$ref": "#/definitions/Address" }
  },
  "definitions": {
    "Address": { "type": "object", "properties": { "city": { "type": "string" } } }
  }
}
```

`address`'s schema is the `Address` object. `$ref` says where in the schema tree that object lives. Multiple fields may point to the same object — they share the same definition.

JSON Pointer paths within a schema:

- `#/definitions/Address` — the `Address` object inside `definitions`
- `#` — the root schema object itself

**Draft-07:** uses `definitions`. (`$defs` is Draft 2019-09+.)

### Recursion

`$ref: "#"` points to the root schema object, enabling self-referential structures:

```json
{
  "type": "object",
  "properties": {
    "name": { "type": "string" },
    "children": { "type": "array", "items": { "$ref": "#" } }
  }
}
```

Mutual loops (`A → B → A`) are disallowed by spec.

---

## Annotations

Not enforced at instantiation. Preserved as metadata on the schema node.

| Keyword | Purpose |
|---|---|
| `title` | Short human-readable name |
| `description` | Longer explanation |
| `default` | Semantic default — documentation only, not auto-fill |
| `examples` | Example valid values |
| `readOnly` | API hint: should not be written |
| `writeOnly` | API hint: should not be read back |
| `$comment` | Schema author notes, invisible to applications |

`$withDefaults()` — explicit opt-in operation that applies `default` values to missing fields. Never automatic.

---

## x-* Extensions

JSON Schema allows any `x-`-prefixed keyword as a custom extension. Ignored by spec-compliant validators. Preserved on the schema and accessible via `$getExtensions(path)`.

---

## Schema Access API

Every ObjectTree instance exposes its schema:

| Access | Returns |
|---|---|
| `$schema` | root schema as plain object |
| `$getSchema(path)` | sub-schema at dot path, e.g. `"address.zip"` |
| `$getExtensions(path)` | `x-*` keys from a schema node |

Schema is immutable after construction.
