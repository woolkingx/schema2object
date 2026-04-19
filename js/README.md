# schema2object — JavaScript

JSON Schema defines the object class — structure + constraints + behavioral logic.

Current mainline: `v0.6.0` pointer + proxy + lazy.

## Install

```javascript
import { ObjectTree, validate } from './schema2object.mjs'
```

## Quick Start

```javascript
const schema = {
  type: 'object',
  properties: {
    name: { type: 'string' },
    age:  { type: 'integer', minimum: 0, default: 0 }
  },
  required: ['name']
}

const user = new ObjectTree({ name: 'Alice' }, schema)
user.name          // 'Alice' — data property
user.age           // 0 — default preserved
user.age = 30      // validates: ok
user.age = -1      // throws RangeError: $.age: must be >= 0
user.age = 'old'   // throws TypeError: $.age: expected integer
```

`tree.key` is a live proxy access path: nested objects and arrays stay wrapped.
`{ ...tree }` is a shallow enumeration path: it materializes a plain object snapshot with raw values.
That difference is intentional.

## Constructor

```javascript
// Inline schema — no external $ref
const tree = new ObjectTree(data, schema)

// With resolver — for $ref resolution
// resolver can be:
//   string   → base directory for relative $ref file paths (no HTTP support)
//   object   → { uri: schema } map, looked up by resolved URI
//   function → (uri) => schema, called when $ref can't resolve internally
const tree = new ObjectTree(data, schema, './schemas/')
const tree = new ObjectTree(data, schema, { 'http://example.com/addr.json': addrSchema })
const tree = new ObjectTree(data, schema, (uri) => fetchSchema(uri))
```

## Loading Schema from File

Schema is always a plain object. Read and parse it yourself — the lib doesn't do I/O.

```javascript
import { readFileSync } from 'fs'

const schema = JSON.parse(readFileSync('./schemas/user.json', 'utf8'))
const user = new ObjectTree(data, schema)

// If schema uses relative $ref, pass base dir as resolver
const user = new ObjectTree(data, schema, './schemas/')
```

## $ Prefix Convention

All API methods use `$` prefix. Data properties don't. Zero collision.

```javascript
// data properties — from schema.properties
user.name               // read
user.name = 'Bob'       // write (validates)

// API methods — $-prefixed
user.$value             // full data as plain object
user.$schema            // resolved schema
user.$toDict()          // schema-defined fields only (projection)
user.$getSchema('age')  // sub-schema for a field
user.$getExtensions()   // all x-* extensions
```

Why: if your schema defines a property called `value` or `schema`, it won't collide with the API.

## Nested Objects

Nested objects are automatically ObjectTree instances. Validation works at any depth.

```javascript
const schema = {
  type: 'object',
  properties: {
    address: {
      type: 'object',
      properties: {
        city: { type: 'string' },
        zip:  { type: 'string', pattern: '^[0-9]{5}$' }
      }
    }
  }
}

const user = new ObjectTree({ address: { city: 'NY', zip: '10001' } }, schema)
user.address.city          // 'NY' — nested ObjectTree
user.address.zip = 'bad'   // throws TypeError: pattern mismatch
user.address = { city: 'LA', zip: '90001' }  // replaces entire nested object
```

## Defaults (Lazy)

Schema `default` is the initial value. Missing keys read their default transparently — no explicit call needed.

```javascript
const schema = {
  type: 'object',
  properties: {
    role: { type: 'string', default: 'user' },
    name: { type: 'string' }
  }
}

const t = new ObjectTree({ name: 'Alice' }, schema)
t.role               // 'user' — reads schema default (not in raw data)
'role' in t          // true — default-only keys are visible
Object.keys(t)       // ['name', 'role'] — includes default-only keys

t.$value = { name: 'Bob' }
t.role               // 'user' — default still available

// Materialize defaults into raw data (for schema-unaware consumers)
t.$withDefaults()
t.$value             // { name: 'Bob', role: 'user' } — defaults now in raw data
```

## $ref Resolution

`$ref` resolves through the resolver. Filesystem resolver handles relative paths only; HTTP URIs require a function or object-map resolver.

```javascript
// Filesystem: relative $ref resolved against base dir
const schema = JSON.parse(readFileSync('./schemas/user.json', 'utf8'))
const user = new ObjectTree(data, schema, './schemas/')
// address.$ref: "address.json" → reads ./schemas/address.json

// Function resolver: handle any URI scheme
const user = new ObjectTree(data, schema, (uri) => {
  if (uri.startsWith('http://')) return fetch(uri).then(r => r.json())
  return JSON.parse(readFileSync(uri, 'utf8'))
})
```

Using definitions (internal $ref):

```javascript
const schema = {
  definitions: {
    address: {
      type: 'object',
      properties: { city: { type: 'string' } }
    }
  },
  type: 'object',
  properties: {
    billing:  { $ref: '#/definitions/address' },
    shipping: { $ref: '#/definitions/address' }
  }
}

const order = new ObjectTree({
  billing:  { city: 'NY' },
  shipping: { city: 'LA' }
}, schema)
```

## Composition Methods

Draft-07 logic keywords become callable methods:

```javascript
// oneOf — XOR: exactly one branch matches
const resolved = tree.$oneOf()

// anyOf — OR: all matching branches
const branches = tree.$anyOf()  // returns array

// allOf — AND: merge all sub-schemas
const merged = tree.$allOf()

// if/then/else — conditional
const branch = tree.$ifThen()

// not — negation
const isValid = tree.$notOf()  // returns boolean

// contains — array element match
const hasMatch = tree.$contains()  // returns boolean

// project — keep only schema-defined fields
const projected = tree.$project()

// withDefaults — fill missing defaults
const filled = tree.$withDefaults()
```

## Observer Hook (v0.5.2+)

Global hook for reactive bindings, logging, or instrumentation. Receives schema metadata as 5th parameter.

```javascript
// Signature: fn(op, path, key, val, schema)
// op = 'get' | 'set'
// path = JSON path string (e.g., '$.user.address')
// key = property name
// val = value being read/written
// schema = schema node for the property (undefined if no constraint)

ObjectTree._observer = (op, path, key, val, schema) => {
  const type = schema?.type || 'any'
  const desc = schema?.description || ''
  console.log(`[${op}] ${path}.${key} (${type}): ${desc}`)
}

const user = new ObjectTree({ name: 'Alice', age: 30 }, schema)
user.name  // logs: [get] $.name (string): ...
user.age = 31  // logs: [set] $.age (integer): ...

ObjectTree._observer = null  // disable (zero cost)
```

Use cases:
- **Filter by extension**: Skip properties with `x-observable: false`
- **Format-aware serialization**: Use `schema.format` to serialize dates/emails
- **Type-aware logging**: Log with type/description metadata
- **Reactive bindings**: Build UI bindings, change tracking, etc.

See [examples/observer_schema_context.mjs](examples/observer_schema_context.mjs) for complete examples.

## Standalone Validation

Validate without constructing an ObjectTree:

```javascript
const result = validate(data, schema)
// { valid: true }
// { valid: false, error: '$.age: expected integer, got string' }
```

## Serialization

```javascript
JSON.stringify(user)     // works — toJSON() protocol
user.$toDict()           // plain object, schema-defined fields only
user.$toJSON()           // formatted JSON string
user.$value              // plain object, all fields
```

## See Also

- [Usage Guide](../docs/schema2object-usage.md) — Loader/resolve patterns, dot key support, observer hooks, common mistakes
- [Python implementation](../python/)
- [Rust implementation](../rust/)
- [schema2object root](../)
