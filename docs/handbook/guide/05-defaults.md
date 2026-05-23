# 05. Defaults

Back to the [handbook entry](../index.html).

## What This Chapter Teaches

Schema `default` values are visible through the cursor without being written into raw data. Materialization is explicit.

## Mental Model

A default belongs to the schema. It is the initial value a schema-aware reader can see when the raw data is missing that field. It is not automatically stored in the raw object.

## Correct Posture

Read defaults lazily:

```js
tree.role
```

Use `$withDefaults()` only when a schema-unaware consumer needs the defaults committed into a plain data shape.

```js
const filled = tree.$withDefaults()
```

## What To Watch Out For

- Reading a missing key with a default does not mutate raw data.
- `Object.keys(tree)` includes default-only keys so enumeration matches cursor reads.
- Invalid defaults fail at boundary validation.
- `$withDefaults()` returns a fresh cursor over cloned data.

## Common Mistakes

| Mistake | Correct shape |
|---|---|
| Expect defaults to appear in raw `$value` after read | Use `$withDefaults()` for materialization |
| Mutate raw data to fill defaults manually | Let schema defaults drive cursor reads or explicit materialization |
| Treat object defaults as shared state | `$withDefaults()` deep-clones object defaults |

## Checkpoint

You can tell whether a value came from raw data or schema default, and you know when to leave it lazy versus when to materialize it.
