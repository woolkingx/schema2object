# 01. Mental Model

Back to the [handbook entry](../index.html).

## What This Chapter Teaches

`schema2object` treats JSON Schema Draft-07 as the object class definition. The schema defines what shape is legal. Data provides values. The runtime gives you a live cursor over data that is governed by the schema.

## Mental Model

Think in four layers:

| Layer | Owns | In this repo |
|---|---|---|
| Data | Raw values | JSON payloads such as `user.json` |
| Schema | Legal shape | `schema.json` and Draft-07 keywords |
| Boundary | Legality check | `validate()` and `ObjectTree.from()` |
| Cursor | Live access | `ObjectTree` proxy navigation and mutation |

The schema is not decoration. It is the class definition. The data is not a class. It is an instance that must satisfy the schema before it crosses the external boundary.

## Correct Posture

Start from the schema file. Resolve the schema node you want. Validate external data at the boundary. Use the returned `ObjectTree` as the live cursor.

```text
schema file -> Loader -> resolved schema node -> validate/ObjectTree.from -> cursor
```

Use `new ObjectTree()` only when the data is already legal and you need an internal cursor.

## What To Watch Out For

- Do not confuse schema defaults with data already stored in the raw object.
- Do not treat spread syntax as the same as live property access.
- Do not make inline schemas the primary teaching path.
- Do not treat other language chapters as active package implementations.

## Common Mistakes

| Mistake | Correct shape |
|---|---|
| Start with a hand-written class | Start with a schema file |
| Validate after mutation only | Validate at the external boundary |
| Use `new ObjectTree()` for untrusted data | Use `ObjectTree.from()` |
| Copy JSON Schema reference text into runtime docs | Link reference material and keep runtime ownership clear |

## Checkpoint

After this chapter, you should be able to explain why this project is not "JSON validation plus helpers." It is a schema-defined object runtime: schema defines the legal class, boundary checks legality, and cursor operations work inside that legality.
