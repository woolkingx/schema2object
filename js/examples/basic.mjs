import { ObjectTree, validate } from '../schema2object.mjs'
import {
  loadExampleSchemaWorld,
  printSection,
  readExampleJson,
  resolveExampleDefinition,
} from './support.mjs'

const { rootLoader } = loadExampleSchemaWorld()
const input = readExampleJson('user.json')
const { ref, schema: userSchema, loader: userLoader } =
  resolveExampleDefinition(rootLoader, 'User')

printSection('schema entry')
console.log('schema file                : js/examples/schema.json')
console.log('schema node                :', ref)

printSection('boundary gate')
console.log('validate(input, schema)    :', validate(input, userSchema, userLoader))

const user = ObjectTree.from(input, userSchema, userLoader)
console.log('ObjectTree.from(...)       : ok')

printSection('live cursor access')
console.log('user.name                  :', user.name)
console.log('user.email                 :', user.email)
console.log('user.role                  :', user.role)

printSection('explicit materialization')
console.log('user.$withDefaults().$value:', user.$withDefaults().$value)

printSection('schema metadata')
console.log('user.$getSchema("email")   :', user.$getSchema('email'))

printSection('mutation after boundary')
user.age = 31
console.log('user.age = 31              : ok')
try {
  user.age = -1
} catch (e) {
  console.log('user.age = -1 throws       :', e.message)
}

printSection('invalid external data')
try {
  ObjectTree.from({ ...input, email: 42 }, userSchema, userLoader)
} catch (e) {
  console.log('ObjectTree.from(...) throws:', e.message)
}
