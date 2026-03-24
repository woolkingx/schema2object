# schema2object — Python

**JSON Schema Draft-07 is the object class definition.**

`ObjectTree(data, schema=schema)` — if `data` violates `schema`, this raises. The schema is the class contract, not a hint. Construction validates. Mutation validates. No exceptions.

## Core Semantics

- **Construction validates** — invalid data cannot produce a valid instance
- **Mutation validates** — every write is checked before storage
- **`to_dict()` projects** — only schema-defined fields serialize out
- **Unknown fields are allowed** — extra runtime fields exist but are invisible to `to_dict()`
- **Draft-07 is the target** — all keywords, 100% of the official test suite

## Install

```bash
pip install schema2object
```

## Quick Example

```python
from schema2object import ObjectTree

schema = {
    'type': 'object',
    'properties': {
        'name': {'type': 'string'},
        'age': {'type': 'integer', 'minimum': 0, 'x-docs': 'Age in years'},
        'email': {'type': 'string', 'pattern': r'^[^@]+@[^@]+$'}
    },
    'required': ['name']
}

user = ObjectTree({'name': 'Alice'}, schema=schema)

# Dot-access
print(user.name)  # 'Alice'

# Type checking on assignment
user.age = 30        # ✓ OK
user.age = 'thirty'  # ✗ Raises TypeError

# Explicit schema access
print(user.get_schema('age'))       # {'type': 'integer', 'minimum': 0}
print(user.get_extensions('age'))   # {'x-docs': 'Age in years'}
```

## Draft-07 Logic → Methods

```python
data = ObjectTree({...}, schema=schema)
branch = data.one_of()   # oneOf (XOR)
branches = data.any_of() # anyOf (OR)
merged = data.all_of()   # allOf (AND)
filtered = data.project()# properties (SELECT)
```

## API Reference

### ObjectTree

**Constructor:**
```python
ObjectTree(data=None, *, schema=None, **kwargs)
```

**Methods:**
- `one_of()` → ObjectTree — Select unique oneOf branch
- `any_of()` → List[ObjectTree] — Get all anyOf matches
- `all_of()` → ObjectTree — Merge allOf schemas
- `not_of(schema=None)` → bool — Check exclusion
- `if_then()` → ObjectTree — Conditional branch
- `project()` → ObjectTree — Filter to schema fields
- `contains(schema=None)` → bool — Array element check
- `to_dict()` → dict — Unwrap to native Python
- `get_schema(path=None)` → dict|Any — Read schema (dot path supported)
- `get_extensions(path=None)` → dict — Read x-* extensions on schema

**Properties:**
- `is_mapping` → bool
- `is_sequence` → bool
- `schema` → dict|Any — Root schema as plain dict

### ObjectTreeEncoder

JSON encoder for ObjectTree instances:

```python
import json
json.dumps(obj, cls=ObjectTreeEncoder)
```

## Tests

```bash
python -m pytest tests/ -q
```

## See Also

- [schema2object root](../)
