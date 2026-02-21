# schema2object

[![Python Version](https://img.shields.io/badge/python-3.9%2B-blue.svg)](https://www.python.org/downloads/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![JSON Schema](https://img.shields.io/badge/JSON%20Schema-Draft--07-green.svg)](https://json-schema.org/specification-links.html#draft-7)

> JSON Schema Draft-07 as Python object. Structure maps to attributes, logic maps to methods.

JSON came from JavaScript objects. Serialization stripped the methods.
`ObjectTree` restores them: the schema defines the class, the data is the instance.

Draft-07 logical operators (`oneOf`, `anyOf`, `allOf`, `if/then/else`, ...) map directly to methods.
Dot-access navigates state. Method calls execute logic.

## Features

- 🎯 **Dot-access notation** — `obj.user.name` instead of `obj['user']['name']`
- ✅ **Draft-07 validation** — Type checking, constraints, composition keywords
- 🔒 **Type binding** — Schema validation on every assignment
- 📦 **Zero dependencies** — Pure Python 3.9+, stdlib only
- 🔄 **Auto-wrapping** — Nested dicts/lists become ObjectTree instances
- 🎨 **JSON serialization** — Built-in `ObjectTreeEncoder` support
- 🐍 **Pythonic** — Implements `MutableMapping`, pickle, deepcopy

## Installation

```bash
pip install schema2object
```

## Quick Start

```python
from schema2object import ObjectTree

# Define schema
schema = {
    'type': 'object',
    'properties': {
        'name': {'type': 'string'},
        'age': {'type': 'integer', 'minimum': 0},
        'email': {'type': 'string', 'pattern': r'^[^@]+@[^@]+$'}
    },
    'required': ['name']
}

# Create object with validation
user = ObjectTree({'name': 'Alice'}, schema=schema)

# Dot-access
print(user.name)  # 'Alice'

# Type checking on assignment
user.age = 30      # ✓ OK
user.age = 'thirty'  # ✗ Raises TypeError

# Email validation
user.email = 'alice@example.com'  # ✓ OK
user.email = 'invalid'  # ✗ Raises TypeError (pattern mismatch)
```

## Core Concepts

### Structure vs. Logic

**Schema keywords → attributes** (raw access):
```python
schema = ObjectTree({'oneOf': [...], 'properties': {...}})
schema.oneOf  # Get raw list
```

**Schema logic → methods** (computed access):
```python
data = ObjectTree({...}, schema=schema)
branch = data.one_of()  # Select matching branch (XOR logic)
```

### Automatic Wrapping

Nested structures become ObjectTree instances automatically:

```python
obj = ObjectTree({
    'user': {
        'profile': {
            'name': 'Alice'
        }
    }
})

obj.user.profile.name  # Full dot-access chain
```

### Default Auto-Fill

Missing fields get defaults from schema:

```python
schema = {
    'properties': {
        'status': {'type': 'string', 'default': 'pending'},
        'priority': {'type': 'integer', 'default': 0}
    }
}

task = ObjectTree({}, schema=schema)
print(task.status)    # 'pending'
print(task.priority)  # 0
```

## Schema Composition

### oneOf (XOR Logic)

Select the unique matching branch:

```python
schema = {
    'oneOf': [
        {'properties': {'type': {'const': 'user'}, 'name': {'type': 'string'}}},
        {'properties': {'type': {'const': 'bot'}, 'id': {'type': 'integer'}}}
    ]
}

data = ObjectTree({'type': 'user', 'name': 'Alice'}, schema=schema)
branch = data.one_of()  # Selects user branch

# Type binding now works on selected branch
branch.name = 'Bob'  # ✓ OK
branch.name = 123    # ✗ TypeError
```

### anyOf (OR Logic)

Get all matching branches:

```python
schema = {
    'anyOf': [
        {'properties': {'x': {'type': 'integer'}}},
        {'properties': {'x': {'type': 'number'}}}
    ]
}

data = ObjectTree({'x': 42}, schema=schema)
branches = data.any_of()  # Returns list of ObjectTree instances
```

### allOf (AND Logic)

Merge all sub-schemas:

```python
schema = {
    'allOf': [
        {'properties': {'name': {'type': 'string'}}},
        {'properties': {'age': {'type': 'integer'}}}
    ]
}

data = ObjectTree({'name': 'Alice', 'age': 30}, schema=schema)
merged = data.all_of()  # Merged schema with both constraints
```

### Conditional Logic (if/then/else)

Branch based on conditions:

```python
schema = {
    'if': {'properties': {'role': {'const': 'admin'}}},
    'then': {'properties': {'level': {'minimum': 5}}},
    'else': {'properties': {'level': {'maximum': 4}}}
}

admin = ObjectTree({'role': 'admin', 'level': 10}, schema=schema)
result = admin.if_then()  # Uses 'then' branch
```

### Projection (SELECT)

Filter to schema-defined fields:

```python
schema = {
    'properties': {
        'name': {'type': 'string'},
        'age': {'type': 'integer'}
    }
}

data = ObjectTree({'name': 'Alice', 'age': 30, 'extra': 'ignored'}, schema=schema)
clean = data.project()  # {'name': 'Alice', 'age': 30}
```

## Draft-07 Validation

### Type Validation

```python
schema = {'properties': {'count': {'type': 'integer'}}}
obj = ObjectTree({}, schema=schema)
obj.count = 42   # ✓ OK
obj.count = 3.14 # ✗ TypeError (float is not integer)
obj.count = True # ✗ TypeError (bool is not integer in Draft-07)
```

### Constraints

**Numeric:**
```python
{'type': 'integer', 'minimum': 0, 'maximum': 100, 'multipleOf': 5}
```

**String:**
```python
{'type': 'string', 'minLength': 2, 'maxLength': 50, 'pattern': r'^[A-Z]'}
```

**Array:**
```python
{'type': 'array', 'minItems': 1, 'maxItems': 10, 'uniqueItems': True}
```

**Object:**
```python
{
    'type': 'object',
    'required': ['name'],
    'minProperties': 1,
    'maxProperties': 10,
    'additionalProperties': False
}
```

### Enum and Const

```python
schema = {
    'properties': {
        'status': {'enum': ['pending', 'active', 'done']},
        'version': {'const': 2}
    }
}

obj = ObjectTree({}, schema=schema)
obj.status = 'active'   # ✓ OK
obj.status = 'invalid'  # ✗ TypeError
obj.version = 2         # ✓ OK
obj.version = 3         # ✗ TypeError
```

## JSON Serialization

```python
import json
from schema2object import ObjectTree, ObjectTreeEncoder

obj = ObjectTree({'user': {'name': 'Alice', 'age': 30}})

# Using custom encoder
json_str = json.dumps(obj, cls=ObjectTreeEncoder)

# Or convert to dict first
json_str = json.dumps(obj.to_dict())
```

## Python Protocols

### MutableMapping

```python
obj = ObjectTree({'a': 1, 'b': 2})

# Dict-like access
obj['c'] = 3
del obj['a']
'b' in obj  # True
len(obj)    # 2
list(obj.keys())    # ['b', 'c']
list(obj.values())  # [2, 3]
```

### Copy and Deepcopy

```python
import copy

obj = ObjectTree({'nested': {'value': 42}})
shallow = obj.copy()
deep = copy.deepcopy(obj)
```

### Pickle

```python
import pickle

obj = ObjectTree({'data': [1, 2, 3]})
data = pickle.dumps(obj)
restored = pickle.loads(data)
```

### Merge (|= operator)

```python
a = ObjectTree({'x': 1})
b = ObjectTree({'y': 2})
c = a | b  # {'x': 1, 'y': 2}

a |= {'z': 3}  # In-place merge
```

## Advanced Usage

### Schema Propagation

Child objects inherit sub-schemas:

```python
schema = {
    'properties': {
        'user': {
            'type': 'object',
            'properties': {
                'name': {'type': 'string'}
            }
        }
    }
}

obj = ObjectTree({'user': {'name': 'Alice'}}, schema=schema)

# Child has sub-schema
obj.user.name = 'Bob'  # ✓ Validated
obj.user.name = 123    # ✗ TypeError
```

### Pattern Properties

```python
schema = {
    'patternProperties': {
        '^S_': {'type': 'string'},
        '^I_': {'type': 'integer'}
    }
}

obj = ObjectTree({}, schema=schema)
obj['S_name'] = 'Alice'  # ✓ OK
obj['I_count'] = 42      # ✓ OK
obj['S_name'] = 123      # ✗ TypeError
```

### Dependencies

```python
schema = {
    'dependencies': {
        'credit_card': ['billing_address']
    }
}

# If credit_card exists, billing_address is required
```

### Array Validation

```python
schema = {
    'type': 'array',
    'items': {'type': 'integer'},
    'contains': {'const': 42}  # At least one item must be 42
}

arr = ObjectTree([1, 42, 3], schema=schema)
arr.contains()  # True
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

**Properties:**
- `is_mapping` → bool
- `is_sequence` → bool

### ObjectTreeEncoder

JSON encoder for ObjectTree instances:

```python
json.dumps(obj, cls=ObjectTreeEncoder)
```

## Common Pitfalls

### 1. Use Mapping, Not dict

ObjectTree is NOT a dict subclass:

```python
# ✗ Wrong
if isinstance(data, dict):
    ...

# ✓ Correct
from collections.abc import Mapping
if isinstance(data, Mapping):
    ...
```

### 2. Call Methods Before Type Binding

Schema composition requires method call first:

```python
# ✗ Wrong
obj.field = value  # Validates against original schema

# ✓ Correct
branch = obj.one_of()
branch.field = value  # Validates against selected branch
```

### 3. Circular Imports

Use TYPE_CHECKING for type hints:

```python
from typing import TYPE_CHECKING
if TYPE_CHECKING:
    from schema2object import ObjectTree
```

## Development

```bash
# Clone and set up
git clone https://github.com/woolkingx/schema2object.git
cd schema2object/python
pip install -e .

# Run tests
python -m pytest tests/test_objecttree.py -v

# Run specific test class
python -m pytest tests/test_objecttree.py::TestTypeBinding -v
```

## Requirements

- Python 3.9+
- No external dependencies

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Contributing

Contributions welcome! Please open an issue or submit a pull request.

## See Also

- [schema2object root](../) — multi-language overview
- [Rust implementation](../rust/) — `SchemaValue`, same API in Rust
- [schema-driven-development](https://github.com/woolkingx/schema-driven-development) — full methodology
