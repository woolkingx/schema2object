/**
 * schema2object — JavaScript
 * JSON Schema IS the object class: meta (schema structure) + runtime (instance with internalized constraints).
 */

// ─── Shared resources ─────────────────────────────────────────────────────────

// Cached segmenter for grapheme-length counting
const _segmenter = new Intl.Segmenter()
const _graphemeLen = s => [..._segmenter.segment(s)].length

// RegExp cache for patternProperties
const _reCache = new Map()
const _re = pat => { if (!_reCache.has(pat)) _reCache.set(pat, new RegExp(pat)); return _reCache.get(pat) }

// ─── Deep equality ────────────────────────────────────────────────────────────

// Strict: boolean !== number even if numerically equal
function _deepEqual(a, b) {
  if (a === b) return true
  if (typeof a !== typeof b) return false
  if (a === null || b === null) return a === b
  if (Array.isArray(a) && Array.isArray(b)) {
    if (a.length !== b.length) return false
    return a.every((v, i) => _deepEqual(v, b[i]))
  }
  if (typeof a === 'object' && typeof b === 'object') {
    const ka = Object.keys(a).sort(), kb = Object.keys(b).sort()
    if (ka.length !== kb.length) return false
    return ka.every((k, i) => k === kb[i] && _deepEqual(a[k], b[k]))
  }
  return false
}

// ─── Deep schema merge (for allOf) ───────────────────────────────────────────

function _deepMerge(a, b) {
  if (typeof a !== 'object' || a === null) return b
  if (typeof b !== 'object' || b === null) return b
  const out = { ...a }
  for (const [k, v] of Object.entries(b)) {
    if (k === 'properties' && out.properties) {
      out.properties = { ...out.properties }
      for (const [pk, pv] of Object.entries(v))
        out.properties[pk] = out.properties[pk] ? _deepMerge(out.properties[pk], pv) : pv
    } else if (k === 'required' && Array.isArray(out.required) && Array.isArray(v)) {
      out.required = [...new Set([...out.required, ...v])]
    } else if (typeof v === 'object' && v !== null && !Array.isArray(v) && typeof out[k] === 'object') {
      out[k] = _deepMerge(out[k], v)
    } else {
      out[k] = v
    }
  }
  return out
}

// ─── Core validator ───────────────────────────────────────────────────────────

const TYPE_CHECKS = {
  string:  v => typeof v === 'string',
  integer: v => Number.isInteger(v),
  number:  v => typeof v === 'number' && !Number.isNaN(v),
  boolean: v => typeof v === 'boolean',
  array:   v => Array.isArray(v),
  object:  v => v !== null && typeof v === 'object' && !Array.isArray(v),
  null:    v => v === null,
}

