# 03. Boundary Validation

Back to the [handbook entry](../index.html).

## What This Chapter Teaches

External data enters through a boundary. In this runtime, that boundary is `validate()` when you only need a validation result and `ObjectTree.from()` when you need a validated live cursor.

## Mental Model

`validate()` is the normative gate. It answers: "Does this data satisfy this schema?"

`ObjectTree.from()` is the boundary constructor. It validates first, then returns an `ObjectTree` cursor.

`new ObjectTree()` is not the external boundary. It creates an internal cursor over data that is already legal.

## Correct Posture

For untrusted input:

```js
const result = validate(input, schema, loader)
if (!result.valid) throw new TypeError(result.error)
const tree = ObjectTree.from(input, schema, loader)
```

For trusted internal cursor work:

```js
const cursor = new ObjectTree(alreadyLegalData, schema, loader)
```

## What To Watch Out For

- Boundary validation is where missing required fields, invalid types, invalid defaults, and failed constraints should surface.
- Mutation through a cursor still validates assigned fields, but that does not replace the initial boundary gate.
- Use `ObjectTree.from()` in examples when data crosses from JSON or external input into runtime object access.

## Common Mistakes

| Mistake | Correct shape |
|---|---|
| `new ObjectTree(externalInput, schema)` | `ObjectTree.from(externalInput, schema)` |
| Assume cursor construction proves legality | Only `validate()` and `ObjectTree.from()` are boundary gates |
| Validate once with one schema, cursor with another | Validate and cursor against the same resolved node and loader |

## Checkpoint

You can decide between `validate()`, `ObjectTree.from()`, and `new ObjectTree()` by asking one question: is this data crossing an external boundary?
