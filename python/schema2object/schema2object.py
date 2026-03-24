"""
schema2object — Python
JSON Schema IS the object class.
Spec: docs/draft-07-spec.json
"""

import copy
import json
import math
import os
import re
from urllib.parse import urljoin, urlparse, unquote

# ─── Shared ───────────────────────────────────────────────────────────────────

_MISSING = object()  # sentinel for "no value provided"

_re_cache = {}

def _re(pat):
    if pat not in _re_cache:
        _re_cache[pat] = re.compile(pat)
    return _re_cache[pat]

# ─── Deep equality ────────────────────────────────────────────────────────────

def _deep_equal(a, b):
    # bool is distinct from int/float (True != 1, False != 0)
    if isinstance(a, bool) != isinstance(b, bool):
        return False
    # Numeric: int and float are equal if same value (0 == 0.0, 1 == 1.0)
    if isinstance(a, (int, float)) and not isinstance(a, bool) and isinstance(b, (int, float)) and not isinstance(b, bool):
        return a == b
    # All other cross-type comparisons are unequal
    if type(a) is not type(b):
        return False
    if a is b:
        return True
    if isinstance(a, list) and isinstance(b, list):
        if len(a) != len(b):
            return False
        return all(_deep_equal(x, y) for x, y in zip(a, b))
    if isinstance(a, dict) and isinstance(b, dict):
        ka = sorted(a.keys())
        kb = sorted(b.keys())
        if ka != kb:
            return False
        return all(_deep_equal(a[k], b[k]) for k in ka)
    return a == b

# ─── Deep schema merge ────────────────────────────────────────────────────────

def _deep_merge(a, b):
    if not isinstance(a, dict) or a is None:
        return b
    if not isinstance(b, dict) or b is None:
        return b
    out = dict(a)
    for k, v in b.items():
        if k == 'properties' and isinstance(out.get('properties'), dict):
            out['properties'] = dict(out['properties'])
            for pk, pv in v.items():
                if pk in out['properties']:
                    out['properties'][pk] = _deep_merge(out['properties'][pk], pv)
                else:
                    out['properties'][pk] = pv
        elif k == 'required' and isinstance(out.get('required'), list) and isinstance(v, list):
            seen = set(out['required'])
            merged = list(out['required'])
            for item in v:
                if item not in seen:
                    merged.append(item)
                    seen.add(item)
            out['required'] = merged
        elif isinstance(v, dict) and v is not None and isinstance(out.get(k), dict):
            out[k] = _deep_merge(out[k], v)
        else:
            out[k] = v
    return out

# ─── URI helpers ──────────────────────────────────────────────────────────────

def _resolve_uri(ref, base):
    if not base or re.match(r'^[a-zA-Z][a-zA-Z0-9+\-.]*:', ref):
        return ref
    if ref.startswith('#'):
        return ref
    try:
        return urljoin(base, ref)
    except Exception:
        return ref

def _split_fragment(uri):
    idx = uri.find('#')
    if idx == -1:
        return (uri, None)
    frag = uri[idx + 1:]
    return (uri[:idx], frag if frag else None)

def _unescape_pointer(p):
    # RFC 6901: ~1 → /, ~0 → ~ (in that order)
    return p.replace('~1', '/').replace('~0', '~')

def _json_pointer(root, pointer, percent_decode=False):
    # pointer starts with '/'
    parts_raw = pointer[1:].split('/') if pointer.startswith('/') else pointer.split('/')
    parts = []
    for p in parts_raw:
        unescaped = _unescape_pointer(p)
        if percent_decode:
            unescaped = unquote(unescaped)
        parts.append(unescaped)
    node = root
    for p in parts:
        if node is None or not isinstance(node, (dict, list)):
            raise TypeError(f'$ref path not found: {pointer}')
        if isinstance(node, list):
            try:
                node = node[int(p)]
            except (ValueError, IndexError):
                raise TypeError(f'$ref path not found: {pointer}')
        else:
            if p not in node:
                raise TypeError(f'$ref not found: {pointer}')
            node = node[p]
    if node is None and pointer not in ('', '/'):
        pass  # None is a valid value
    return node

