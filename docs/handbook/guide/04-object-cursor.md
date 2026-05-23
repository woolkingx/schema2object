# 04. Object Cursor

Back to the [handbook entry](../index.html).

## What This Chapter Teaches

`ObjectTree` is a live proxy cursor. It lets code read, write, inspect, and export schema-governed data without hand-writing parallel classes.

## Mental Model

The cursor does not replace the raw data. It wraps access to the data and uses the schema to decide what fields exist, what defaults are visible, and which mutations are legal.

```text
raw data + schema + loader -> ObjectTree cursor
```

## Correct Posture

Use property access for live reads:

```js
user.name
user.address.city
```

Use assignment for schema-checked mutation:

```js
user.age = 31
```

Use `$toDict()` when you need a schema-defined plain-object projection.

## What To Watch Out For

- `tree.key` is live access.
- `{ ...tree }` is shallow enumeration and materialization.
- `$value` is the raw value surface.
- `$toDict()` exports schema-defined fields.
- Nested objects and arrays are wrapped as cursors when accessed.

## Common Mistakes

| Mistake | Correct shape |
|---|---|
| Treat spread as live access | Use direct property access for live cursor behavior |
| Export raw data when a schema projection is required | Use `$toDict()` |
| Expect every nested value to be wrapped before access | Wrapping happens on cursor access |

## Checkpoint

You can explain the difference between live access, shallow enumeration, raw `$value`, and schema-defined export.
