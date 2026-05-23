# TypeScript: JSON Schema Validation with Ajv

Frontend and backend can share the same schema file. Ajv validates against it at runtime.

This is projection guidance only. The active `schema2object` runtime owned by this repository is JavaScript under `js/`; this chapter does not define a TypeScript package implementation.

## Chapter Role

| Field | Value |
|---|---|
| Input artifact | JSON Schema that a TypeScript consumer must validate or type |
| Output artifact | Generated types plus Ajv validator projection |
| Owner | TypeScript consumer project, not this repository runtime |
| Gate | Generated type and validator trace back to the schema file |
| Failure route | Return to the schema; do not hand-write a competing interface truth |

## Install

```bash
npm install ajv ajv-formats
npm install -D json-schema-to-typescript
```

---

## Basic Usage

### Inline schema

```typescript
import Ajv from 'ajv'
import addFormats from 'ajv-formats'

const ajv = new Ajv()
addFormats(ajv)

const schema = {
  type: 'object',
  properties: {
    email: { type: 'string', format: 'email' },
    name:  { type: 'string', minLength: 1 }
  },
  required: ['email', 'name']
}

const validate = ajv.compile(schema)

function validateUser(data: unknown): data is User {
  if (!validate(data)) {
    console.error(validate.errors)
    return false
  }
  return true
}
```

### External schema file

```typescript
import userSchema from '../schemas/user.schema.json'

const validateUser = ajv.compile(userSchema)

export function isValidUser(data: unknown): data is User {
  return validateUser(data)
}

export function assertValidUser(data: unknown): asserts data is User {
  if (!validateUser(data)) {
    throw new Error(`Invalid user: ${JSON.stringify(validateUser.errors)}`)
  }
}
```

---

## Type Generation

Generate TypeScript types from the schema — no hand-written interfaces.

```bash
json2ts schemas/user.schema.json > src/types/user.ts
```

**package.json**:
```json
{
  "scripts": {
    "generate-types": "json2ts schemas/*.schema.json -o src/types/"
  }
}
```

Generated `src/types/user.ts`:
```typescript
export interface User {
  email: string
  name: string
  age?: number
  roles?: ('admin' | 'user' | 'guest')[]
}
```

Schema changes → run `generate-types` → types stay in sync. No manual interface updates.

---

## API Integration

### Fetch with validation

```typescript
import { assertValidUser } from './validators'
import type { User } from './types/user'

async function fetchUser(id: string): Promise<User> {
  const response = await fetch(`/api/users/${id}`)
  const data = await response.json()
  assertValidUser(data)
  return data
}
```

### Form validation

```typescript
function handleSubmit(formData: FormData) {
  const userData = Object.fromEntries(formData)

  if (!validateUser(userData)) {
    const errors = validateUser.errors || []
    showErrors(errors.map(e => `${e.instancePath}: ${e.message}`))
    return
  }

  api.createUser(userData as User)
}
```

---

## React Hook

```typescript
import { useState } from 'react'

export function useValidation<T>(validator: (data: unknown) => boolean) {
  const [errors, setErrors] = useState<string[]>([])

  const validate = (data: unknown): data is T => {
    const valid = validator(data)
    if (!valid && validator.errors) {
      setErrors(validator.errors.map(e => e.message || ''))
    } else {
      setErrors([])
    }
    return valid
  }

  return { validate, errors }
}

// Usage
function UserForm() {
  const { validate, errors } = useValidation<User>(validateUser)

  const handleSubmit = (data: unknown) => {
    if (validate(data)) {
      api.createUser(data)
    }
  }

  return (
    <form onSubmit={handleSubmit}>
      {errors.map(err => <p className="error">{err}</p>)}
    </form>
  )
}
```

---

## Schema Registry

Register multiple schemas by `$id` for type-safe access across the app.

```typescript
import Ajv from 'ajv'
import userSchema from '../schemas/user.schema.json'
import sessionSchema from '../schemas/session.schema.json'

const ajv = new Ajv({ schemas: [userSchema, sessionSchema] })

const validateUser    = ajv.getSchema<User>('https://api.example.com/schemas/user.json')
const validateSession = ajv.getSchema<Session>('https://api.example.com/schemas/session.json')

export const isUser    = (data: unknown): data is User    => validateUser!(data)
export const isSession = (data: unknown): data is Session => validateSession!(data)
```

---

## Testing

```typescript
import { describe, it, expect } from 'vitest'

describe('User schema validation', () => {
  it('accepts a valid user', () => {
    expect(validateUser({ email: 'user@example.com', name: 'Alice' })).toBe(true)
  })

  it('rejects an invalid email', () => {
    expect(validateUser({ email: 'not-an-email', name: 'Alice' })).toBe(false)
    expect(validateUser.errors).toBeDefined()
  })
})
```

---

## CI/CD Integration

Adapt this to the owning project's CI.

Key steps:
1. Validate schema syntax with `ajv-cli`
2. Generate TypeScript types
3. Type-check with `tsc`
4. Run tests

---

## Cross-Language Projection

The same `user.schema.json` remains the source of truth, but each language projects it into its native strongest shape:

- TypeScript frontend: generated types plus Ajv validator
- Rust backend: Rust Projection Guide — `rust.md`
- Python services: Python Projection Guide — `python.md`

These are documentation projections, not active runtime implementations in this repository.

---

## References

- Ajv: https://ajv.js.org/
- json-schema-to-typescript: https://github.com/bcherny/json-schema-to-typescript
- JSON Schema spec: https://json-schema.org/
- SDD methodology: https://codeberg.org/woolkingx/schema-driven-development
