# Language Reference Canonicalization Plan

Goal: move the language projection guide source of truth from `schema-driven-development` into the `schema2object` handbook, then let `schema-driven-development` point back here with symlinks.

## Chapter Role

| Field | Value |
|---|---|
| Input artifact | Completed language-reference migration decision |
| Output artifact | Acceptance record for projection-guide canonicalization |
| Owner | Decision history |
| Gate | Python and Rust remain projection guides, not active runtimes |
| Failure route | Return to `../handbook-skills.md` for current routing or `../language/quick-start.md` for projection posture |

## Tasks

- [x] Copy language reference files from `/home/claude/projects/schema-driven-development/master/references`.
- [x] Localize copied references under `docs/handbook/language/` so they describe `schema2object` as the owner, not `schema-driven-development`.
- [x] Update root and per-language README/AGENTS surfaces to match the new language posture.
- [x] Verify no stale navigation still presents Python or Rust as active dynamic runtimes.
- [x] Remove active Python/Rust runtime directories from this repository shape.

## Acceptance

- `docs/handbook/language/quick-start.md`, `docs/handbook/language/typescript-ajv.md`, `docs/handbook/language/python.md`, and `docs/handbook/language/rust.md` exist in this repo.
- Root README and AGENTS identify JavaScript as the active dynamic runtime.
- Python and Rust docs point to projection guides and do not claim ownership of canonical `ObjectTree` semantics.
- Readback grep shows the expected projection phrases and no stale `Python | 922/922` / `Rust | 922/922` implementation table remains.
