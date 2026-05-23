# 09. Common Mistakes

Back to the [handbook entry](../index.html).

## What This Chapter Teaches

This chapter collects the fastest ways to drift away from the project model and the shortest correction for each drift.

## Mental Model

Most mistakes come from reversing the order:

```text
wrong: code/class -> example -> schema afterthought
right: schema -> boundary -> cursor -> projection
```

## Correct Posture

When unsure, return to the chain:

```text
schema file -> Loader -> resolved schema node -> validate/ObjectTree.from -> ObjectTree cursor -> explicit export
```

## What To Watch Out For

| Drift | Why it matters | Correction |
|---|---|---|
| Inline schema becomes the primary example | Hides file and loader behavior | Teach from `js/examples/schema.json` |
| `new ObjectTree()` used for external input | Skips boundary validation | Use `ObjectTree.from()` |
| Resolved node loses returned loader | `$ref` scope can drift | Pass node and loader together |
| Defaults are assumed to mutate raw data | Confuses schema-aware read with storage | Use `$withDefaults()` only when needed |
| README becomes a full manual | Competes with handbook | Keep README as front page |
| Runtime docs explain every JSON Schema keyword | Blurs ownership | Keep Draft-07 detail in `draft-07/` or reference appendix |
| Language projection looks like active package code | Misleads readers about repo ownership | Keep non-JS material under `language/` |

## Common Mistakes

The most expensive mistake is creating a second source of truth. If code, examples, README, and handbook each teach a different entry posture, every later fix becomes archaeology. Keep one posture and link to it.

## Checkpoint

Before changing docs or examples, answer:

1. Which file owns this truth?
2. Is this usage, runtime API, Draft-07 legality, language projection, or appendix material?
3. Which gate proves the claim?
4. Did the example enter through a schema file when it should?
