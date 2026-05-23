import { ObjectTree } from '../schema2object.mjs'
import {
  loadExampleSchemaWorld,
  printSection,
  readExampleJson,
  resolveExampleDefinition,
} from './support.mjs'

const { rootLoader } = loadExampleSchemaWorld()
const flatRule = readExampleJson('rule.flat.json')
const { ref, schema: ruleSchema, loader: ruleLoader } =
  resolveExampleDefinition(rootLoader, 'Rule')

printSection('schema entry')
console.log('schema file                : js/examples/schema.json')
console.log('schema node                :', ref)

printSection('flat authoring data')
console.log(flatRule)

const cursorSchema = { type: 'object', properties: ruleSchema.properties }
const draft = new ObjectTree({}, cursorSchema, ruleLoader)

for (const [key, value] of Object.entries(flatRule)) {
  draft[key] = value
}

printSection('expanded runtime shape')
console.log(draft.$toDict())
console.log('draft.match.command.name   :', draft.match.command.name)

printSection('final boundary validation')
const rule = ObjectTree.from(draft.$toDict(), ruleSchema, ruleLoader)
console.log('ObjectTree.from(...)       : ok')
console.log('rule.action                :', rule.action)

printSection('per-field validation still applies')
try {
  draft['match.command.name'] = ''
} catch (e) {
  console.log('empty command name throws  :', e.message)
}
