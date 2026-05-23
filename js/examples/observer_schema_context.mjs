import { ObjectTree } from '../schema2object.mjs'
import {
  loadExampleSchemaWorld,
  printSection,
  readExampleJson,
  resolveExampleDefinition,
} from './support.mjs'

const { rootLoader } = loadExampleSchemaWorld()
const input = {
  ...readExampleJson('user.json'),
  internalToken: 'secret'
}
const { ref, schema: userSchema, loader: userLoader } =
  resolveExampleDefinition(rootLoader, 'User')

const events = []

ObjectTree._observer = (op, path, key, value, schema) => {
  if (schema?.['x-observable'] === false) return

  events.push({
    op,
    path: `${path}.${key}`,
    type: schema?.type || 'any',
    format: schema?.format || null,
    value
  })
}

const user = ObjectTree.from(input, userSchema, userLoader)

user.name
user.email
user.internalToken
user.email = 'alice@example.com'

ObjectTree._observer = null

printSection('schema entry')
console.log('schema file                : js/examples/schema.json')
console.log('schema node                :', ref)

printSection('observer events')
console.log(events)
