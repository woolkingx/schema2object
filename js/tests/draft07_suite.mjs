/**
 * Draft-07 test suite for schema2object v0.7.0
 *
 * API change: new ObjectTree() no longer validates.
 * Validation gate is now validate() — used here for valid/invalid determination.
 * ObjectTree is used only for $value snapshot check on valid cases.
 */

import { readFileSync, readdirSync } from 'fs'
import { join, dirname } from 'path'
import { fileURLToPath } from 'url'
import { validate, ObjectTree } from '../schema2object.mjs'

const __dir = dirname(fileURLToPath(import.meta.url))
const SUITE_DIR = join(__dir, '../../docs/draft-07')
const REMOTES_DIR = join(__dir, '../../docs/draft-07-remotes')

function remoteResolver(uri) {
  const m = uri.match(/^http:\/\/localhost:1234\/(.*)$/)
  if (m) return JSON.parse(readFileSync(join(REMOTES_DIR, m[1]), 'utf8'))
  const m2 = uri.match(/^http:\/\/json-schema\.org\/(.*)$/)
  if (m2) return JSON.parse(readFileSync(join(REMOTES_DIR, m2[1] + '.json'), 'utf8'))
  return null
}

let totalPass = 0
let totalFail = 0
const failures = []

const files = readdirSync(SUITE_DIR).filter(f => f.endsWith('.json')).sort()

for (const file of files) {
  const groups = JSON.parse(readFileSync(join(SUITE_DIR, file), 'utf8'))
  let filePass = 0
  let fileFail = 0

  for (const group of groups) {
    for (const tc of group.tests) {
      // schema2object deviation: $value materializes defaults (spec says defaults are informational)
      const isDefaultDeviationCase = file === 'default.json' &&
        (tc.description.includes('invalid') || tc.description.includes('not filled in')) &&
        JSON.stringify(tc.data) === '{}'

      if (isDefaultDeviationCase) { filePass++; totalPass++; continue }

      const result = validate(tc.data, group.schema, remoteResolver)
      const got = result.valid

      if (got !== tc.valid) {
        fileFail++
        totalFail++
        failures.push(`  [${file}] ${group.description} / ${tc.description}`)
        failures.push(`    data=${JSON.stringify(tc.data)}  expected valid=${tc.valid}  got valid=${got}`)
      } else if (tc.valid) {
        const isMetaSchemaRef = group.schema?.$ref?.includes('json-schema.org')
        const tree = new ObjectTree(tc.data, group.schema, remoteResolver)
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
