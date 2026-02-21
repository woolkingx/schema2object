import { ObjectTree } from './schema2object.mjs'

const tree = new ObjectTree({
  type: 'object',
  properties: {
    name: { type: 'string', minLength: 1, 'x-docs': 'Display name' },
    age:  { type: 'integer', minimum: 0, maximum: 100, 'x-tests': ['min', 'max'] },
    role: { type: 'string', enum: ['admin', 'user', 'guest'], 'x-meta': { ui: 'select' } },
  },
  required: ['name'],
})

// ── meta (explicit API) ───────────────────────────────────────────────────────
console.log('--- meta ---')
console.log('schema.type                :', tree.getSchema().type)
console.log('schema.age.minimum         :', tree.getSchema('age')?.minimum)
console.log('schema.age.maximum         :', tree.getSchema('age')?.maximum)
console.log('schema.role.enum           :', tree.getSchema('role')?.enum)
console.log('x.name                     :', tree.getExtensions('name'))
console.log('x.age                      :', tree.getExtensions('age'))
console.log('x.role                     :', tree.getExtensions('role'))

// ── runtime instance ──────────────────────────────────────────────────────────
console.log('\n--- runtime ---')
tree.name = 'Alice'
console.log('tree.name = "Alice"        : ok')

tree.age = 30
console.log('tree.age = 30              : ok')

tree.role = 'admin'
console.log('tree.role = "admin"        : ok')

console.log('tree.name                  :', tree.name)
console.log('tree.age                   :', tree.age)
console.log('tree.toDict()              :', tree.toDict())
tree.extra = 'ignored'
console.log('tree.extra                 :', tree.extra)
console.log('tree.toDict() (no extra)   :', tree.toDict())

// ── constraints ───────────────────────────────────────────────────────────────
console.log('\n--- constraints ---')
try { tree.age = 150 } catch (e) { console.log('tree.age = 150 throws      :', e.message) }
try { tree.role = 'root' } catch (e) { console.log('tree.role = "root" throws  :', e.message) }
try { tree.name = '' } catch (e) { console.log('tree.name = "" throws      :', e.message) }

// ── schema still accessible ───────────────────────────────────────────────────
console.log('\n--- schema() ---')
console.log('tree.schema.type           :', tree.schema.type)
