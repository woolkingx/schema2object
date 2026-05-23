import { readFileSync } from 'fs'
import { Loader } from '../schema2object.mjs'

const here = new URL('.', import.meta.url)
let sectionCount = 0

export function readExampleJson(file) {
  return JSON.parse(readFileSync(new URL(file, here), 'utf8'))
}

export function loadExampleSchemaWorld() {
  const rootSchema = readExampleJson('schema.json')
  const rootLoader = new Loader(rootSchema)
  return { rootSchema, rootLoader }
}

export function resolveExampleDefinition(rootLoader, name) {
  const ref = `#/definitions/${name}`
  const { node, loader } = rootLoader.resolve(ref)
  return { ref, schema: node, loader }
}

export function printSection(title) {
  const prefix = sectionCount === 0 ? '' : '\n'
  sectionCount++
  console.log(`${prefix}--- ${title} ---`)
}
