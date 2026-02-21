import { ObjectTree } from './schema2object.mjs'

const tree = new ObjectTree({
  type: 'object',
  properties: {
    name: { type: 'string', minLength: 1 },
    age:  { type: 'integer', minimum: 0, maximum: 100 },
    role: { type: 'string', enum: ['admin', 'user', 'guest'] },
  },
  required: ['name'],
})

// ── meta ──────────────────────────────────────────────────────────────────────
console.log('--- meta ---')
console.log('tree.type                  :', tree.type)
console.log('tree.properties.age.minimum:', tree.properties.age.minimum)
console.log('tree.properties.age.maximum:', tree.properties.age.maximum)
console.log('tree.properties.role.enum  :', tree.properties.role.enum)

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

// ── constraints ───────────────────────────────────────────────────────────────
console.log('\n--- constraints ---')
try { tree.age = 150 } catch (e) { console.log('tree.age = 150 throws      :', e.message) }
try { tree.role = 'root' } catch (e) { console.log('tree.role = "root" throws  :', e.message) }
try { tree.name = '' } catch (e) { console.log('tree.name = "" throws      :', e.message) }

// ── schema still accessible ───────────────────────────────────────────────────
console.log('\n--- schema() ---')
console.log('tree.schema().type         :', tree.schema().type)