function validateType(value, schema, path, root) {
  // Boolean schema: true = accept all, false = reject all
  if (schema === true) return
  if (schema === false) throw new TypeError(`${path}: schema is false`)

  // $ref: resolve before setting root default so top-level $ref works
  if (schema.$ref !== undefined) {
    if (!root) throw new TypeError(`${path}: $ref requires root schema`)
    return validateType(value, _resolveRef(schema.$ref, root), path, root)
  }

  if (!root) root = schema

  const {
    type, const: constVal,
    minimum, maximum, exclusiveMinimum, exclusiveMaximum, multipleOf,
    minLength, maxLength, pattern,
    minItems, maxItems, uniqueItems, items, additionalItems, contains,
    minProperties, maxProperties, required, properties, additionalProperties, patternProperties,
    dependencies,
    enum: enumVals,
    not, oneOf, anyOf, allOf,
    if: ifSchema, then: thenSchema, else: elseSchema,
  } = schema

  // type
  if (type !== undefined) {
    const types = Array.isArray(type) ? type : [type]
    if (!types.some(t => TYPE_CHECKS[t]?.(value)))
      throw new TypeError(`${path}: expected ${types.join('|')}, got ${value === null ? 'null' : typeof value}`)
  }

  // const / enum
  if (constVal !== undefined && !_deepEqual(value, constVal))
    throw new TypeError(`${path}: must equal const ${JSON.stringify(constVal)}`)
  if (enumVals !== undefined && !enumVals.some(e => _deepEqual(e, value)))
    throw new TypeError(`${path}: not in enum`)

  // number constraints
  if (typeof value === 'number') {
    if (minimum !== undefined && value < minimum)
      throw new RangeError(`${path}: ${value} < minimum ${minimum}`)
    if (maximum !== undefined && value > maximum)
      throw new RangeError(`${path}: ${value} > maximum ${maximum}`)
    if (exclusiveMinimum !== undefined && value <= exclusiveMinimum)
      throw new RangeError(`${path}: ${value} <= exclusiveMinimum ${exclusiveMinimum}`)
    if (exclusiveMaximum !== undefined && value >= exclusiveMaximum)
      throw new RangeError(`${path}: ${value} >= exclusiveMaximum ${exclusiveMaximum}`)
    if (multipleOf !== undefined) {
      const q = value / multipleOf
      if (!isFinite(q) || Math.abs(q - Math.round(q)) > 1e-9)
        throw new RangeError(`${path}: ${value} not multipleOf ${multipleOf}`)
    }
  }

  // string constraints (grapheme length)
  if (typeof value === 'string') {
    const len = _graphemeLen(value)
    if (minLength !== undefined && len < minLength)
      throw new RangeError(`${path}: length ${len} < minLength ${minLength}`)
    if (maxLength !== undefined && len > maxLength)
      throw new RangeError(`${path}: length ${len} > maxLength ${maxLength}`)
    if (pattern !== undefined && !_re(pattern).test(value))
      throw new TypeError(`${path}: does not match pattern ${pattern}`)
  }

  // array constraints
  if (Array.isArray(value)) {
    if (minItems !== undefined && value.length < minItems)
      throw new RangeError(`${path}: length ${value.length} < minItems ${minItems}`)
    if (maxItems !== undefined && value.length > maxItems)
      throw new RangeError(`${path}: length ${value.length} > maxItems ${maxItems}`)
    if (uniqueItems) {
      // fast path: primitives only
      const allPrimitive = value.every(v => v === null || typeof v !== 'object')
      if (allPrimitive) {
        if (new Set(value.map(v => `${typeof v}:${v}`)).size !== value.length)
          throw new TypeError(`${path}: duplicate items`)
      } else {
        for (let i = 0; i < value.length; i++)
          for (let j = i + 1; j < value.length; j++)
            if (_deepEqual(value[i], value[j]))
              throw new TypeError(`${path}: duplicate items at [${i}] and [${j}]`)
      }
    }
    if (items !== undefined) {
      if (Array.isArray(items)) {
        items.forEach((s, i) => { if (i < value.length) validateType(value[i], s, `${path}[${i}]`, root) })
        if (additionalItems !== undefined && value.length > items.length)
          for (let i = items.length; i < value.length; i++)
            validateType(value[i], additionalItems, `${path}[${i}]`, root)
      } else if (items !== true) {
        value.forEach((v, i) => validateType(v, items, `${path}[${i}]`, root))
      }
    }
    if (contains !== undefined) {
      if (!value.some(v => _softValidate(v, contains, root)))
        throw new TypeError(`${path}: no item matches contains`)
    }
  }

  // object constraints
  if (value !== null && typeof value === 'object' && !Array.isArray(value)) {
    const keys = Object.keys(value)
    if (minProperties !== undefined && keys.length < minProperties)
      throw new RangeError(`${path}: ${keys.length} properties < minProperties ${minProperties}`)
    if (maxProperties !== undefined && keys.length > maxProperties)
      throw new RangeError(`${path}: ${keys.length} properties > maxProperties ${maxProperties}`)
    if (required !== undefined)
      for (const k of required)
        if (!Object.prototype.hasOwnProperty.call(value, k))
          throw new TypeError(`${path}: missing required "${k}"`)
    if (properties !== undefined)
      for (const [k, s] of Object.entries(properties))
        if (Object.prototype.hasOwnProperty.call(value, k))
          validateType(value[k], s, `${path}.${k}`, root)
    if (dependencies !== undefined)
      for (const [k, dep] of Object.entries(dependencies)) {
        if (!Object.prototype.hasOwnProperty.call(value, k)) continue
        if (Array.isArray(dep)) {
          for (const req of dep)
            if (!Object.prototype.hasOwnProperty.call(value, req))
              throw new TypeError(`${path}: dependency "${k}" requires "${req}"`)
        } else {
          validateType(value, dep, path, root)
        }
      }
    if (patternProperties !== undefined)
      for (const [pat, s] of Object.entries(patternProperties)) {
        const re = _re(pat)
        for (const k of keys)
          if (re.test(k)) validateType(value[k], s, `${path}.${k}`, root)
      }
    // additionalProperties: only active when schema explicitly restricts
    if (additionalProperties !== undefined) {
      const knownFromProps = properties ? Object.keys(properties) : []
      const knownFromPattern = patternProperties
        ? keys.filter(k => Object.keys(patternProperties).some(pat => _re(pat).test(k)))
        : []
      const knownKeys = new Set([...knownFromProps, ...knownFromPattern])
      for (const k of keys) {
        if (knownKeys.has(k)) continue
        if (additionalProperties === false)
          throw new TypeError(`${path}: additional property "${k}" not allowed`)
        else if (additionalProperties !== true)
          validateType(value[k], additionalProperties, `${path}.${k}`, root)
      }
    }
  }

  // not
  if (not !== undefined && _softValidate(value, not, root))
    throw new TypeError(`${path}: matches "not" schema`)

  // allOf / anyOf / oneOf
  if (allOf !== undefined)
    for (const s of allOf) validateType(value, s, path, root)
  if (anyOf !== undefined && !anyOf.some(s => _softValidate(value, s, root)))
    throw new TypeError(`${path}: matches none of anyOf`)
  if (oneOf !== undefined) {
    const n = oneOf.filter(s => _softValidate(value, s, root)).length
    if (n !== 1) throw new TypeError(`${path}: matched ${n} of oneOf (expected 1)`)
  }

  // if/then/else
  if (ifSchema !== undefined) {
    if (_softValidate(value, ifSchema, root)) {
      if (thenSchema !== undefined) validateType(value, thenSchema, path, root)
    } else {
      if (elseSchema !== undefined) validateType(value, elseSchema, path, root)
    }
  }
}

