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
      try {
        new ObjectTree(tc.data, group.schema)
      } catch {
        threw = true
      }
      const got = !threw  // true = valid, false = invalid

      if (got === tc.valid) {
        filePass++
        totalPass++
      } else {
        fileFail++
        totalFail++
        failures.push(`  [${file}] ${group.description} / ${tc.description}`)
        failures.push(`    data=${JSON.stringify(tc.data)}  expected valid=${tc.valid}  got valid=${got}`)
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
if (JSON.stringify(basic.toDict()) !== JSON.stringify({ name: 'Alice', age: 30 })) {
  throw new Error('toDict should exclude unknown fields')
}