# ─── Loader ───────────────────────────────────────────────────────────────────

class Loader:
    """Resolves $ref per JSON Schema Draft-07 URI semantics."""

    def __init__(self, root, resolver=None, document_base=None):
        self.__root = root
        self.__resolver = resolver if resolver is not None else None
        self.__remote_cache = {}   # docUri → Loader
        self.__id_map = {}         # uri → node
        self.__scope_map = {}      # id(node) → {scope, parent_scope, resource}

        raw_base = document_base or (root.get('$id') if isinstance(root, dict) else None)
        if isinstance(raw_base, str) and raw_base.endswith('#'):
            raw_base = raw_base[:-1]
        base = raw_base
        self._scan_ids(root, base, base)

    def _scan_ids(self, node, base_uri, resource_root):
        if not isinstance(node, dict):
            if isinstance(node, list):
                for n in node:
                    self._scan_ids(n, base_uri, resource_root)
            return

        current_base = base_uri
        current_resource = resource_root

        if isinstance(node.get('$id'), str):
            current_base = _resolve_uri(node['$id'], base_uri)
            normalized = current_base[:-1] if current_base.endswith('#') else current_base
            self.__id_map[normalized] = node
            if normalized != current_base:
                self.__id_map[current_base] = node
            if node['$id'].startswith('#'):
                self.__id_map[node['$id']] = node
            current_base = normalized
            current_resource = normalized

        self.__scope_map[id(node)] = {
            'scope': current_base,
            'parent_scope': base_uri,
            'resource': current_resource,
        }

        for v in node.values():
            self._scan_ids(v, current_base, current_resource)

    def scope_of(self, node):
        return self.__scope_map.get(id(node), {}).get('scope')

    def parent_scope_of(self, node):
        return self.__scope_map.get(id(node), {}).get('parent_scope')

    def resource_of(self, node):
        return self.__scope_map.get(id(node), {}).get('resource')

    def resolve(self, ref, base_uri, resource_uri=None):
        # '#' = root of the nearest $id-bearing resource
        if ref == '#':
            res_node = self.__id_map.get(resource_uri) if resource_uri else None
            return (res_node if res_node is not None else self.__root, self)

        # '#/path' — JSON Pointer relative to nearest resource root
        if ref.startswith('#/'):
            res_node = self.__id_map.get(resource_uri) if resource_uri else None
            doc_root = res_node if res_node is not None else self.__root
            return (_json_pointer(doc_root, ref[1:], True), self)

        # Named anchor (fragment-only, not a pointer)
        if ref.startswith('#'):
            if ref in self.__id_map:
                return (self.__id_map[ref], self)
            raise TypeError(f'$ref anchor not found: {ref}')

        # Resolve relative ref against base URI
        resolved = _resolve_uri(ref, base_uri)
        doc_uri, fragment = _split_fragment(resolved)

        # Check $id map for the document part
        doc_node = self.__id_map.get(doc_uri) or self.__id_map.get(resolved)
        if doc_node is not None:
            if not fragment:
                return (doc_node, self)
            sub_loader = Loader(doc_node, self.__resolver)
            return sub_loader.resolve('#' + fragment, None)

        # External loading
        if re.match(r'^https?://', resolved) or self.__resolver:
            return self._resolve_remote(doc_uri, fragment, ref)

        raise TypeError(f'Unsupported $ref: {ref}')

    def _resolve_remote(self, doc_uri, fragment, original_ref=None):
        if doc_uri not in self.__remote_cache:
            schema = self._load_schema(doc_uri, original_ref)
            self.__remote_cache[doc_uri] = Loader(schema, self.__resolver, doc_uri)
        sub_loader = self.__remote_cache[doc_uri]
        if not fragment:
            return (sub_loader.__root, sub_loader)
        return sub_loader.resolve('#' + fragment, None)

    def _load_schema(self, doc_uri, original_ref=None):
        r = self.__resolver
        if not r:
            raise TypeError(f'$ref remote not supported (no resolver): {doc_uri}')

        if callable(r):
            schema = r(doc_uri) or (r(original_ref) if original_ref else None)
            if schema is None:
                raise TypeError(f'$ref resolver returned nothing for: {doc_uri}')
            return schema

        if isinstance(r, dict):
            schema = r.get(doc_uri) or (r.get(original_ref) if original_ref else None)
            if schema is not None:
                return schema
            raise TypeError(f'$ref not found in schema map: {doc_uri}')

        if isinstance(r, str):
            # String resolver: directory path
            m = re.match(r'^https?://[^/]+/(.*)$', doc_uri)
            file_path = m.group(1) if m else doc_uri
            full = os.path.join(r, file_path)
            if not os.path.exists(full):
                full = full + '.json'
            try:
                with open(full, 'r', encoding='utf-8') as f:
                    return json.load(f)
            except Exception:
                raise TypeError(f'$ref file not found: {full}')

        raise TypeError(f'$ref unsupported resolver type: {type(r)}')

    @property
    def root(self):
        return self.__root

    @property
    def resolver(self):
        return self.__resolver

