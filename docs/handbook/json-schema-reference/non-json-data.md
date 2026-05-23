# Media: String-encoding Non-JSON Data
Source: https://json-schema.org/understanding-json-schema/reference/non_json_data

## Appendix Role

This file is upstream JSON Schema reference material. It does not own `schema2object` runtime behavior; use `../draft-07/` and `../runtime/` for local gates.

New in Draft 7.

JSON Schema provides keywords to describe and optionally validate non-JSON data stored inside JSON strings. Validators are not required to validate the contents based on these keywords. Applications use them to encode and decode data during storage and transmission.

## contentMediaType

Specifies the media type of the string's content, per RFC 2046.

```json
{ "type": "string", "contentMediaType": "text/html" }
```

## contentEncoding

Specifies the encoding used to store the contents, per RFC 2045 §6.1 and RFC 4648.

Acceptable values: `quoted-printable`, `base16`, `base32`, `base64`.

If not specified, the encoding is the same as the containing JSON document.

Two scenarios:
1. **Text content** — omit `contentEncoding`, include content as-is (assumes UTF-8).
2. **Binary content** — set `contentEncoding` to `base64`.

```json
{ "type": "string", "contentEncoding": "base64", "contentMediaType": "image/png" }
```

## contentSchema

New in Draft 2019-09. Out of scope for Draft-07.

A valid JSON Schema that describes the structure of the decoded content. Only used when `contentMediaType` is also present.
