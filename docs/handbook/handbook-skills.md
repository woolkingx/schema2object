# Handbook Skills

This chapter defines how to read and maintain the `schema2object` handbook.
It is a routing map, not a usage guide. The operational usage guide remains
[`runtime/schema2object-usage.md`](runtime/schema2object-usage.md) and is an
independent chapter.

## Core Contract

| Field | Value |
|---|---|
| Input artifact | A question, edit, release note, or runtime/API change |
| Output artifact | The owning handbook chapter, gate, and next action |
| Owner | `docs/handbook/index.html` routes; each chapter owns its own stage |
| Gate | `GATE-HANDBOOK-01`: links resolve, JSON docs parse, usage guide and `.mjs` files remain untouched unless explicitly requested |
| Failure route | Return to `index.html`, find the correct owner, then edit the owning chapter instead of scattering prose |

## Stage Cards

| Stage | First question | Owning chapter | Output |
|---|---|---|---|
| Perceive | What is this repo and what is active? | `index.html`, `README.md`, `AGENTS.md` | JS-only active runtime posture |
| Usage | How do I use it correctly? | `runtime/schema2object-usage.md` | Loader -> resolve -> boundary -> ObjectTree posture |
| Runtime | What API surface exists? | `runtime/javascript.md`, `runtime/schema2object-api.json`, `runtime/schema2object.schema.json` | JS API contract and cursor/boundary split |
| Schema | What JSON Schema shape is legal? | `draft-07/*.md`, `draft-07/spec.json`, `draft-07/meta-schema.json` | Draft-07 keyword and structure contract |
| Projection | How do other languages carry the schema truth? | `language/*.md` | Documentation-only TypeScript/Python/Rust projection guidance |
| Reference | What upstream JSON Schema facts are being cited? | `json-schema-reference/*.md` | Appendix reference notes, not runtime ownership |
| Proof | Did the claim hold? | `js/examples/*.mjs`, `js/tests/*.mjs`, `docs/draft-07*` | Fresh command output and fixture coverage |
| Closure | What becomes durable? | `changelog.md`, `roadmap.md`, `decisions/*.md` | Historical record, future work, completed decisions |

## Chapter Ownership Matrix

| Chapter | Owns | Must not own | Gate |
|---|---|---|---|
| `index.html` | Reader contract, source-of-truth map, chapter map, reading order, gate families | Full runtime spec or long examples | `GATE-HANDBOOK-01` |
| `handbook-skills.md` | Stage routing, chapter ownership, maintenance rules | Runtime usage examples | `GATE-HANDBOOK-01` |
| `runtime/schema2object-usage.md` | Correct operational usage posture | General API inventory or projection methodology | `GATE-USAGE-01`: untouched unless explicitly requested |
| `runtime/javascript.md` | JavaScript runtime surface and examples | Replacing the usage guide | `GATE-RUNTIME-01`: examples agree with `js/schema2object.mjs` |
| `runtime/schema2object-api.json` | Machine-readable public API shape | Narrative usage doctrine | `GATE-RUNTIME-02`: JSON parse succeeds |
| `runtime/schema2object.schema.json` | Runtime role schema and flow contract | User tutorial prose | `GATE-RUNTIME-02`: JSON parse succeeds |
| `draft-07/*.md` | Draft-07 semantics used by the JS runtime and projections | Active Python/Rust implementation claims | `GATE-SCHEMA-01`: keyword notes agree with Draft-07 fixtures |
| `draft-07/spec.json` | Local encoded Draft-07 keyword contract | Runtime API behavior | `GATE-SCHEMA-02`: JSON parse succeeds |
| `draft-07/meta-schema.json` | Local Draft-07 meta-schema artifact | Project-specific runtime semantics | `GATE-SCHEMA-02`: JSON parse succeeds |
| `language/*.md` | Documentation-only projection guidance | Active runtime package code or canonical ObjectTree semantics | `GATE-PROJECTION-01`: projection does not invent schema truth |
| `json-schema-reference/*.md` | Appendix facts copied/summarized from JSON Schema reference material | `schema2object` ownership or runtime implementation truth | `GATE-REFERENCE-01`: reference remains appendix material |
| `changelog.md` | Historical release notes | Current architecture truth | `GATE-HISTORY-01`: historical non-JS entries are clearly historical |
| `roadmap.md` | Future work | Completed decisions or current truth | `GATE-ROADMAP-01`: no active non-JS runtime tasks |
| `decisions/*.md` | Completed decision records | General documentation prose | `GATE-DECISION-01`: acceptance remains checkable |

## Maintenance Rules

1. Start at `index.html` to locate the owning chapter.
2. If the question is "how do I use it?", route to `runtime/schema2object-usage.md`.
3. If the question is "what API exists?", route to `runtime/javascript.md` or the runtime JSON contracts.
4. If the question is "what is legal schema shape?", route to `draft-07/`.
5. If the question is "how does this map into another language?", route to `language/`.
6. If the question is "what proves this?", run the relevant gate before making a completion claim.

## Verification Command Set

```bash
node /home/claude/agents/skills/wx-explore-handbook/scripts/handbook-link-check.mjs docs/handbook
node -e "for (const f of ['docs/handbook/runtime/schema2object-api.json','docs/handbook/runtime/schema2object.schema.json','docs/handbook/draft-07/spec.json','docs/handbook/draft-07/meta-schema.json']) JSON.parse(require('fs').readFileSync(f,'utf8'))"
git diff -- docs/handbook/runtime/schema2object-usage.md
git diff -- 'js/**/*.mjs'
node js/examples/basic.mjs
node js/examples/dotkey.mjs
node js/examples/observer_schema_context.mjs
node js/tests/validate.mjs
node js/tests/draft07_suite.mjs
```

## Drift Rules

- Runtime behavior comes from `js/schema2object.mjs` and is proved by `js/tests/`.
- Operational usage comes from `runtime/schema2object-usage.md`.
- Projection chapters must not claim active runtime ownership.
- Reference appendix chapters must not become architecture truth.
- Historical changelog entries may mention Python/Rust, but current posture must stay JS-only active runtime.
