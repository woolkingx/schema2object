# String
Source: https://json-schema.org/understanding-json-schema/reference/string

## Appendix Role

This file is upstream JSON Schema reference material. It does not own `schema2object` runtime behavior; use `../draft-07/` and `../runtime/` for local gates.

The `string` type is used for strings of text. May contain Unicode characters.

## Length

`minLength` and `maxLength` — value must be a non-negative integer.

```json
{ "type": "string", "minLength": 2, "maxLength": 3 }
```

## Pattern

`pattern` restricts a string to a particular regular expression. Syntax is ECMA 262 with Unicode support. See [regular-expressions.md](regular-expressions.md).

The match is a **search**, not anchored. Pattern `"p"` matches `"apple"`. Use `^...$` to anchor explicitly.

```json
{ "type": "string", "pattern": "^([0-9]{3})[0-9]{3}-[0-9]{4}$" }
```

## format

See [type.md](type.md) for the full list of built-in formats. By default `format` is an annotation only. Implementations may optionally treat it as an assertion.
