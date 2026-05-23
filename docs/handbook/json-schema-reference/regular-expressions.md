# Regular Expressions
Source: https://json-schema.org/understanding-json-schema/reference/regular_expressions

## Appendix Role

This file is upstream JSON Schema reference material. It does not own `schema2object` runtime behavior; use `../draft-07/` and `../runtime/` for local gates.

Used by `pattern` (string) and `patternProperties` (object). Syntax is ECMA 262 (JavaScript) with Unicode support. Not all implementations support the full ECMA 262 syntax — stick to the subset below for portability.

## Supported subset

| Syntax | Meaning |
|---|---|
| `.` | Any character except line break |
| `^` | Start of string |
| `$` | End of string |
| `(...)` | Group |
| `\|` | Alternation |
| `[abc]` | Character class |
| `[a-z]` | Character range |
| `[^abc]` | Negated character class |
| `+` | One or more (greedy) |
| `*` | Zero or more (greedy) |
| `?` | Zero or one (greedy) |
| `+?`, `*?`, `??` | Non-greedy versions |
| `(?!x)`, `(?=x)` | Negative and positive lookahead |
| `{x}` | Exactly x occurrences |
| `{x,y}` | Between x and y occurrences |
| `{x,}` | x or more occurrences |
| `{x,y}?`, `{x,}?` | Non-greedy versions |

Use only standard escapes: `\n`, `\r`, `\t`. Also apply JSON string escaping.

## Key behavior

Patterns are **unanchored** — a pattern matches if it matches anywhere in the string. Use `^...$` to require a full-string match.

For multiline strings, use `(.|\r?\n)*` rather than the `.` with regex flags (flags are not supported in JSON Schema patterns).

## Examples

North American phone number with optional area code:
```json
{ "type": "string", "pattern": "^(\\([0-9]{3}\\))?[0-9]{3}-[0-9]{4}$" }
```

Mustache template allowing multiline:
```json
{ "type": "string", "pattern": "^\\{\\{(.|[\\r\\n])*\\}\\}$" }
```