# ─── Soft validate ────────────────────────────────────────────────────────────

def _soft_validate(data, schema, loader):
    try:
        validate_type(data, schema, '$', loader)
        return True
    except Exception:
        return False

# ─── Type checks ──────────────────────────────────────────────────────────────

def _type_check(t, value):
    if t == 'string':
        return isinstance(value, str)
    if t == 'integer':
        if isinstance(value, bool):
            return False
        if isinstance(value, int):
            return True
        if isinstance(value, float):
            return math.isfinite(value) and value == int(value)
        return False
    if t == 'number':
        return isinstance(value, (int, float)) and not isinstance(value, bool) and not (isinstance(value, float) and math.isnan(value))
    if t == 'boolean':
        return isinstance(value, bool)
    if t == 'array':
        return isinstance(value, list)
    if t == 'object':
        return isinstance(value, dict)
    if t == 'null':
        return value is None
    return False

# ─── Core validator ───────────────────────────────────────────────────────────

def validate_type(value, schema, path='$', loader=None):
    if schema is True:
        return
    if schema is False:
        raise TypeError(f'{path}: schema is false')

    # $ref → resolve and delegate; sibling keywords ignored (Draft 4-7)
    if isinstance(schema, dict) and '$ref' in schema:
        if loader is None:
            loader = Loader(schema)
        base_uri = loader.parent_scope_of(schema)
        resource_uri = loader.resource_of(schema)
        node, ref_loader = loader.resolve(schema['$ref'], base_uri, resource_uri)
        return validate_type(value, node, path, ref_loader)

    if loader is None:
        loader = Loader(schema) if isinstance(schema, dict) else Loader({})

    if not isinstance(schema, dict):
        return

    type_ = schema.get('type')
    const_val = schema.get('const', _MISSING)
    enum_vals = schema.get('enum', _MISSING)
    minimum = schema.get('minimum', _MISSING)
    maximum = schema.get('maximum', _MISSING)
    exclusive_minimum = schema.get('exclusiveMinimum', _MISSING)
    exclusive_maximum = schema.get('exclusiveMaximum', _MISSING)
    multiple_of = schema.get('multipleOf', _MISSING)
    min_length = schema.get('minLength', _MISSING)
    max_length = schema.get('maxLength', _MISSING)
    pattern = schema.get('pattern', _MISSING)
    min_items = schema.get('minItems', _MISSING)
    max_items = schema.get('maxItems', _MISSING)
    unique_items = schema.get('uniqueItems', False)
    items = schema.get('items', _MISSING)
    additional_items = schema.get('additionalItems', _MISSING)
    contains = schema.get('contains', _MISSING)
    min_properties = schema.get('minProperties', _MISSING)
    max_properties = schema.get('maxProperties', _MISSING)
    required = schema.get('required', _MISSING)
    properties = schema.get('properties', _MISSING)
    additional_properties = schema.get('additionalProperties', _MISSING)
    pattern_properties = schema.get('patternProperties', _MISSING)
    property_names = schema.get('propertyNames', _MISSING)
    dependencies = schema.get('dependencies', _MISSING)
    not_ = schema.get('not', _MISSING)
    one_of = schema.get('oneOf', _MISSING)
    any_of = schema.get('anyOf', _MISSING)
    all_of = schema.get('allOf', _MISSING)
    if_schema = schema.get('if', _MISSING)
    then_schema = schema.get('then', _MISSING)
    else_schema = schema.get('else', _MISSING)

    # --- type ---
    if type_ is not None:
        types = type_ if isinstance(type_, list) else [type_]
        if isinstance(value, bool) and 'boolean' not in types:
            raise TypeError(f'{path}: expected {"|".join(types)}, got boolean')
        if not any(_type_check(t, value) for t in types):
            got = 'null' if value is None else type(value).__name__
            raise TypeError(f'{path}: expected {"|".join(types)}, got {got}')

    # --- const ---
    if const_val is not _MISSING:
        if not _deep_equal(value, const_val):
            raise TypeError(f'{path}: must equal const {json.dumps(const_val)}')

    # --- enum ---
    if enum_vals is not _MISSING:
        if not any(_deep_equal(e, value) for e in enum_vals):
            raise TypeError(f'{path}: not in enum')

    # --- numeric ---
    if isinstance(value, (int, float)) and not isinstance(value, bool):
        if minimum is not _MISSING and value < minimum:
            raise TypeError(f'{path}: {value} < minimum {minimum}')
        if maximum is not _MISSING and value > maximum:
            raise TypeError(f'{path}: {value} > maximum {maximum}')
        if exclusive_minimum is not _MISSING and value <= exclusive_minimum:
            raise TypeError(f'{path}: {value} <= exclusiveMinimum {exclusive_minimum}')
        if exclusive_maximum is not _MISSING and value >= exclusive_maximum:
            raise TypeError(f'{path}: {value} >= exclusiveMaximum {exclusive_maximum}')
        if multiple_of is not _MISSING:
            q = value / multiple_of
            if not math.isfinite(q) or abs(q - round(q)) > 1e-9 * max(1, abs(q)):
                raise TypeError(f'{path}: {value} not multipleOf {multiple_of}')

    # --- string ---
    if isinstance(value, str):
        length = len(value)  # code point length (JSON Schema spec)
        if min_length is not _MISSING and length < min_length:
            raise TypeError(f'{path}: length {length} < minLength {min_length}')
        if max_length is not _MISSING and length > max_length:
            raise TypeError(f'{path}: length {length} > maxLength {max_length}')
        if pattern is not _MISSING:
            if not _re(pattern).search(value):
                raise TypeError(f'{path}: does not match pattern {pattern}')

    # --- array ---
    if isinstance(value, list):
        if min_items is not _MISSING and len(value) < min_items:
            raise TypeError(f'{path}: length {len(value)} < minItems {min_items}')
        if max_items is not _MISSING and len(value) > max_items:
            raise TypeError(f'{path}: length {len(value)} > maxItems {max_items}')
        if unique_items:
            all_primitive = all(v is None or not isinstance(v, (dict, list)) for v in value)
            if all_primitive:
                tagged = [f'{type(v).__name__}:{v}' for v in value]
                if len(set(tagged)) != len(value):
                    raise TypeError(f'{path}: duplicate items')
            else:
                for i in range(len(value)):
                    for j in range(i + 1, len(value)):
                        if _deep_equal(value[i], value[j]):
                            raise TypeError(f'{path}: duplicate items at [{i}] and [{j}]')
        if items is not _MISSING:
            if isinstance(items, list):
                for i, s in enumerate(items):
                    if i < len(value):
                        validate_type(value[i], s, f'{path}[{i}]', loader)
                if len(value) > len(items) and additional_items is not _MISSING:
                    for i in range(len(items), len(value)):
                        validate_type(value[i], additional_items, f'{path}[{i}]', loader)
            else:
                for i, v in enumerate(value):
                    validate_type(v, items, f'{path}[{i}]', loader)
        if contains is not _MISSING:
            if not any(_soft_validate(v, contains, loader) for v in value):
                raise TypeError(f'{path}: no item matches contains')

    # --- object ---
    if isinstance(value, dict):
        keys = list(value.keys())
        if min_properties is not _MISSING and len(keys) < min_properties:
            raise TypeError(f'{path}: {len(keys)} properties < minProperties {min_properties}')
        if max_properties is not _MISSING and len(keys) > max_properties:
            raise TypeError(f'{path}: {len(keys)} properties > maxProperties {max_properties}')
        if required is not _MISSING:
            for k in required:
                if k not in value:
                    raise TypeError(f'{path}: missing required "{k}"')
        if property_names is not _MISSING:
            for k in keys:
                validate_type(k, property_names, f'{path}/<key:{k}>', loader)
        if properties is not _MISSING:
            for k, s in properties.items():
                if k in value:
                    validate_type(value[k], s, f'{path}.{k}', loader)
        if pattern_properties is not _MISSING:
            for pat, s in pattern_properties.items():
                r = _re(pat)
                for k in keys:
                    if r.search(k):
                        validate_type(value[k], s, f'{path}.{k}', loader)
        if additional_properties is not _MISSING and additional_properties is not True:
            known = set(properties.keys()) if properties is not _MISSING else set()
            pp_patterns = [_re(p) for p in pattern_properties.keys()] if pattern_properties is not _MISSING else []
            for k in keys:
                if k in known:
                    continue
                if any(r.search(k) for r in pp_patterns):
                    continue
                if additional_properties is False:
                    raise TypeError(f'{path}: additional property "{k}" not allowed')
                validate_type(value[k], additional_properties, f'{path}.{k}', loader)
        if dependencies is not _MISSING:
            for k, dep in dependencies.items():
                if k not in value:
                    continue
                if isinstance(dep, list):
                    for req in dep:
                        if req not in value:
                            raise TypeError(f'{path}: dependency "{k}" requires "{req}"')
                else:
                    validate_type(value, dep, path, loader)

    # --- combinators ---
    if not_ is not _MISSING:
        if _soft_validate(value, not_, loader):
            raise TypeError(f'{path}: matches "not" schema')

    if all_of is not _MISSING:
        for s in all_of:
            validate_type(value, s, path, loader)

    if any_of is not _MISSING:
        if not any(_soft_validate(value, s, loader) for s in any_of):
            raise TypeError(f'{path}: matches none of anyOf')

    if one_of is not _MISSING:
        n = sum(1 for s in one_of if _soft_validate(value, s, loader))
        if n != 1:
            raise TypeError(f'{path}: matched {n} of oneOf (expected 1)')

    if if_schema is not _MISSING:
        if _soft_validate(value, if_schema, loader):
            if then_schema is not _MISSING:
                validate_type(value, then_schema, path, loader)
        else:
            if else_schema is not _MISSING:
                validate_type(value, else_schema, path, loader)