// ─── ObjectTree ───────────────────────────────────────────────────────────────

export class ObjectTree {
  #schema
  #root    // top-level schema for $ref resolution
  #values  // runtime instance values

  constructor(schema, root) {
    this.#root = root ?? schema
    // Resolve $ref once at construction — #schema is always the concrete schema
    this.#schema = (schema && schema.$ref) ? _resolveRef(schema.$ref, this.#root) : schema
    this.#values = {}
    if (typeof this.#schema === 'object' && this.#schema !== null) this._buildTree(this.#schema)
  }

  _buildTree(schema) {
    const { properties, items } = schema

    // Expose schema metadata as read-only properties (meta layer)
    const metaKeys = ['type', 'minimum', 'maximum', 'exclusiveMinimum', 'exclusiveMaximum',
                      'minLength', 'maxLength', 'pattern', 'enum', 'const', 'format',
                      'description', 'default', 'required', 'title', 'multipleOf']
    for (const key of metaKeys) {
      if (key in schema) {
        Object.defineProperty(this, key, {
          value: schema[key], writable: false, enumerable: true, configurable: false,
        })
      }
    }

    // properties → both meta (tree.properties.age) and runtime (tree.age)
    // tree.age always returns the child ObjectTree (the property's class)
    // tree.age.value is the scalar instance value
    if (properties) {
      const propsNode = {}
      for (const [key, subSchema] of Object.entries(properties)) {
        const child = new ObjectTree(subSchema, this.#root)

        // meta: tree.properties.age.minimum
        Object.defineProperty(propsNode, key, {
          get: () => child,
          set: (v) => { child.value = v },
          enumerable: true, configurable: false,
        })

        // runtime: tree.age → always the child ObjectTree
        // tree.age = 30  sets tree.age.value = 30 (internalized in child)
        Object.defineProperty(this, key, {
          get: () => child,
          set: (v) => { child.value = v },
          enumerable: true, configurable: true,
        })
      }
      Object.defineProperty(this, 'properties', {
        value: propsNode, writable: false,
        enumerable: false,   // hidden from runtime iteration
        configurable: false,
      })
    }

    // items schema node (meta)
    if (items) {
      const itemsTree = Array.isArray(items)
        ? items.map(s => new ObjectTree(s, this.#root))
        : new ObjectTree(items, this.#root)
      Object.defineProperty(this, 'items', {
        value: itemsTree, writable: false, enumerable: true, configurable: false,
      })
    }
  }

  // A node is an object node if it defines properties (with or without explicit type: 'object')
  _isObjectNode() {
    return typeof this.#schema === 'object' && this.#schema !== null
      && (this.#schema.type === 'object' || this.#schema.properties !== undefined)
  }

  // ── value: runtime instance setter/getter ──────────────────────────────────

  get value() { return this.#values['$value'] }

  set value(v) {
    validateType(v, this.#schema, '$', this.#root)
    this.#values['$value'] = v
  }

  // ── Logic methods ──────────────────────────────────────────────────────────

  /** oneOf(data) — exactly one branch matches; returns that branch as ObjectTree */
  oneOf(data) {
    const schemas = this.#schema.oneOf
    if (!schemas) throw new Error('Schema has no oneOf')
    const matches = schemas.filter(s => _softValidate(data, s, this.#root))
    if (matches.length !== 1)
      throw new TypeError(`oneOf: expected exactly 1 match, got ${matches.length}`)
    return new ObjectTree(matches[0], this.#root)
  }

  /** anyOf(data) — one or more branches match; returns all matching as ObjectTree[] */
  anyOf(data) {
    const schemas = this.#schema.anyOf
    if (!schemas) throw new Error('Schema has no anyOf')
    const matches = schemas.filter(s => _softValidate(data, s, this.#root))
    if (matches.length === 0) throw new TypeError('anyOf: no schema matched')
    return matches.map(s => new ObjectTree(s, this.#root))
  }

  /** allOf(data) — all branches must match; returns deep-merged ObjectTree */
  allOf(data) {
    const schemas = this.#schema.allOf
    if (!schemas) throw new Error('Schema has no allOf')
    for (const s of schemas)
      if (!_softValidate(data, s, this.#root))
        throw new TypeError(`allOf: schema failed — ${JSON.stringify(s)}`)
    return new ObjectTree(schemas.reduce(_deepMerge, {}), this.#root)
  }

  /** notOf(data) — data must NOT match the not-schema */
  notOf(data) {
    const notSchema = this.#schema.not
    if (!notSchema) throw new Error('Schema has no not')
    if (_softValidate(data, notSchema, this.#root))
      throw new TypeError('notOf: data matches the excluded schema')
    return true
  }

  /** ifThen(data) — evaluate if/then/else; returns the chosen branch as ObjectTree */
  ifThen(data) {
    const { if: ifSchema, then: thenSchema, else: elseSchema } = this.#schema
    if (!ifSchema) throw new Error('Schema has no if')
    const branch = _softValidate(data, ifSchema, this.#root) ? thenSchema : elseSchema
    return branch ? new ObjectTree(branch, this.#root) : null
  }

  /** project(data) — filter data to schema-defined properties, return new ObjectTree with values set */
  project(data) {
    if (data === null || typeof data !== 'object' || Array.isArray(data))
      throw new TypeError('project: data must be an object')
    const props = this.#schema.properties
    const projected = new ObjectTree(this.#schema, this.#root)
    if (props)
      for (const key of Object.keys(props))
        if (Object.prototype.hasOwnProperty.call(data, key))
          projected[key] = data[key]
    return projected
  }

  /** contains(data) — verify array contains at least one item matching the contains schema */
  contains(data) {
    if (!Array.isArray(data)) throw new TypeError('contains: data must be an array')
    const containsSchema = this.#schema.contains
    if (!containsSchema) throw new Error('Schema has no contains')
    if (!data.some(item => _softValidate(item, containsSchema, this.#root)))
      throw new TypeError('contains: no item matches the contains schema')
    return true
  }

  // ── Serialization ──────────────────────────────────────────────────────────

  /** Runtime instance → plain object */
  toDict() {
    if (this._isObjectNode()) {
      const out = {}
      for (const key of Object.keys(this.#schema.properties)) {
        const v = this.properties[key].toDict()
        if (v !== undefined) out[key] = v
      }
      return Object.keys(out).length ? out : undefined
    }
    return this.#values['$value']
  }

  /** Schema definition → plain object */
  schema() { return _schemaToDict(this.#schema) }

  toJSON() { return JSON.stringify(this.toDict(), null, 2) }
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

/** Resolve a JSON Pointer $ref against root schema */
function _resolveRef(ref, root) {
  if (!ref.startsWith('#/')) throw new TypeError(`Unsupported $ref: ${ref}`)
  const parts = ref.slice(2).split('/').map(p => p.replace(/~1/g, '/').replace(/~0/g, '~'))
  let node = root
  for (const part of parts) {
    if (node == null || typeof node !== 'object') throw new TypeError(`$ref path not found: ${ref}`)
    node = node[part]
  }
  if (node === undefined) throw new TypeError(`$ref not found: ${ref}`)
  return node
}

/** Soft validate: ObjectTree is the validator — try setting value, catch = invalid */
function _softValidate(data, schema, root) {
  try { new ObjectTree(schema, root).value = data; return true } catch { return false }
}

/** Schema → plain object (for schema() method) */
function _schemaToDict(schema) {
  if (schema === null || typeof schema !== 'object') return schema
  const out = {}
  for (const [k, v] of Object.entries(schema)) {
    if (k === 'properties') {
      out.properties = {}
      for (const [pk, pv] of Object.entries(v)) out.properties[pk] = _schemaToDict(pv)
    } else if (Array.isArray(v)) {
      out[k] = v.map(_schemaToDict)
    } else if (typeof v === 'object' && v !== null) {
      out[k] = _schemaToDict(v)
    } else {
      out[k] = v
    }
  }
  return out
}
