# 08. Observer Hooks

Back to the [handbook entry](../index.html).

## What This Chapter Teaches

The observer hook can inspect cursor reads and writes with schema metadata. It is for instrumentation and reactive bindings, not for owning validation truth.

## Mental Model

The observer sees operations. The schema still owns legality. The cursor performs access and mutation. The observer should remain a side channel.

```text
cursor operation -> observer event with schema metadata
```

## Correct Posture

Use the hook to observe, filter, log, or bind:

```js
ObjectTree._observer = (op, path, key, val, schema) => {
  if (schema?.['x-observable'] === false) return
  console.log(op, path, key, schema?.type)
}
```

Clear it after use:

```js
ObjectTree._observer = null
```

See [`observer_schema_context.mjs`](../../../js/examples/observer_schema_context.mjs).

## What To Watch Out For

- The hook is global. Treat it as shared runtime instrumentation.
- Do not put validation rules in the observer.
- Do not mutate unrelated data from inside observer callbacks.
- Use schema extensions such as `x-observable` as metadata, not as hidden runtime contracts.

## Common Mistakes

| Mistake | Correct shape |
|---|---|
| Observer owns legality | Schema and validation own legality |
| Leave observer enabled across unrelated tests | Reset `ObjectTree._observer = null` |
| Use observer to patch data shape | Use schema-governed cursor mutation |

## Checkpoint

You can describe observer hooks as instrumentation over cursor operations, not as a second rule engine.
