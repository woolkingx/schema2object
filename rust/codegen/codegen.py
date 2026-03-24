#!/usr/bin/env python3
"""schema2object codegen — JSON Schema → runtime + typed struct in one .rs file.

Copies schema2object.rs (runtime) and appends typed struct definitions
generated from the schema. One file = runtime + types.

Usage:
    python codegen.py schema.json --out src/user.rs
"""
import json, sys, os, re

# ─── type map ─────────────────────────────────────────────────────────────────

TYPE_MAP = {
    'string':  'String',
    'integer': 'i64',
    'number':  'f64',
    'boolean': 'bool',
    'null':    '()',
}

def title_case(s):
    s = re.sub(r'(?:^|[_\-\s])([a-z])', lambda m: m.group(1).upper(), s)
    return re.sub(r'[^a-zA-Z0-9]', '', s) or 'Unknown'

RUST_KEYWORDS = {
    'if', 'else', 'then', 'for', 'while', 'loop', 'match', 'return',
    'break', 'continue', 'fn', 'let', 'mut', 'ref', 'type', 'use',
    'mod', 'pub', 'self', 'super', 'crate', 'as', 'in', 'where',
    'struct', 'enum', 'impl', 'trait', 'const', 'static', 'move',
    'true', 'false', 'async', 'await', 'dyn', 'abstract', 'become',
    'box', 'do', 'final', 'macro', 'override', 'priv', 'try',
    'typeof', 'unsafe', 'unsized', 'virtual', 'yield', 'extern',
}

def snake_case(s):
    s = re.sub(r'([A-Z])', r'_\1', s).lower().lstrip('_')
    s = re.sub(r'[^a-z0-9_]', '_', s)
    if s in RUST_KEYWORDS:
        s = f'r#{s}'
    return s

# ─── $ref / allOf ─────────────────────────────────────────────────────────────

def resolve_ref(schema, root):
    if not isinstance(schema, dict) or '$ref' not in schema:
        return schema
    ref = schema['$ref']
    if not ref.startswith('#/'):
        return schema
    node = root
    for p in ref[2:].split('/'):
        p = p.replace('~1', '/').replace('~0', '~')
        node = node.get(p) if isinstance(node, dict) else None
        if node is None:
            return schema
    return resolve_ref(node, root)

def merge_all_of(schema, root):
    if not isinstance(schema, dict) or 'allOf' not in schema:
        return schema
    merged = {}
    for s in schema['allOf']:
        s = resolve_ref(s, root)
        s = merge_all_of(s, root)
        merged = deep_merge(merged, s)
    for k, v in schema.items():
        if k != 'allOf' and k not in merged:
            merged[k] = v
    return merged

def deep_merge(a, b):
    if not isinstance(a, dict) or not isinstance(b, dict):
        return b if b is not None else a
    out = dict(a)
    for k, v in b.items():
        if k == 'properties' and 'properties' in out:
            out['properties'] = {**out['properties']}
            for pk, pv in v.items():
                out['properties'][pk] = deep_merge(out['properties'].get(pk, {}), pv)
        elif k == 'required' and isinstance(out.get('required'), list):
            seen = set(out['required'])
            out['required'] = list(out['required']) + [x for x in v if x not in seen]
        else:
            out[k] = v
    return out

# ─── collect structs ──────────────────────────────────────────────────────────

def rust_type(schema, required, field_name, structs, root):
    if not isinstance(schema, dict):
        return 'Value'
    schema = resolve_ref(schema, root)
    schema = merge_all_of(schema, root)
    t = schema.get('type')
    if isinstance(t, list):
        t = t[0] if len(t) == 1 else None  # multi-type → Value

    if t in TYPE_MAP:
        rt = TYPE_MAP[t]
    elif t == 'array':
        items = resolve_ref(schema.get('items', {}), root)
        inner = rust_type(items, True, field_name + '_item', structs, root)
        rt = f'Vec<{inner}>'
    elif t == 'object' or 'properties' in schema:
        name = title_case(field_name)
        if name not in structs:
            collect_struct(name, schema, structs, root)
        rt = name
    else:
        rt = 'Value'

    return rt if required else f'Option<{rt}>'

def collect_struct(name, schema, structs, root):
    schema = resolve_ref(schema, root)
    schema = merge_all_of(schema, root)
    structs[name] = None  # cycle guard
    props = schema.get('properties', {})
    req = set(schema.get('required', []))
    fields = []
    for fname, fschema in props.items():
        fields.append({
            'name': snake_case(fname),
            'original': fname,
            'rust_type': rust_type(fschema, fname in req, fname, structs, root),
        })
    structs[name] = fields

# ─── emit ─────────────────────────────────────────────────────────────────────

def emit_structs(structs):
    lines = []
    lines.append('')
    lines.append('// ─── Generated typed structs ──────────────────────────────────────────────────')
    lines.append('')
    for name, fields in structs.items():
        if fields is None:
            continue
        lines.append(f'pub struct {name} {{')
        for f in fields:
            if f['original'] != f['name']:
                lines.append(f'    // JSON key: "{f["original"]}"')
            lines.append(f'    pub {f["name"]}: {f["rust_type"]},')
        lines.append('}')
        lines.append('')
    return '\n'.join(lines)

# ─── main ─────────────────────────────────────────────────────────────────────

RUNTIME_PATH = os.path.join(os.path.dirname(__file__), '..', 'runtime', 'src', 'schema2object.rs')

def main():
    if len(sys.argv) < 2:
        print('Usage: python codegen.py schema.json [--out src/generated.rs]', file=sys.stderr)
        sys.exit(1)

    schema_path = os.path.abspath(sys.argv[1])
    out_path = None
    if '--out' in sys.argv:
        out_path = os.path.abspath(sys.argv[sys.argv.index('--out') + 1])

    with open(schema_path) as f:
        root = json.load(f)

    # read runtime source
    with open(os.path.abspath(RUNTIME_PATH)) as f:
        runtime_src = f.read()

    # collect types from schema
    flat = merge_all_of(root, root)
    structs = {}
    root_name = title_case(flat.get('title', 'Root'))
    collect_struct(root_name, flat, structs, root)

    for dk, dv in {**flat.get('definitions', {}), **flat.get('$defs', {})}.items():
        name = title_case(dk)
        if name not in structs:
            collect_struct(name, dv, structs, root)

    # skip names already in runtime
    runtime_names = set(re.findall(r'pub struct (\w+)', runtime_src))
    runtime_names |= set(re.findall(r'pub enum (\w+)', runtime_src))
    for name in list(structs):
        if name in runtime_names:
            print(f'warning: skipping "{name}" (conflicts with runtime built-in)', file=sys.stderr)
            del structs[name]

    # runtime + types = one file
    output = runtime_src.rstrip() + '\n' + emit_structs(structs)

    if out_path:
        with open(out_path, 'w') as f:
            f.write(output)
        print(f'wrote {out_path}', file=sys.stderr)
    else:
        sys.stdout.write(output)

if __name__ == '__main__':
    main()
