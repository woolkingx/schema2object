# 02. Schema File First

Back to the [handbook entry](../index.html).

## What This Chapter Teaches

The durable entrypoint is a schema file, not an inline schema literal. Examples should begin from [`js/examples/schema.json`](../../../js/examples/schema.json) so the reader learns the same posture used by real projects.

## Mental Model

A schema file is the root of a schema graph. It may contain definitions and `$ref` edges. A `Loader` owns traversal through that graph and returns both the resolved node and the loader scope needed to continue resolving references correctly.

## Correct Posture

Use this order:

1. Read the root schema file.
2. Create a root `Loader`.
3. Resolve the definition or schema node needed by the boundary.
4. Pass both the resolved node and returned loader to validation or cursor construction.

```js
const rootSchema = readJson('schema.json')
const rootLoader = new Loader(rootSchema)
const { node: userSchema, loader: userLoader } =
  rootLoader.resolve('#/definitions/User')
const user = ObjectTree.from(input, userSchema, userLoader)
```

See [`basic.mjs`](../../../js/examples/basic.mjs) for the runnable version.

## What To Watch Out For

- Keep the schema graph root explicit.
- Keep resolved schema nodes paired with the loader returned by `resolve()`.
- Use inline schema snippets only for tiny API reference examples, not as the project posture.
- If examples drift away from `schema.json`, the handbook stops teaching the real runtime path.

## Common Mistakes

| Mistake | Why it fails |
|---|---|
| Build examples with inline schemas only | Hides root loader and `$ref` scope behavior |
| Resolve a schema node but keep using the parent loader | Relative `$ref` resolution may use the wrong scope |
| Treat `schema.json` as demo data | It is the example contract entrypoint |

## Checkpoint

You can identify the active example schema file, follow a definition through `Loader.resolve()`, and name which loader should be passed into `ObjectTree.from()`.
