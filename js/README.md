# schema2object — JavaScript

> Coming soon.

JSON Schema defines the object class — structure + constraints + behavioral logic.

Best practice: treat JSON Schema the same way you define a JS object class.

## Planned API

```javascript
import { ObjectTree } from 'schema2object'

const schema = {
  type: 'object',
  properties: {
    email: { type: 'string', format: 'email' },
    age:   { type: 'integer', minimum: 0 }
  },
  required: ['email']
}

const user = new ObjectTree(schema)

user.email = 'alice@example.com'
user.age = 30
user.email        // 'alice@example.com'
user.getSchema('age')  // { type: 'integer', minimum: 0 }
user.getExtensions('age') // { 'x-docs': '...' }
user.oneOf(...)   // XOR branch dispatch
user.ifThen(...)  // conditional branch
user.project(...) // keep only schema-defined fields
user.extra = 'ignored' // allowed at runtime
user.toDict()    // schema-defined fields only
```

## See Also

- [Python implementation](../python/)
- [schema2object root](../)
