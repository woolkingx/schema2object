# 07. Dot-Key Authoring

Back to the [handbook entry](../index.html).

## What This Chapter Teaches

Flat dot-key data is an authoring convenience. The runtime can expand it into nested object shape while still enforcing the nested schema.

## Mental Model

Humans may prefer this:

```json
{
  "match.command.name": "rm"
}
```

Runtime code usually wants this:

```json
{
  "match": {
    "command": {
      "name": "rm"
    }
  }
}
```

The schema still describes the nested shape. Dot-key input is only a convenient authoring surface.

## Correct Posture

Use an internal cursor for dot-key expansion, then validate the final nested output at the boundary:

```js
const draft = new ObjectTree({}, cursorSchema, ruleLoader)
draft['match.command.name'] = 'rm'
const rule = ObjectTree.from(draft.$toDict(), ruleSchema, ruleLoader)
```

See [`dotkey.mjs`](../../../js/examples/dotkey.mjs).

## What To Watch Out For

- Dot-key authoring does not remove nested schema constraints.
- The final shape should still pass `ObjectTree.from()`.
- Empty or invalid nested values should fail through the same schema gate.

## Common Mistakes

| Mistake | Correct shape |
|---|---|
| Treat flat keys as the canonical data model | Treat them as authoring input |
| Skip final boundary validation after expansion | Validate the nested projection |
| Define a separate schema for flat keys | Keep the schema focused on the nested object contract |

## Checkpoint

You can explain why flat authoring and nested runtime access are two views of the same schema-defined object.
