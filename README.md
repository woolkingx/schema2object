# schema2object

`schema2object` is a JavaScript runtime for using JSON Schema Draft-07 as an object contract.

It loads a schema file, resolves `$ref` through a root `Loader`, validates external data at the boundary, and exposes the result as a live `ObjectTree` with schema-aware property access, defaults, dot-key authoring support, and observer hooks.

Use it when JSON Schema is the source of truth for object shape and you want runtime objects without hand-writing parallel classes.

Current repository posture: JavaScript is the only active runtime implementation. Python, Rust, and TypeScript material belong in the handbook as projection guidance, not package usage or active runtime code.

## Project

- [Handbook](./docs/handbook/index.html)
- [Guided learning path](./docs/handbook/guide/01-mental-model.md)
- [Usage guide](./docs/handbook/runtime/schema2object-usage.md)
- [Changelog](./CHANGELOG.md)
- [Runtime source](./js/schema2object.mjs)
- [Examples](./js/examples/)
