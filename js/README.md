# schema2object — JavaScript

JSON Schema defines the object class — structure + constraints + behavioral logic.

Best practice: treat JSON Schema the same way you define a JS object class.

## Core Semantics

- Schema defines the class; data is the instance
- Mutation validates — at any nesting depth (nested objects are ObjectTree instances)
- Unknown fields may exist at runtime (native JS behavior)
- `$toDict()` exports schema-defined fields only
- Schema extensions live in `x-*` (e.g. `x-docs`, `x-tests`)
- All API methods use `$` prefix — data properties don't. No naming collision.

## API

```javascript
import { ObjectTree } from 'schema2object'

const schema = {
  type: 'object',
  properties: {
    email: { type: 'string', format: 'email', 'x-docs': 'User email' },
    age:   { type: 'integer', minimum: 0 }
  },
  required: ['email']
}

const user = new ObjectTree({}, schema)

user.email = 'alice@example.com'
user.age = 30
user.email               // 'alice@example.com' — data property (no prefix)
user.$getSchema('age')   // { type: 'integer', minimum: 0 }
user.$getExtensions('email') // { 'x-docs': 'User email' }
user.$oneOf()            // XOR branch dispatch
user.$ifThen()           // conditional branch
user.$project()          // keep only schema-defined fields
user.$withDefaults()     // apply schema defaults (object only)

user.$value              // full data as plain object
user.$toDict()           // schema-defined fields only
JSON.stringify(user)     // works — toJSON() protocol
```

## See Also

- [Python implementation](../python/)
- [Rust implementation](../rust/)
- [schema2object root](../)
