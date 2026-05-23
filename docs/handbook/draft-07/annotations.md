# Annotation Keywords — JSON Schema Draft-07 Spec Reference

> Annotations do not affect validation. They carry metadata about the schema. No annotation keyword constrains what instances are valid.

---

## Chapter Role

| Field | Value |
|---|---|
| Input artifact | Annotation keyword in a schema |
| Output artifact | Validation-neutral meaning and allowed runtime use |
| Owner | Draft-07 annotation contract |
| Gate | `default` stays validation-neutral; runtime default behavior is described separately |
| Failure route | Return to `keywords.md` for `schema2object` runtime exception notes |

---

## Annotation Keywords

| Keyword | Type | Purpose |
|---|---|---|
| `title` | string | Short human-readable name for the schema. |
| `description` | string | Longer explanation of the schema's purpose. |
| `default` | any | Documents the intended default value. Does not fill in missing values. |
| `examples` | array | Example values that would validate against the schema. Not used for validation. |
| `readOnly` | boolean | Signals that the value should not be modified. Meaningful in API contexts. |
| `writeOnly` | boolean | Signals that the value will not be returned when reading. Meaningful in API contexts. |

### `default`

The spec states explicitly: `default` is **"not used to fill in missing values during the validation process."**

`default` documents intent. It tells a reader what value the author considers appropriate when a field is absent. It does not instruct a validator or runtime to supply that value automatically. Any system that applies defaults does so as an explicit opt-in operation outside the validation process — not as a consequence of the keyword's presence.

This is the most commonly misread annotation. `default` is documentation, not behavior.

### `readOnly` and `writeOnly`

Both are boolean. They are hints for API tooling, not validation constraints.

- `readOnly: true` — the value should not be modified by the client. Sending a modified value is not a schema validation error.
- `writeOnly: true` — the value will not be returned in a read response. Its presence in a write payload is expected.

Neither keyword causes validation to fail. They inform tooling about the intended access pattern.

### `deprecated` (Draft 2019-09+)

Out of scope for Draft-07.

---

## `$comment`

Introduced in Draft-07.

`$comment` is for schema authors. Its value must be a string. It records notes, rationale, or maintenance context directly in the schema.

Three rules from the spec:

1. Implementations must not attach any meaning or behavior to `$comment`.
2. Implementations may strip `$comment` at any time without affecting correctness.
3. `$comment` is not for communicating with users of the schema. It is for schema maintainers.

A validator that reads `$comment` and changes its behavior is non-compliant. A toolchain that strips all `$comment` values before distribution is compliant.

---

## `x-*` Extensions

JSON Schema permits any keyword prefixed with `x-` as a custom extension.

Spec-compliant validators ignore all unknown keywords, including `x-*` keywords. Their presence does not cause validation to fail.

Applications may define and use `x-*` keywords for their own purposes — documentation tooling, test harness metadata, code generation hints, or any other application-layer need. The prefix signals that the keyword is outside the JSON Schema vocabulary and carries no spec-defined semantics.

Examples of common usage patterns:

- `x-docs` — link to external documentation
- `x-tests` — reference to test fixtures for this schema
- `x-generated` — code generation metadata

The spec makes no requirements about what `x-*` values contain or how applications must interpret them.
