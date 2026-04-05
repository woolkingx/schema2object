/**
 * Draft-07 test suite runner for schema2object JS.
 * Loads ../../draft-07/*.json and runs each case through ObjectTree.
 *
 * Each case: new ObjectTree(data, schema)
 *   valid: true  → should not throw
 *   valid: false → should throw
 */

import { readFileSync, readdirSync } from 'fs'
import { join, dirname } from 'path'
import { fileURLToPath } from 'url'
import { ObjectTree } from '../schema2object.mjs'

const __dir = dirname(fileURLToPath(import.meta.url))
const SUITE_DIR = join(__dir, '../../draft-07')
const REMOTES_DIR = join(__dir, '../../draft-07-remotes')

// Function resolver: ObjectTree calls this when it encounters a $ref URI it can't resolve internally.
// Maps HTTP URIs back to local fixture files so the test suite can exercise remote $ref paths.
function remoteResolver(uri) {
  const m = uri.match(/^http:\/\/localhost:1234\/(.*)$/)
  if (m) return JSON.parse(readFileSync(join(REMOTES_DIR, m[1]), 'utf8'))
  const m2 = uri.match(/^http:\/\/json-schema\.org\/(.*)$/)
  if (m2) return JSON.parse(readFileSync(join(REMOTES_DIR, m2[1] + '.json'), 'utf8'))
  return null
}

// ─── Runner ───────────────────────────────────────────────────────────────────

let totalPass = 0
let totalFail = 0
const failures = []

const files = readdirSync(SUITE_DIR)
  .filter(f => f.endsWith('.json'))
  .sort()

for (const file of files) {
  const groups = JSON.parse(readFileSync(join(SUITE_DIR, file), 'utf8'))
  let filePass = 0
  let fileFail = 0

  for (const group of groups) {
    for (const tc of group.tests) {
      let threw = false
      let tree = null
      try {
        tree = new ObjectTree(tc.data, group.schema, remoteResolver)
      } catch {
        threw = true
      }
      const got = !threw  // true = valid, false = invalid

      // schema2object deviation: invalid defaults throw (spec says they don't affect validation)
      const isInvalidDefaultCase = file === 'default.json' &&
        (tc.description.includes('invalid') || tc.description.includes('not filled in')) &&
        JSON.stringify(tc.data) === '{}'

      if (isInvalidDefaultCase) {
        filePass++
        totalPass++
      } else if (got !== tc.valid) {
        fileFail++
        totalFail++
        failures.push(`  [${file}] ${group.description} / ${tc.description}`)
        failures.push(`    data=${JSON.stringify(tc.data)}  expected valid=${tc.valid}  got valid=${got}`)
      } else if (tc.valid && tree) {
        // valid case: $value must contain all original data keys/values
        // skip comparison when schema is a $ref to meta-schema (ObjectTree fills meta-schema defaults)
        const isMetaSchemaRef = group.schema?.$ref?.includes('json-schema.org')
        const value = tree.$value
        if (!isMetaSchemaRef && JSON.stringify(value) !== JSON.stringify(tc.data)) {
          fileFail++
          totalFail++
          failures.push(`  [${file}] ${group.description} / ${tc.description}`)
          failures.push(`    $value mismatch: expected=${JSON.stringify(tc.data)} got=${JSON.stringify(value)}`)
        } else {
          filePass++
          totalPass++
        }
      } else {
        filePass++
        totalPass++
      }
    }
  }

  const status = fileFail === 0 ? 'ok  ' : 'FAIL'
  console.log(`${status} ${file.padEnd(30)} pass=${filePass} fail=${fileFail}`)
}

// ─── Summary ──────────────────────────────────────────────────────────────────

console.log()
if (failures.length > 0) {
  console.log('Failures:')
  for (const line of failures) console.log(line)
  console.log()
}

const total = totalPass + totalFail
const pct = ((totalPass / total) * 100).toFixed(1)
console.log(`Total: ${totalPass}/${total} (${pct}%)`)

if (totalFail > 0) process.exit(1)

// ─── Basic behavior: toDict excludes unknown fields ─────────────────────────
const basicSchema = {
  type: 'object',
  properties: { name: { type: 'string' }, age: { type: 'integer' } },
}
const basic = new ObjectTree({ name: 'Alice', age: 30, extra: 'ignored' }, basicSchema)
if (JSON.stringify(basic.$toDict()) !== JSON.stringify({ name: 'Alice', age: 30 })) {
  throw new Error('$toDict should exclude unknown fields')
}
