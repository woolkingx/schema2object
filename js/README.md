# schema2object — JavaScript

> Coming soon.

JSON Schema as JS object definition — structure maps to properties, logic maps to methods.

JSON came from JavaScript objects. This is the native implementation.

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

const user = new ObjectTree({ email: 'alice@example.com', age: 30 }, { schema })

user.email        // 'alice@example.com'
user.one_of()     // XOR branch dispatch
user.if_then()    // conditional branch
user.project()    // keep only schema-defined fields
```

## See Also

- [Python implementation](../python/)
- [schema2object root](../)
