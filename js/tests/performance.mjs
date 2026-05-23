/**
 * Performance smoke benchmark for the current schema2object mainline.
 *
 * This is not a pass/fail threshold test. It preserves a runnable performance
 * surface so runtime changes can be compared against the same operations:
 * schema boundary, pure cursor construction, validation, access, mutation, and
 * materialization.
 */
import { ObjectTree, validate } from '../schema2object.mjs'

const userSchema = {
  type: 'object',
  properties: {
    name: { type: 'string', default: 'anon' },
    email: { type: 'string', pattern: '^[^@]+@[^@]+$' },
    age: { type: 'integer', minimum: 0, maximum: 200 },
    tags: { type: 'array', items: { type: 'string' }, minItems: 1 },
    address: {
      type: 'object',
      properties: {
        city: { type: 'string' },
        zip: { type: 'string', pattern: '^\\d{5}$' },
        street: { type: 'string', default: '123 Main St' },
      },
      required: ['city', 'zip'],
      additionalProperties: false,
    },
    scores: {
      type: 'array',
      items: {
        type: 'object',
        properties: {
          subject: { type: 'string' },
          value: { type: 'number', minimum: 0, maximum: 100 },
        },
        required: ['subject', 'value'],
        additionalProperties: false,
      },
    },
  },
  required: ['email', 'age', 'address'],
  additionalProperties: false,
}

const oneOfSchema = {
  oneOf: [
    { type: 'object', properties: { kind: { const: 'a' }, x: { type: 'number' } }, required: ['kind'] },
    { type: 'object', properties: { kind: { const: 'b' }, y: { type: 'string' } }, required: ['kind'] },
    { type: 'object', properties: { kind: { const: 'c' }, z: { type: 'boolean' } }, required: ['kind'] },
  ],
}

const validUser = {
  email: 'test@example.com',
  age: 30,
  tags: ['dev', 'js'],
  address: { city: 'Tokyo', zip: '12345' },
  scores: [
    { subject: 'math', value: 95 },
    { subject: 'eng', value: 88 },
    { subject: 'sci', value: 72 },
  ],
}

const invalidUser = { email: 'bad', age: -1, address: { city: 'Tokyo', zip: 'x' } }

function bench(label, fn, iters = 5000) {
  for (let i = 0; i < 300; i++) fn()
  const t0 = performance.now()
  for (let i = 0; i < iters; i++) fn()
  const elapsed = performance.now() - t0
  const usPerOp = (elapsed / iters) * 1000
  console.log(`${label.padEnd(40)} ${usPerOp.toFixed(2).padStart(8)} us/op  (${iters} iters)`)
  return usPerOp
}

console.log('=== schema2object current-mainline performance ===\n')

bench('boundary: ObjectTree.from(valid)', () => {
  ObjectTree.from(validUser, userSchema)
})

bench('cursor: new ObjectTree(already legal)', () => {
  new ObjectTree(validUser, userSchema)
})

bench('validate: valid data', () => {
  validate(validUser, userSchema)
})

bench('validate: invalid data', () => {
  validate(invalidUser, userSchema)
})

bench('access: nested default read', () => {
  const tree = new ObjectTree(validUser, userSchema)
  void tree.name
  void tree.address.street
})

bench('access: array cursor read', () => {
  const tree = new ObjectTree(validUser, userSchema)
  void tree.scores[0].subject
  void tree.scores[1].value
})

bench('materialize: $withDefaults()', () => {
  const tree = ObjectTree.from(validUser, userSchema)
  tree.$withDefaults()
})

bench('snapshot: $value', () => {
  const tree = new ObjectTree(validUser, userSchema)
  void tree.$value
})

bench('logic: $oneOf()', () => {
  const tree = ObjectTree.from({ kind: 'b', y: 'hello' }, oneOfSchema)
  tree.$oneOf()
})

bench('mutation: setter validation', () => {
  const tree = ObjectTree.from({ ...validUser }, userSchema)
  tree.age = 31
})

console.log('\n=== done ===')
