# Schema as Object

## Chapter Role

| Field | Value |
|---|---|
| Input artifact | A JSON Schema document |
| Output artifact | The schema-as-object identity used by runtime, projection, and proof chapters |
| Owner | Draft-07 schema contract |
| Gate | The chapter does not claim language-specific runtime ownership |
| Failure route | Return to `structure.md` for ObjectTree binding details or `keywords.md` for keyword legality |

## JSON Schema Is JSON

The JSON Schema specification states: "JSON Schema itself is written in JSON. It is data itself, not a computer program."

A schema is a JSON object. Its keywords are keys. Their values are the constraints. The schema does not execute. It does not run. It sits in memory as a plain object, and other processes read it.

This is the foundational identity: a schema is not a description of data stored separately from data. A schema is data.

## Schema as Class Definition

A schema defines which instances are valid. Any piece of data that satisfies the schema is an instance of that schema. The schema is the class definition.

This is not metaphor. The schema object holds the type, the required fields, the property constraints — everything that distinguishes valid from invalid. Nothing else is needed. The schema IS the class.

## The Schema Tree Mirrors the Data Tree

A schema for an object contains a `properties` keyword. The value of `properties` is itself a JSON object. Each key in `properties` names a field. Each value is a schema object for that field.

The structure recurses. A field schema is itself a full schema — it can have its own `properties`, its own `type`, its own constraints. The schema tree and the instance data tree have the same shape.

Consider an object with a field `address`, which is itself an object with a field `city`. The root schema has a property `address`. The `address` schema has a property `city`. The schema tree mirrors the data tree at every level.

Each node in the schema tree carries the definition for the corresponding node in the data tree.

## `$ref` — Pointing to Another Schema Object

`$ref` is a keyword whose value is a JSON Pointer string. It identifies another schema object within the schema document.

The pointer `#/definitions/Foo` means: start at the root (`#`), descend into `definitions`, take the value at key `Foo`. That value is a schema object. The `$ref` stands in for that object.

Multiple fields can use the same `$ref` value. They all point to the same schema object. They share one definition — no duplication.

The pointer `#` refers to the root schema object itself. A schema can reference itself. This enables recursive structures: a tree node whose children are also tree nodes, a linked list whose `next` field has the same schema as the list itself.

Draft-07 convention places reusable schema objects under `definitions` at the root of the schema document.

## Reuse Without Duplication

`definitions` is a JSON object at the root of the schema. Its values are schema objects. Those objects are inert until referenced — they participate in validation only when a `$ref` points to them.

This is object reuse by reference. A schema object in `definitions` is defined once. Any number of `$ref` pointers can name it. The object is not copied. It is shared.

This pattern is not specific to JSON Schema. It is how any object graph works: one object, multiple references to it. JSON Schema applies the same principle to schema objects within a schema document.

---

The schema is an object. The schema tree mirrors the data tree. References point to objects within that tree. Reuse is reference, not repetition. These are not design decisions — they follow directly from the identity: JSON Schema is JSON.
