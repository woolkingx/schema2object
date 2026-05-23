# 06. Reference Resolution

Back to the [handbook entry](../index.html).

## What This Chapter Teaches

`$ref` resolution is not only about finding a node. It is also about keeping the correct scope for future relative references.

## Mental Model

`Loader.resolve(ref)` returns a pair:

| Return value | Meaning |
|---|---|
| `node` | The resolved schema node |
| `loader` | The loader scoped to that resolved node |

The returned loader is part of the resolved contract. Keep it with the node.

## Correct Posture

```js
const { node, loader } = rootLoader.resolve('#/definitions/User')
const tree = ObjectTree.from(input, node, loader)
```

If the schema graph uses relative file references, the loader carries the context needed to keep those references legal.

## What To Watch Out For

- A resolved node without its returned loader is incomplete.
- Reusing the parent loader after resolving into another document can break nested relative `$ref`s.
- Filesystem resolution handles relative paths. HTTP or custom URI handling requires an object or function resolver.

## Common Mistakes

| Mistake | Correct shape |
|---|---|
| Pass `rootLoader` after resolving a child document | Pass the returned `loader` |
| Resolve the same ref manually in several places | Use `Loader.resolve()` as the graph boundary |
| Hide `$ref` behavior behind inline examples | Teach from schema files and resolved nodes |

## Checkpoint

You can explain why `node` and `loader` travel together after resolution.
