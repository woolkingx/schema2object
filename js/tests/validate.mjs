/**
 * Tests for schema2object v0.7.0: validate() and ObjectTree.from()
 *
 * Key API change from v0.6.1:
 *   - new ObjectTree() no longer validates — it is a cursor constructor
 *   - validate() is now independent (no ObjectTree construction)
 *   - ObjectTree.from() is the boundary entry point — validates then returns cursor
 */
import { validate, ObjectTree } from '../schema2object.mjs'

let pass = 0
let fail = 0

function assert(label, condition) {
  if (condition) { pass++; console.log(`ok   ${label}`) }
  else           { fail++; console.log(`FAIL ${label}`) }
}

function throws(label, fn, msgFragment) {
  try { fn(); fail++; console.log(`FAIL ${label} (expected throw)`) }
  catch (e) {
    const ok = msgFragment ? e.message.includes(msgFragment) : true
    if (ok) { pass++; console.log(`ok   ${label}`) }
    else    { fail++; console.log(`FAIL ${label} (got: ${e.message})`) }
  }
}

// ─── validate(): valid cases ──────────────────────────────────────────────────

const r1 = validate(42, { type: 'integer' })
assert('validate integer valid',       r1.valid === true)
assert('validate no error field',      r1.error === undefined)

const r2 = validate('hello', { type: 'string', minLength: 3 })
assert('validate string valid',        r2.valid === true)

const r3 = validate({ name: 'Alice', age: 30 }, {
  type: 'object',
  properties: { name: { type: 'string' }, age: { type: 'integer' } },
  required: ['name'],
})
assert('validate object valid',        r3.valid === true)

assert('validate schema true',         validate(42,    true).valid === true)
assert('validate schema false',        validate(42,   false).valid === false)
assert('validate null data',           validate(null, { type: 'null' }).valid === true)

// ─── validate(): invalid cases ────────────────────────────────────────────────

const r4 = validate('hello', { type: 'integer' })
assert('validate type mismatch',       r4.valid === false)
assert('validate has error string',    typeof r4.error === 'string')

const r5 = validate(5, { type: 'integer', minimum: 10 })
assert('validate below minimum',       r5.valid === false)

const r6 = validate({ age: 'not-a-number' }, {
  type: 'object',
  properties: { age: { type: 'integer' } },
})
assert('validate wrong property type', r6.valid === false)

const r7 = validate({}, { type: 'object', required: ['name'] })
assert('validate missing required',    r7.valid === false)

const r8 = validate('badnope', { type: 'string', pattern: '^[^@]+@[^@]+$' })
assert('validate pattern mismatch',    r8.valid === false)

// ─── validate(): $ref via resolver ───────────────────────────────────────────

const addrSchema = { type: 'object', properties: { city: { type: 'string' } }, required: ['city'] }
const personSchema = { type: 'object', properties: { address: { '$ref': 'addr' } } }

const r9 = validate({ address: { city: 'Taipei' } }, personSchema, { addr: addrSchema })
assert('validate $ref map valid',      r9.valid === true)

const r10 = validate({ address: { city: 123 } }, personSchema, { addr: addrSchema })
assert('validate $ref map invalid',    r10.valid === false)

// ─── ObjectTree.from(): happy path ────────────────────────────────────────────

const schema = {
  type: 'object',
  properties: {
    name: { type: 'string', default: 'anon' },
    age:  { type: 'integer', minimum: 0 },
  },
  required: ['age'],
}

const t1 = ObjectTree.from({ age: 25 }, schema)
assert('from() returns ObjectTree',    t1 instanceof ObjectTree)
assert('from() data access',           t1.age === 25)
assert('from() default fill',          t1.name === 'anon')

const t2 = ObjectTree.from({ age: 30, name: 'Bob' }, schema)
assert('from() explicit value',        t2.name === 'Bob')

// nested object
const nestedSchema = {
  type: 'object',
  properties: {
    addr: {
      type: 'object',
      properties: { city: { type: 'string' } },
      required: ['city'],
    },
  },
  required: ['addr'],
}
const t3 = ObjectTree.from({ addr: { city: 'Tokyo' } }, nestedSchema)
assert('from() nested access',         t3.addr.city === 'Tokyo')
assert('from() nested is ObjectTree',  t3.addr instanceof ObjectTree)

// ─── ObjectTree.from(): invalid input throws ──────────────────────────────────

throws('from() throws on type mismatch',
  () => ObjectTree.from('not-an-object', schema))

throws('from() throws on missing required',
  () => ObjectTree.from({}, schema))

throws('from() throws on constraint violation',
  () => ObjectTree.from({ age: -1 }, schema))

// ─── Boundary separation: new ObjectTree() does NOT validate ─────────────────
// This is the key API difference from v0.6.1.

{
  // Invalid data — new ObjectTree() should not throw (no validation at construction)
  let didThrow = false
  try { new ObjectTree({ age: -999 }, schema) }
  catch { didThrow = true }
  assert('new ObjectTree() no throw on invalid', !didThrow)
}

{
  // new ObjectTree() with invalid data: cursor still works, raw data is accessible
  const raw = { age: -999 }
  const t = new ObjectTree(raw, schema)
  assert('new ObjectTree() cursor accesses data', t.age === -999)
}

// ─── from() + $-methods on resulting cursor ──────────────────────────────────

const oneOfSchema = {
  oneOf: [
    { type: 'object', properties: { kind: { const: 'a' }, x: { type: 'number' } }, required: ['kind'] },
    { type: 'object', properties: { kind: { const: 'b' }, y: { type: 'string' } }, required: ['kind'] },
  ],
}
const t4 = ObjectTree.from({ kind: 'b', y: 'hi' }, oneOfSchema)
const branch = t4.$oneOf()
assert('from() $oneOf() returns ObjectTree', branch instanceof ObjectTree)
assert('from() $oneOf() correct branch',     branch.y === 'hi')

const t5 = ObjectTree.from({ age: 30 }, schema)
const wd = t5.$withDefaults()
assert('from() $withDefaults() fills name',  wd.name === 'anon')
assert('from() $withDefaults() age present', wd.age === 30)

const t6 = ObjectTree.from({ age: 25, name: 'Alice' }, schema)
const val = t6.$value
assert('from() $value round-trips',          val.age === 25 && val.name === 'Alice')

// ─── from() with $ref resolver ───────────────────────────────────────────────

const t7 = ObjectTree.from({ address: { city: 'Taipei' } }, personSchema, { addr: addrSchema })
assert('from() $ref valid creates cursor',   t7 instanceof ObjectTree)
assert('from() $ref nested city',            t7.address.city === 'Taipei')

throws('from() $ref invalid throws',
  () => ObjectTree.from({ address: { city: 123 } }, personSchema, { addr: addrSchema }))

// ─── summary ──────────────────────────────────────────────────────────────────

console.log()
console.log(`Total: ${pass}/${pass + fail}`)
if (fail > 0) process.exit(1)
