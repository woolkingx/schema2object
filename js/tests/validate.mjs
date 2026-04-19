/**
 * Tests for the validate() export.
 */
import { validate, ObjectTree } from '../schema2object.mjs'

let pass = 0
let fail = 0

function assert(label, condition) {
  if (condition) { pass++; console.log(`ok   ${label}`) }
  else           { fail++; console.log(`FAIL ${label}`) }
}

// ─── valid cases ─────────────────────────────────────────────────────────────

const r1 = validate(42, { type: 'integer' })
assert('integer valid',        r1.valid === true)
assert('integer no error',     r1.error === undefined)

const r2 = validate('hello', { type: 'string', minLength: 3 })
assert('string valid',         r2.valid === true)

const r3 = validate({ name: 'Alice', age: 30 }, {
  type: 'object',
  properties: { name: { type: 'string' }, age: { type: 'integer' } },
  required: ['name'],
})
assert('object valid',         r3.valid === true)

// ─── invalid cases ────────────────────────────────────────────────────────────

const r4 = validate('hello', { type: 'integer' })
assert('string as integer invalid',  r4.valid === false)
assert('string as integer has error', typeof r4.error === 'string')

const r5 = validate(5, { type: 'integer', minimum: 10 })
assert('below minimum invalid',      r5.valid === false)

const r6 = validate({ age: 'not-a-number' }, {
  type: 'object',
  properties: { age: { type: 'integer' } },
})
assert('wrong property type invalid', r6.valid === false)

const r7 = validate({}, {
  type: 'object',
  required: ['name'],
})
assert('missing required invalid',    r7.valid === false)

// ─── boolean schema ───────────────────────────────────────────────────────────

assert('schema true valid',   validate(42,   true).valid === true)
assert('schema false invalid', validate(42,  false).valid === false)

// ─── $ref via object resolver ─────────────────────────────────────────────────

const addrSchema = { type: 'object', properties: { city: { type: 'string' } }, required: ['city'] }
const personSchema = {
  type: 'object',
  properties: { address: { '$ref': 'addr' } },
}
const r8 = validate({ address: { city: 'Taipei' } }, personSchema, { addr: addrSchema })
assert('$ref via map valid',   r8.valid === true)

const r9 = validate({ address: { city: 123 } }, personSchema, { addr: addrSchema })
assert('$ref via map invalid', r9.valid === false)

// ─── _observer: static hook ──────────────────────────────────────────────────

// get triggers observer for primitive, object, array
{
  const log = []
  ObjectTree._observer = (op, path, key, val) => log.push({ op, path, key, val })

  const schema = {
    type: 'object',
    properties: {
      name: { type: 'string' },
      age: { type: 'integer' },
      tags: { type: 'array', items: { type: 'string' } },
      addr: { type: 'object', properties: { city: { type: 'string' } } },
    },
  }
  const tree = new ObjectTree({ name: 'Alice', age: 30, tags: ['a', 'b'], addr: { city: 'Taipei' } }, schema)

  log.length = 0
  const _name = tree.name
  assert('observer get primitive',    log.length === 1 && log[0].op === 'get' && log[0].key === 'name' && log[0].val === 'Alice')

  log.length = 0
  const _addr = tree.addr
  assert('observer get object',       log.length === 1 && log[0].op === 'get' && log[0].key === 'addr')

  log.length = 0
  const _tags = tree.tags
  assert('observer get array',        log.length === 1 && log[0].op === 'get' && log[0].key === 'tags' && Array.isArray(log[0].val))

  // set triggers observer
  log.length = 0
  tree.name = 'Bob'
  assert('observer set',              log.length === 1 && log[0].op === 'set' && log[0].key === 'name' && log[0].val === 'Bob')

  // null disables — zero events
  ObjectTree._observer = null
  log.length = 0
  const _age = tree.age
  assert('observer null no events',   log.length === 0)

  // cached object access does not re-trigger (cache hit returns before observer)
  ObjectTree._observer = (op, path, key, val) => log.push({ op, path, key, val })
  log.length = 0
  const _addr2 = tree.addr
  assert('observer cached no re-emit', log.length === 0)

  ObjectTree._observer = null
}

// ─── summary ──────────────────────────────────────────────────────────────────

console.log()
console.log(`Total: ${pass}/${pass + fail}`)
if (fail > 0) process.exit(1)
