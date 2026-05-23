# TODO — schema2object

## Chapter Role

| Field | Value |
|---|---|
| Input artifact | Future work candidate |
| Output artifact | Bounded backlog item with owner and active-runtime scope |
| Owner | Project planning |
| Gate | No roadmap item introduces active non-JS runtime ownership |
| Failure route | Return to `handbook-skills.md` and route the item to runtime, schema, projection, or proof owner |

## Observer (L1/L2 observability)

### Done
- [x] `ObjectTree._observer` static hook — get/set emit `(op, path, key, val)` (v0.5.1)

### Next
- [ ] Schema-driven filter — `x-observe: true/false` per property, observer only fires on marked fields
- [ ] `$observe(key, callback)` — per-instance subscription (KVO-style), returns unsubscribe fn
- [ ] Batch mode — collect events during a transaction, emit once on commit
- [ ] Observer receives schema context — `(op, path, key, val, { type, description, extensions })` for structured logging

## Schema-run integration

- [ ] `x-effects when: trace` auto-wires to `_observer` at runtime — schema declares observe intent, ObjectTree executes
- [ ] L2 self-check — post-pipeline audit: count transform points vs observe decisions, flag uncovered transforms
- [ ] Pipeline step observer — schema-run pipeline emits step enter/exit/error through `_observer`

## Language Projection Documentation

- [ ] Keep TypeScript, Python, and Rust projection guides aligned with the active JavaScript runtime semantics
- [ ] Add projection examples that show schema-to-native-shape flow without introducing active non-JS runtime packages

## Performance

- [ ] Benchmark observer overhead — measure null vs active observer on 922 Draft-07 suite
- [ ] Benchmark `ObjectTree.from()` boundary cost against `new ObjectTree()` cursor creation for representative schemas