# ─── ObjectTree ───────────────────────────────────────────────────────────────

class ObjectTree:
    """Schema-bound object. Data and schema are one identity.
    Validation is automatic on construction and on every property set.
    """

    def __init__(self, data=_MISSING, schema=None, resolver=None):
        # resolver: Loader (reuse existing — internal re-bind only)
        #         | string (schema file path → dirname used as base dir)
        #         | object (uri → schema map)
        #         | function (uri → schema)
        #         | None (no external $ref)
        if isinstance(resolver, Loader):
            loader = resolver
        else:
            if isinstance(resolver, str):
                r = os.path.dirname(resolver) or '.'
            else:
                r = resolver if resolver is not None else None
            loader = Loader(schema if isinstance(schema, dict) else (schema if schema is not None else {}), r)

        object.__setattr__(self, '_ObjectTree__path', '$')

        if isinstance(schema, dict) and '$ref' in schema:
            node, ref_loader = loader.resolve(
                schema['$ref'],
                loader.scope_of(schema),
                loader.resource_of(schema),
            )
            object.__setattr__(self, '_ObjectTree__schema', node)
            object.__setattr__(self, '_ObjectTree__loader', ref_loader)
        else:
            object.__setattr__(self, '_ObjectTree__schema', schema)
            object.__setattr__(self, '_ObjectTree__loader', loader)

        object.__setattr__(self, '_ObjectTree__props', {})
        object.__setattr__(self, '_ObjectTree__value', None)
        object.__setattr__(self, '_ObjectTree__data', {})

        if self._is_object_node():
            object.__setattr__(self, '_ObjectTree__data', {})
            self._define_properties(self.__schema)

        if data is not _MISSING:
            self.value = data

    def _is_object_node(self):
        s = self.__schema
        return (
            isinstance(s, dict)
            and (s.get('type') == 'object' or 'properties' in s)
        )

    def _define_properties(self, schema):
        if not isinstance(schema, dict):
            return
        props = schema.get('properties')
        if not props:
            return
        loader = self.__loader
        for key, sub_schema in props.items():
            # Resolve $ref to get default
            resolved = sub_schema
            if isinstance(sub_schema, dict) and '$ref' in sub_schema:
                try:
                    resolved, _ = loader.resolve(
                        sub_schema['$ref'],
                        loader.scope_of(sub_schema),
                        loader.resource_of(sub_schema),
                    )
                except Exception:
                    resolved = sub_schema

            # Set default if present
            if isinstance(sub_schema, dict) and 'default' in sub_schema:
                def_ = sub_schema['default']
                data = object.__getattribute__(self, '_ObjectTree__data')
                data[key] = copy.deepcopy(def_) if isinstance(def_, (dict, list)) else def_
            elif isinstance(resolved, dict) and 'default' in resolved:
                def_ = resolved['default']
                data = object.__getattribute__(self, '_ObjectTree__data')
                data[key] = copy.deepcopy(def_) if isinstance(def_, (dict, list)) else def_

    def __getattr__(self, name):
        if name.startswith('_ObjectTree__') or name.startswith('__'):
            raise AttributeError(name)
        # Check if it's a schema-defined property
        schema = object.__getattribute__(self, '_ObjectTree__schema')
        if isinstance(schema, dict) and 'properties' in schema and name in schema['properties']:
            data = object.__getattribute__(self, '_ObjectTree__data')
            return data.get(name)
        raise AttributeError(f"'{type(self).__name__}' has no attribute '{name}'")

    def __setattr__(self, name, value):
        if name.startswith('_ObjectTree__') or name.startswith('__'):
            object.__setattr__(self, name, value)
            return
        schema = object.__getattribute__(self, '_ObjectTree__schema')
        loader = object.__getattribute__(self, '_ObjectTree__loader')
        path = object.__getattribute__(self, '_ObjectTree__path')
        if isinstance(schema, dict) and 'properties' in schema and name in schema['properties']:
            sub_schema = schema['properties'][name]
            validate_type(value, sub_schema, f'{path}.{name}', loader)
            data = object.__getattribute__(self, '_ObjectTree__data')
            data[name] = value
        else:
            object.__setattr__(self, name, value)

    @property
    def value(self):
        if self._is_object_node():
            return object.__getattribute__(self, '_ObjectTree__data')
        return object.__getattribute__(self, '_ObjectTree__value')

    @value.setter
    def value(self, v):
        schema = object.__getattribute__(self, '_ObjectTree__schema')
        loader = object.__getattribute__(self, '_ObjectTree__loader')
        path = object.__getattribute__(self, '_ObjectTree__path')
        validate_type(v, schema, path, loader)
        if self._is_object_node():
            # Merge: start from defaults already in __data, overlay with incoming data
            existing = object.__getattribute__(self, '_ObjectTree__data')
            if isinstance(v, dict):
                merged = dict(existing)
                merged.update(v)
                object.__setattr__(self, '_ObjectTree__data', merged)
            else:
                object.__setattr__(self, '_ObjectTree__data', dict(existing))
        else:
            object.__setattr__(self, '_ObjectTree__value', v)

    def one_of(self):
        schema = object.__getattribute__(self, '_ObjectTree__schema')
        loader = object.__getattribute__(self, '_ObjectTree__loader')
        schemas = schema.get('oneOf') if isinstance(schema, dict) else None
        if not schemas:
            raise Exception('Schema has no oneOf')
        data = self.value
        matches = [s for s in schemas if _soft_validate(data, s, loader)]
        if len(matches) != 1:
            raise TypeError(f'oneOf: expected exactly 1 match, got {len(matches)}')
        return ObjectTree(data, matches[0], loader)

    def any_of(self):
        schema = object.__getattribute__(self, '_ObjectTree__schema')
        loader = object.__getattribute__(self, '_ObjectTree__loader')
        schemas = schema.get('anyOf') if isinstance(schema, dict) else None
        if not schemas:
            raise Exception('Schema has no anyOf')
        data = self.value
        matches = [s for s in schemas if _soft_validate(data, s, loader)]
        if len(matches) == 0:
            raise TypeError('anyOf: no schema matched')
        return [ObjectTree(data, s, loader) for s in matches]

    def all_of(self):
        schema = object.__getattribute__(self, '_ObjectTree__schema')
        loader = object.__getattribute__(self, '_ObjectTree__loader')
        schemas = schema.get('allOf') if isinstance(schema, dict) else None
        if not schemas:
            raise Exception('Schema has no allOf')
        data = self.value
        merged = {}
        for s in schemas:
            merged = _deep_merge(merged, s)
        return ObjectTree(data, merged, loader)

    def not_of(self):
        schema = object.__getattribute__(self, '_ObjectTree__schema')
        loader = object.__getattribute__(self, '_ObjectTree__loader')
        not_schema = schema.get('not') if isinstance(schema, dict) else None
        if not not_schema:
            return True
        return not _soft_validate(self.value, not_schema, loader)

    def if_then(self):
        schema = object.__getattribute__(self, '_ObjectTree__schema')
        loader = object.__getattribute__(self, '_ObjectTree__loader')
        if not isinstance(schema, dict):
            return self
        if_schema = schema.get('if')
        if not if_schema:
            return self
        data = self.value
        then_schema = schema.get('then')
        else_schema = schema.get('else')
        branch = then_schema if _soft_validate(data, if_schema, loader) else else_schema
        return ObjectTree(data, branch, loader) if branch else self

    def project(self):
        data = self.value
        if data is None or not isinstance(data, dict):
            raise TypeError('project: data must be an object')
        schema = object.__getattribute__(self, '_ObjectTree__schema')
        loader = object.__getattribute__(self, '_ObjectTree__loader')
        props = schema.get('properties') if isinstance(schema, dict) else None
        if not props:
            return ObjectTree(data, schema, loader)
        out = {k: data[k] for k in props if k in data}
        projected = ObjectTree(schema=schema, resolver=loader)
        object.__setattr__(projected, '_ObjectTree__data', out)
        return projected

    def with_defaults(self):
        data = self.value
        if data is None or not isinstance(data, dict):
            raise TypeError('withDefaults: data must be an object')
        schema = object.__getattribute__(self, '_ObjectTree__schema')
        loader = object.__getattribute__(self, '_ObjectTree__loader')
        new_data = _apply_defaults(dict(data), schema, loader)
        return ObjectTree(new_data, schema, loader)

    def contains(self):
        data = self.value
        if not isinstance(data, list):
            return None
        schema = object.__getattribute__(self, '_ObjectTree__schema')
        loader = object.__getattribute__(self, '_ObjectTree__loader')
        cs = schema.get('contains') if isinstance(schema, dict) else None
        if not cs:
            return None
        return any(_soft_validate(item, cs, loader) for item in data)

    def to_dict(self):
        if self._is_object_node():
            schema = object.__getattribute__(self, '_ObjectTree__schema')
            data = object.__getattribute__(self, '_ObjectTree__data')
            props = schema.get('properties') if isinstance(schema, dict) else None
            if not props:
                return dict(data)
            return {k: data[k] for k in props if k in data}
        return object.__getattribute__(self, '_ObjectTree__value')

    @property
    def schema(self):
        return _schema_to_dict(object.__getattribute__(self, '_ObjectTree__schema'))

    def get_schema(self, path=''):
        if not path:
            return self.schema
        parts = [p for p in path.split('.') if p]
        node = object.__getattribute__(self, '_ObjectTree__schema')
        for part in parts:
            if not isinstance(node, dict):
                return None
            props = node.get('properties')
            if not props or part not in props:
                return None
            node = props[part]
        return _schema_to_dict(node)

    def get_extensions(self, path=''):
        node = self.get_schema(path) if path else self.schema
        if not isinstance(node, dict):
            return {}
        return {k: v for k, v in node.items() if k.startswith('x-')}

    def to_json(self):
        return json.dumps(self.to_dict(), indent=2)

    def __repr__(self):
        return f'ObjectTree({self.to_dict()!r})'

# ─── Helpers ──────────────────────────────────────────────────────────────────

def _schema_to_dict(schema):
    if schema is None or not isinstance(schema, dict):
        return schema
    out = {}
    for k, v in schema.items():
        if k == 'properties':
            out['properties'] = {pk: _schema_to_dict(pv) for pk, pv in v.items()}
        elif isinstance(v, list):
            out[k] = [_schema_to_dict(item) for item in v]
        elif isinstance(v, dict):
            out[k] = _schema_to_dict(v)
        else:
            out[k] = v
    return out

def _apply_defaults(data, schema, loader):
    if not isinstance(schema, dict):
        return data
    resolved = schema
    if '$ref' in schema:
        try:
            resolved, _ = loader.resolve(schema['$ref'], schema.get('$id'))
        except Exception:
            resolved = schema
    props = resolved.get('properties') if isinstance(resolved, dict) else None
    if not props:
        return data
    for k, raw_s in props.items():
        s = raw_s
        if isinstance(raw_s, dict) and '$ref' in raw_s:
            try:
                s, _ = loader.resolve(raw_s['$ref'], raw_s.get('$id'))
            except Exception:
                s = raw_s
        if k in data:
            if isinstance(data[k], dict):
                data[k] = _apply_defaults(dict(data[k]), raw_s, loader)
        else:
            if isinstance(raw_s, dict) and 'default' in raw_s:
                def_ = raw_s['default']
            elif isinstance(s, dict) and 'default' in s:
                def_ = s['default']
            else:
                def_ = _MISSING
            if def_ is not _MISSING:
                data[k] = copy.deepcopy(def_) if isinstance(def_, (dict, list)) else def_
    return data

# ─── validate ─────────────────────────────────────────────────────────────────

def validate(data, schema, resolver=None):
    try:
        ObjectTree(data, schema, resolver)
        return {'valid': True}
    except Exception as e:
        return {'valid': False, 'error': str(e)}
