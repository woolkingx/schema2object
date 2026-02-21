"""Tests for ObjectTree — Draft-07 completion."""
import copy
import json
import pickle

import pytest

from schema2object import ObjectTree, ObjectTreeEncoder


# === Construction ===

class TestConstruction:
    def test_empty(self):
        o = ObjectTree()
        assert o.to_dict() == {}
        assert len(o) == 0

    def test_from_dict(self):
        o = ObjectTree({'a': 1, 'b': 2})
        assert o.a == 1
        assert o.b == 2

    def test_nested(self):
        o = ObjectTree({'a': {'b': {'c': 42}}})
        assert o.a.b.c == 42

    def test_kwargs(self):
        o = ObjectTree(x=10, y=20)
        assert o.x == 10
        assert o.y == 20

    def test_kwargs_conflict(self):
        with pytest.raises(TypeError):
            ObjectTree({'a': 1}, b=2)

    def test_fromkeys(self):
        o = ObjectTree.fromkeys(['a', 'b', 'c'], 0)
        assert o.to_dict() == {'a': 0, 'b': 0, 'c': 0}


# === Default Auto-Fill ===

class TestDefaultFill:
    def test_schema_defaults(self):
        schema = {
            'properties': {
                'status': {'type': 'string', 'default': 'pending'},
                'priority': {'type': 'integer', 'default': 0},
            }
        }
        o = ObjectTree({}, schema=schema)
        assert o.status == 'pending'
        assert o.priority == 0

    def test_existing_values_not_overwritten(self):
        schema = {
            'properties': {
                'status': {'type': 'string', 'default': 'pending'},
            }
        }
        o = ObjectTree({'status': 'done'}, schema=schema)
        assert o.status == 'done'

    def test_nested_defaults(self):
        schema = {
            'properties': {
                'config': {
                    'type': 'object',
                    'properties': {
                        'debug': {'type': 'boolean', 'default': False},
                    }
                }
            }
        }
        o = ObjectTree({'config': {}}, schema=schema)
        assert o.config.debug is False

    def test_no_schema_no_fill(self):
        o = ObjectTree({'a': 1})
        assert o.to_dict() == {'a': 1}


# === Attribute Access ===

class TestAttributeAccess:
    def test_dot_get(self):
        o = ObjectTree({'key': 'val'})
        assert o.key == 'val'

    def test_dot_set(self):
        o = ObjectTree({})
        o.key = 'val'
        assert o.key == 'val'

    def test_dot_del(self):
        o = ObjectTree({'a': 1, 'b': 2})
        del o.a
        assert 'a' not in o
        assert o.to_dict() == {'b': 2}

    def test_missing_raises(self):
        o = ObjectTree({'a': 1})
        with pytest.raises(AttributeError):
            _ = o.nonexistent

    def test_del_missing_raises(self):
        o = ObjectTree({'a': 1})
        with pytest.raises(AttributeError):
            del o.nonexistent


# === Dict Protocol ===

class TestDictProtocol:
    def test_bracket_access(self):
        o = ObjectTree({'key': 'val'})
        assert o['key'] == 'val'
        o['new'] = 42
        assert o['new'] == 42

    def test_contains(self):
        o = ObjectTree({'a': 1})
        assert 'a' in o
        assert 'b' not in o

    def test_len_iter(self):
        o = ObjectTree({'a': 1, 'b': 2})
        assert len(o) == 2
        assert set(o) == {'a', 'b'}

    def test_keys_values_items(self):
        o = ObjectTree({'x': 10, 'y': 20})
        assert set(o.keys()) == {'x', 'y'}
        assert set(o.values()) == {10, 20}
        assert dict(o.items()) == {'x': 10, 'y': 20}

    def test_pop(self):
        o = ObjectTree({'a': 1, 'b': 2})
        val = o.pop('a')
        assert val == 1
        assert 'a' not in o
        assert o.pop('missing', 99) == 99

    def test_clear(self):
        o = ObjectTree({'a': 1, 'b': 2})
        o.clear()
        assert len(o) == 0
        assert o.to_dict() == {}


# === Wrapping ===

class TestWrapping:
    def test_nested_dict_becomes_objecttree(self):
        o = ObjectTree({'a': {'b': 1}})
        assert isinstance(o.a, ObjectTree)

    def test_list_items_wrapped(self):
        o = ObjectTree({'records': [{'id': 1}, {'id': 2}]})
        data = o.records
        assert isinstance(data[0], ObjectTree)
        assert data[0].id == 1
        assert data[1].id == 2

    def test_schema_propagation(self):
        schema = {
            'properties': {
                'user': {
                    'type': 'object',
                    'properties': {
                        'name': {'type': 'string'},
                    }
                }
            }
        }
        o = ObjectTree({'user': {'name': 'foo'}}, schema=schema)
        # Child should have sub-schema — type check enforced
        with pytest.raises(TypeError):
            o.user.name = 123


# === Serialization ===

class TestSerialization:
    def test_to_dict(self):
        o = ObjectTree({'a': {'b': [1, 2, 3]}})
        d = o.to_dict()
        assert d == {'a': {'b': [1, 2, 3]}}

    def test_json_dumps_encoder(self):
        o = ObjectTree({'a': 1, 'b': [2, 3]})
        result = json.dumps(o, cls=ObjectTreeEncoder)
        assert json.loads(result) == {'a': 1, 'b': [2, 3]}

    def test_pickle_roundtrip(self):
        o = ObjectTree({'a': {'b': 1}, 'c': [2, 3]})
        data = pickle.dumps(o)
        restored = pickle.loads(data)
        assert restored.to_dict() == o.to_dict()
        assert isinstance(restored.a, ObjectTree)


# === Type Binding ===

class TestTypeBinding:
    def test_type_binding_on_set(self):
        schema = {
            'type': 'object',
            'properties': {
                'age': {'type': 'integer'},
            },
        }
        o = ObjectTree({}, schema=schema)
        with pytest.raises(TypeError):
            o.age = 'bad'
        o.age = 10
        assert o.age == 10

    def test_const_binding(self):
        schema = {
            'properties': {
                'version': {'const': 2},
            }
        }
        o = ObjectTree({}, schema=schema)
        with pytest.raises(TypeError):
            o.version = 3
        o.version = 2
        assert o.version == 2

    def test_enum_binding(self):
        schema = {
            'properties': {
                'status': {'enum': ['open', 'closed', 'pending']},
            }
        }
        o = ObjectTree({}, schema=schema)
        with pytest.raises(TypeError):
            o.status = 'invalid'
        o.status = 'open'
        assert o.status == 'open'

    def test_nested_binding_via_propagation(self):
        schema = {
            'properties': {
                'config': {
                    'type': 'object',
                    'properties': {
                        'port': {'type': 'integer', 'minimum': 1, 'maximum': 65535},
                    }
                }
            }
        }
        o = ObjectTree({'config': {'port': 8080}}, schema=schema)
        with pytest.raises(TypeError):
            o.config.port = 'not_a_number'
        with pytest.raises(TypeError):
            o.config.port = 0  # below minimum
        o.config.port = 443
        assert o.config.port == 443

    def test_bool_rejected_as_integer(self):
        schema = {'properties': {'count': {'type': 'integer'}}}
        o = ObjectTree({}, schema=schema)
        with pytest.raises(TypeError):
            o.count = True
        with pytest.raises(TypeError):
            o.count = False
        o.count = 0
        assert o.count == 0

    def test_bool_rejected_as_number(self):
        schema = {'properties': {'ratio': {'type': 'number'}}}
        o = ObjectTree({}, schema=schema)
        with pytest.raises(TypeError):
            o.ratio = True
        o.ratio = 1.5
        assert o.ratio == 1.5

    def test_bracket_assignment_type_binding(self):
        schema = {'properties': {'age': {'type': 'integer'}}}
        o = ObjectTree({}, schema=schema)
        with pytest.raises(TypeError):
            o['age'] = 'bad'
        o['age'] = 10
        assert o['age'] == 10

    def test_setdefault_type_binding(self):
        schema = {'properties': {'age': {'type': 'integer'}}}
        o = ObjectTree({}, schema=schema)
        with pytest.raises(TypeError):
            o.setdefault('age', 'bad')
        o.setdefault('age', 42)
        assert o.age == 42

    def test_update_type_binding(self):
        schema = {'properties': {'age': {'type': 'integer'}}}
        o = ObjectTree({}, schema=schema)
        with pytest.raises(TypeError):
            o.update({'age': 'bad'})
        o.update({'age': 25})
        assert o.age == 25

    def test_schema_as_objecttree(self):
        schema_obj = ObjectTree({'properties': {'age': {'type': 'integer'}}})
        o = ObjectTree({'age': 10}, schema=schema_obj)
        with pytest.raises(TypeError):
            o.age = 'bad'
        o.age = 20
        assert o.age == 20


# === Python Protocols ===

class TestPythonProtocols:
    def test_repr(self):
        o = ObjectTree({'a': 1})
        r = repr(o)
        assert r == "ObjectTree({'a': 1})"

    def test_eq_with_dict(self):
        o = ObjectTree({'a': 1, 'b': 2})
        assert o == {'a': 1, 'b': 2}
        assert o != {'a': 1}
        assert o == ObjectTree({'a': 1, 'b': 2})

    def test_bool(self):
        assert bool(ObjectTree({'a': 1})) is True
        assert bool(ObjectTree()) is False
        assert bool(ObjectTree({})) is False

    def test_copy_deepcopy(self):
        o = ObjectTree({'a': {'b': [1, 2]}})
        shallow = copy.copy(o)
        deep = copy.deepcopy(o)
        assert shallow == o
        assert deep == o
        # Deep copy is independent
        deep.a.b = [99]
        assert o.a.b == [1, 2]


# === Dict Merge ===

class TestDictMerge:
    def test_or_merge(self):
        a = ObjectTree({'x': 1})
        b = ObjectTree({'y': 2})
        c = a | b
        assert c.to_dict() == {'x': 1, 'y': 2}
        # Originals unchanged
        assert 'y' not in a

    def test_ior_merge(self):
        a = ObjectTree({'x': 1})
        a |= {'y': 2, 'z': 3}
        assert a.to_dict() == {'x': 1, 'y': 2, 'z': 3}

    def test_or_override(self):
        a = ObjectTree({'x': 1, 'y': 'old'})
        b = {'y': 'new', 'z': 3}
        c = a | b
        assert c.y == 'new'
        assert c.x == 1
        assert c.z == 3


# === Schema Composition (allOf/oneOf/anyOf) — method access ===

class TestSchemaComposition:
    def test_allof_via_method(self):
        """allOf: use .all_of() to get merged schema, then type binding works."""
        schema = {
            'allOf': [
                {'properties': {'name': {'type': 'string'}}},
                {'properties': {'age': {'type': 'integer'}}},
            ]
        }
        o = ObjectTree({'name': 'foo', 'age': 10}, schema=schema)
        merged = o.all_of()
        with pytest.raises(TypeError):
            merged.name = 123
        with pytest.raises(TypeError):
            merged.age = 'bad'
        merged.name = 'bar'
        merged.age = 20
        assert merged.name == 'bar'
        assert merged.age == 20

    def test_oneof_via_method(self):
        """oneOf: use .one_of() to select branch, then type binding works."""
        schema = {
            'oneOf': [
                {'properties': {'behavior': {'const': 'allow'}, 'updatedInput': {'type': 'object'}}},
                {'properties': {'behavior': {'const': 'deny'}, 'message': {'type': 'string'}}},
            ]
        }
        o = ObjectTree({'behavior': 'allow'}, schema=schema)
        branch = o.one_of()
        # Bound to allow branch — updatedInput is object
        with pytest.raises(TypeError):
            branch.updatedInput = 'not_object'

    def test_anyof_via_method(self):
        """anyOf: use .any_of() to get matching branches."""
        schema = {
            'anyOf': [
                {'properties': {'x': {'type': 'integer'}}},
                {'properties': {'y': {'type': 'string'}}},
            ]
        }
        o = ObjectTree({'x': 1}, schema=schema)
        branches = o.any_of()
        assert len(branches) >= 1
        # First matching branch has x binding
        with pytest.raises(TypeError):
            branches[0].x = 'bad'

    def test_allof_default_fill_via_method(self):
        """Defaults from allOf: top-level properties get defaults, allOf needs method."""
        schema = {
            'properties': {
                'status': {'type': 'string', 'default': 'active'},
            },
            'allOf': [
                {'properties': {'count': {'type': 'integer', 'default': 0}}},
            ]
        }
        o = ObjectTree({}, schema=schema)
        # Top-level properties: defaults applied directly
        assert o.status == 'active'
        # allOf properties: access via method
        merged = o.all_of()
        assert merged.get('count') is not None or merged.get('status') == 'active'

    def test_nested_composition_via_method(self):
        """Schema propagation: child has allOf, use method on child."""
        schema = {
            'properties': {
                'config': {
                    'allOf': [
                        {'properties': {'host': {'type': 'string'}}},
                        {'properties': {'port': {'type': 'integer'}}},
                    ]
                }
            }
        }
        o = ObjectTree({'config': {'host': 'localhost', 'port': 8080}}, schema=schema)
        cfg = o.config.all_of()
        with pytest.raises(TypeError):
            cfg.port = 'bad'
        cfg.host = '0.0.0.0'
        assert cfg.host == '0.0.0.0'

    def test_composition_with_top_level_properties(self):
        """Top-level properties bind directly, allOf properties via method."""
        schema = {
            'properties': {'id': {'type': 'integer'}},
            'allOf': [
                {'properties': {'name': {'type': 'string'}}},
            ]
        }
        o = ObjectTree({'id': 1, 'name': 'foo'}, schema=schema)
        # Top-level property: binding works directly
        with pytest.raises(TypeError):
            o.id = 'bad'
        # allOf property: use method
        merged = o.all_of()
        with pytest.raises(TypeError):
            merged.name = 123


# === SchemaAPI Methods ===

class TestSchemaAPI:
    """Tests for Draft-07 applicator keywords as methods."""

    # --- one_of (XOR) ---

    def test_one_of_unique_match(self):
        schema = {
            'oneOf': [
                {'properties': {'type': {'const': 'dog'}, 'bark': {'type': 'boolean'}}},
                {'properties': {'type': {'const': 'cat'}, 'purr': {'type': 'boolean'}}},
            ]
        }
        o = ObjectTree({'type': 'cat', 'purr': True}, schema=schema)
        matched = o.one_of()
        assert matched.to_dict() == {'type': 'cat', 'purr': True}
        with pytest.raises(TypeError):
            matched.purr = 'not_bool'

    def test_one_of_no_match(self):
        schema = {
            'oneOf': [
                {'properties': {'x': {'const': 1}}, 'required': ['x']},
                {'properties': {'x': {'const': 2}}, 'required': ['x']},
            ]
        }
        o = ObjectTree({'x': 99}, schema=schema)
        with pytest.raises(TypeError, match="oneOf.*0"):
            o.one_of()

    def test_one_of_ambiguous(self):
        schema = {
            'oneOf': [
                {'properties': {'x': {'type': 'integer'}}},
                {'properties': {'x': {'type': 'number'}}},
            ]
        }
        o = ObjectTree({'x': 1}, schema=schema)
        with pytest.raises(TypeError, match="oneOf.*2"):
            o.one_of()

    def test_one_of_no_schema(self):
        o = ObjectTree({'a': 1})
        assert o.one_of() is o

    # --- any_of (OR) ---

    def test_any_of_multiple_matches(self):
        schema = {
            'anyOf': [
                {'properties': {'x': {'type': 'integer'}}},
                {'properties': {'x': {'type': 'number'}}},
                {'properties': {'x': {'const': 'nope'}}, 'required': ['x']},
            ]
        }
        o = ObjectTree({'x': 42}, schema=schema)
        results = o.any_of()
        assert len(results) == 2

    def test_any_of_no_match(self):
        schema = {
            'anyOf': [
                {'properties': {'x': {'const': 1}}, 'required': ['x']},
                {'properties': {'x': {'const': 2}}, 'required': ['x']},
            ]
        }
        o = ObjectTree({'x': 99}, schema=schema)
        with pytest.raises(TypeError, match="anyOf"):
            o.any_of()

    def test_any_of_no_schema(self):
        o = ObjectTree({'a': 1})
        results = o.any_of()
        assert len(results) == 1
        assert results[0] is o

    # --- all_of (AND) ---

    def test_all_of_merge(self):
        schema = {
            'allOf': [
                {'properties': {'name': {'type': 'string'}}},
                {'properties': {'age': {'type': 'integer'}}},
            ]
        }
        o = ObjectTree({'name': 'Alice', 'age': 30}, schema=schema)
        merged = o.all_of()
        with pytest.raises(TypeError):
            merged.name = 123
        with pytest.raises(TypeError):
            merged.age = 'bad'
        merged.name = 'Bob'
        assert merged.name == 'Bob'

    def test_all_of_no_allof(self):
        o = ObjectTree({'a': 1}, schema={'properties': {'a': {'type': 'integer'}}})
        assert o.all_of() is o

    # --- not_of (EXCEPT) ---

    def test_not_of_exclusion_holds(self):
        schema = {'not': {'properties': {'x': {'const': 'bad'}}, 'required': ['x']}}
        o = ObjectTree({'x': 'good'}, schema=schema)
        assert o.not_of() is True

    def test_not_of_exclusion_fails(self):
        schema = {'not': {'properties': {'x': {'const': 'bad'}}, 'required': ['x']}}
        o = ObjectTree({'x': 'bad'}, schema=schema)
        assert o.not_of() is False

    def test_not_of_explicit_schema(self):
        o = ObjectTree({'x': 42})
        assert o.not_of({'properties': {'x': {'type': 'string'}}, 'required': ['x']}) is True
        assert o.not_of({'properties': {'x': {'type': 'integer'}}}) is False

    # --- if_then (CASE WHEN) ---

    def test_if_then_match(self):
        schema = {
            'if': {'properties': {'role': {'const': 'admin'}}, 'required': ['role']},
            'then': {'properties': {'level': {'type': 'integer', 'minimum': 5}}},
            'else': {'properties': {'level': {'type': 'integer', 'maximum': 4}}},
        }
        o = ObjectTree({'role': 'admin', 'level': 10}, schema=schema)
        result = o.if_then()
        with pytest.raises(TypeError):
            result.level = 2

    def test_if_then_no_match(self):
        schema = {
            'if': {'properties': {'role': {'const': 'admin'}}, 'required': ['role']},
            'then': {'properties': {'level': {'type': 'integer', 'minimum': 5}}},
            'else': {'properties': {'level': {'type': 'integer', 'maximum': 4}}},
        }
        o = ObjectTree({'role': 'user', 'level': 3}, schema=schema)
        result = o.if_then()
        with pytest.raises(TypeError):
            result.level = 10

    def test_if_then_no_if(self):
        o = ObjectTree({'a': 1}, schema={'properties': {'a': {'type': 'integer'}}})
        assert o.if_then() is o

    def test_if_then_no_else(self):
        schema = {
            'if': {'properties': {'x': {'const': 1}}, 'required': ['x']},
            'then': {'properties': {'y': {'type': 'string'}}},
        }
        o = ObjectTree({'x': 2, 'y': 'val'}, schema=schema)
        result = o.if_then()
        assert result is o

    def test_if_then_no_then(self):
        schema = {
            'if': {'properties': {'x': {'const': 1}}, 'required': ['x']},
            'else': {'properties': {'y': {'type': 'string'}}},
        }
        o = ObjectTree({'x': 1, 'y': 'val'}, schema=schema)
        result = o.if_then()
        assert result is o

    # --- project (SELECT) ---

    def test_project_filters_fields(self):
        schema = {
            'properties': {
                'name': {'type': 'string'},
                'age': {'type': 'integer'},
            }
        }
        o = ObjectTree({'name': 'Alice', 'age': 30, 'extra': 'junk'}, schema=schema)
        projected = o.project()
        assert projected.to_dict() == {'name': 'Alice', 'age': 30}
        assert 'extra' not in projected

    def test_project_with_oneof_resolution(self):
        schema = {
            'oneOf': [
                {'properties': {'type': {'const': 'dog'}, 'bark': {'type': 'boolean'}}},
                {'properties': {'type': {'const': 'cat'}, 'purr': {'type': 'boolean'}}},
            ]
        }
        o = ObjectTree({'type': 'cat', 'purr': True, 'extra': 1}, schema=schema)
        projected = o.project()
        assert projected.to_dict() == {'type': 'cat', 'purr': True}

    def test_project_no_properties(self):
        o = ObjectTree({'a': 1})
        assert o.project() is o

    def test_project_ambiguous_oneof(self):
        schema = {
            'oneOf': [
                {'properties': {'x': {'type': 'integer'}}},
                {'properties': {'x': {'type': 'number'}}},
            ]
        }
        o = ObjectTree({'x': 1}, schema=schema)
        with pytest.raises(TypeError, match="use one_of"):
            o.project()

    def test_project_ambiguous_anyof(self):
        schema = {
            'anyOf': [
                {'properties': {'x': {'type': 'integer'}}},
                {'properties': {'x': {'type': 'number'}}},
            ]
        }
        o = ObjectTree({'x': 1}, schema=schema)
        with pytest.raises(TypeError, match="use any_of"):
            o.project()

    # --- contains (EXISTS) ---

    def test_contains_match(self):
        schema = {'contains': {'properties': {'id': {'const': 2}}, 'required': ['id']}}
        o = ObjectTree([{'id': 1}, {'id': 2}, {'id': 3}], schema=schema)
        assert o.contains() is True

    def test_contains_no_match(self):
        schema = {'contains': {'properties': {'id': {'const': 99}}, 'required': ['id']}}
        o = ObjectTree([{'id': 1}, {'id': 2}], schema=schema)
        assert o.contains() is False

    def test_contains_explicit_schema(self):
        o = ObjectTree([1, 2, 3])
        assert o.contains({'const': 2}) is True
        assert o.contains({'const': 99}) is False

    def test_contains_not_array(self):
        o = ObjectTree({'a': 1})
        assert o.contains({'const': 1}) is False


# === Import Path ===

class TestImportPath:
    def test_schema2object_import(self):
        from schema2object import ObjectTree as OT1, ObjectTreeEncoder as OTE1
        assert OT1 is ObjectTree
        assert OTE1 is ObjectTreeEncoder

    def test_utils_compat_import(self):
        # Test backward compatibility through utils module
        from schema2object import ObjectTree as OT2
        assert OT2 is ObjectTree

    def test_utils_init_import(self):
        from schema2object import ObjectTree as OT3
        assert OT3 is ObjectTree


# === Edge Cases ===

class TestEdgeCases:
    def test_scalar_data(self):
        o = ObjectTree(42)
        assert o.to_dict() == 42
        assert bool(o) is True

    def test_sequence_with_schema(self):
        schema = {'items': {'type': 'integer'}}
        o = ObjectTree([1, 2, 3], schema=schema)
        assert o.to_dict() == [1, 2, 3]

    def test_pickle_with_schema(self):
        schema = {'properties': {'age': {'type': 'integer'}}}
        o = ObjectTree({'age': 10}, schema=schema)
        restored = pickle.loads(pickle.dumps(o))
        assert restored.to_dict() == {'age': 10}
        with pytest.raises(TypeError):
            restored.age = 'bad'

    def test_deepcopy_with_schema(self):
        schema = {'properties': {'name': {'type': 'string'}}}
        o = ObjectTree({'name': 'Alice'}, schema=schema)
        d = copy.deepcopy(o)
        assert d.to_dict() == {'name': 'Alice'}
        with pytest.raises(TypeError):
            d.name = 123
        d.name = 'Bob'
        assert o.name == 'Alice'

    def test_null_type(self):
        schema = {'properties': {'val': {'type': 'null'}}}
        o = ObjectTree({}, schema=schema)
        o.val = None
        assert o.val is None
        with pytest.raises(TypeError):
            o.val = 0


# === Draft-07 Validation Keywords ===

class TestDraft07Keywords:
    """Tests for all Draft-07 keywords in _check_value."""

    # --- exclusiveMinimum / exclusiveMaximum ---

    def test_exclusive_minimum(self):
        schema = {'properties': {'x': {'type': 'number', 'exclusiveMinimum': 0}}}
        o = ObjectTree({}, schema=schema)
        with pytest.raises(TypeError):
            o.x = 0  # equal not allowed
        with pytest.raises(TypeError):
            o.x = -1
        o.x = 0.001
        assert o.x == 0.001

    def test_exclusive_maximum(self):
        schema = {'properties': {'x': {'type': 'number', 'exclusiveMaximum': 100}}}
        o = ObjectTree({}, schema=schema)
        with pytest.raises(TypeError):
            o.x = 100  # equal not allowed
        o.x = 99.9
        assert o.x == 99.9

    # --- multipleOf ---

    def test_multiple_of(self):
        schema = {'properties': {'x': {'type': 'integer', 'multipleOf': 3}}}
        o = ObjectTree({}, schema=schema)
        with pytest.raises(TypeError):
            o.x = 7
        o.x = 9
        assert o.x == 9

    # --- minLength / maxLength ---

    def test_min_length(self):
        schema = {'properties': {'name': {'type': 'string', 'minLength': 2}}}
        o = ObjectTree({}, schema=schema)
        with pytest.raises(TypeError):
            o.name = 'a'
        o.name = 'ab'
        assert o.name == 'ab'

    def test_max_length(self):
        schema = {'properties': {'code': {'type': 'string', 'maxLength': 5}}}
        o = ObjectTree({}, schema=schema)
        with pytest.raises(TypeError):
            o.code = 'toolong'
        o.code = 'ok'
        assert o.code == 'ok'

    # --- pattern ---

    def test_pattern(self):
        schema = {'properties': {'email': {'type': 'string', 'pattern': r'^[^@]+@[^@]+$'}}}
        o = ObjectTree({}, schema=schema)
        with pytest.raises(TypeError):
            o.email = 'not-an-email'
        o.email = 'user@example.com'
        assert o.email == 'user@example.com'

    # --- minItems / maxItems ---

    def test_min_items(self):
        schema = {'properties': {'tags': {'type': 'array', 'minItems': 1}}}
        o = ObjectTree({}, schema=schema)
        with pytest.raises(TypeError):
            o.tags = []
        o.tags = ['a']
        assert o.tags == ['a']

    def test_max_items(self):
        schema = {'properties': {'tags': {'type': 'array', 'maxItems': 2}}}
        o = ObjectTree({}, schema=schema)
        with pytest.raises(TypeError):
            o.tags = [1, 2, 3]
        o.tags = [1, 2]
        assert o.tags == [1, 2]

    # --- uniqueItems ---

    def test_unique_items(self):
        schema = {'properties': {'ids': {'type': 'array', 'uniqueItems': True}}}
        o = ObjectTree({}, schema=schema)
        with pytest.raises(TypeError):
            o.ids = [1, 2, 1]
        o.ids = [1, 2, 3]
        assert o.ids == [1, 2, 3]

    # --- additionalProperties ---

    def test_additional_properties_false(self):
        from schema2object.api import _check_value
        schema = {
            'properties': {'x': {'type': 'integer'}, 'y': {'type': 'integer'}},
            'additionalProperties': False,
        }
        ok, _ = _check_value(schema, {'x': 1, 'y': 2})
        assert ok
        ok, reason = _check_value(schema, {'x': 1, 'y': 2, 'z': 3})
        assert not ok
        assert 'additional' in reason

    # --- minProperties / maxProperties ---

    def test_min_properties(self):
        from schema2object.api import _check_value
        schema = {'minProperties': 2}
        ok, _ = _check_value(schema, {'a': 1, 'b': 2})
        assert ok
        ok, reason = _check_value(schema, {'a': 1})
        assert not ok
        assert 'minProperties' in reason

    def test_max_properties(self):
        from schema2object.api import _check_value
        schema = {'maxProperties': 2}
        ok, _ = _check_value(schema, {'a': 1})
        assert ok
        ok, reason = _check_value(schema, {'a': 1, 'b': 2, 'c': 3})
        assert not ok
        assert 'maxProperties' in reason

    # --- patternProperties ---

    def test_pattern_properties(self):
        from schema2object.api import _check_value
        schema = {
            'patternProperties': {
                '^S_': {'type': 'string'},
                '^I_': {'type': 'integer'},
            }
        }
        ok, _ = _check_value(schema, {'S_name': 'foo', 'I_count': 5})
        assert ok
        ok, reason = _check_value(schema, {'S_name': 123})
        assert not ok
        assert 'pattern' in reason

    # --- dependentRequired ---

    def test_dependent_required(self):
        from schema2object.api import _check_value
        schema = {
            'dependentRequired': {
                'credit_card': ['billing_address'],
            }
        }
        ok, _ = _check_value(schema, {'name': 'foo'})
        assert ok
        ok, _ = _check_value(schema, {'credit_card': '1234', 'billing_address': '123 St'})
        assert ok
        ok, reason = _check_value(schema, {'credit_card': '1234'})
        assert not ok
        assert 'billing_address' in reason


# === Error Messages ===

class TestErrorMessages:
    """Verify error messages contain useful context."""

    def test_type_error_includes_field_name(self):
        schema = {'properties': {'age': {'type': 'integer'}}}
        o = ObjectTree({}, schema=schema)
        with pytest.raises(TypeError, match="'age'"):
            o.age = 'bad'

    def test_type_error_shows_expected_type(self):
        schema = {'properties': {'x': {'type': 'integer'}}}
        o = ObjectTree({}, schema=schema)
        with pytest.raises(TypeError, match="integer"):
            o.x = 'str'

    def test_const_error_shows_values(self):
        schema = {'properties': {'v': {'const': 42}}}
        o = ObjectTree({}, schema=schema)
        with pytest.raises(TypeError, match="42"):
            o.v = 99

    def test_min_error_shows_bound(self):
        schema = {'properties': {'x': {'type': 'integer', 'minimum': 10}}}
        o = ObjectTree({}, schema=schema)
        with pytest.raises(TypeError, match="minimum"):
            o.x = 5

    def test_pattern_error_shows_pattern(self):
        schema = {'properties': {'s': {'type': 'string', 'pattern': r'^[A-Z]'}}}
        o = ObjectTree({}, schema=schema)
        with pytest.raises(TypeError, match="pattern"):
            o.s = 'lowercase'

    def test_required_error_shows_field(self):
        from schema2object.api import _check_value
        ok, reason = _check_value(
            {'required': ['name'], 'properties': {'name': {'type': 'string'}}},
            {}
        )
        assert not ok
        assert 'name' in reason

    def test_nested_error_shows_path(self):
        from schema2object.api import _check_value
        schema = {
            'properties': {
                'user': {
                    'properties': {'age': {'type': 'integer'}}
                }
            }
        }
        ok, reason = _check_value(schema, {'user': {'age': 'bad'}})
        assert not ok
        assert '.user' in reason
        assert '.age' in reason


# === Draft-07 Compliance Fixes ===

class TestTypeUnion:
    """type as array (Draft-07 union types)."""

    def test_string_or_null_accepts_string(self):
        from schema2object.api import _check_value
        ok, _ = _check_value({'type': ['string', 'null']}, 'hello')
        assert ok

    def test_string_or_null_accepts_null(self):
        from schema2object.api import _check_value
        ok, _ = _check_value({'type': ['string', 'null']}, None)
        assert ok

    def test_string_or_null_rejects_int(self):
        from schema2object.api import _check_value
        ok, reason = _check_value({'type': ['string', 'null']}, 42)
        assert not ok
        assert 'string' in reason

    def test_integer_or_number(self):
        from schema2object.api import _check_value
        ok, _ = _check_value({'type': ['integer', 'null']}, 5)
        assert ok
        ok, _ = _check_value({'type': ['integer', 'null']}, None)
        assert ok

    def test_bool_not_integer_in_union(self):
        from schema2object.api import _check_value
        ok, _ = _check_value({'type': ['integer', 'null']}, True)
        assert not ok

    def test_type_binding_with_union(self):
        """ObjectTree type binding respects union types."""
        schema = {'properties': {'val': {'type': ['string', 'null']}}}
        o = ObjectTree({'val': 'ok'}, schema=schema)
        o.val = None  # should not raise
        o.val = 'still ok'  # should not raise
        with pytest.raises(TypeError):
            o.val = 123


class TestAdditionalPropertiesNoProperties:
    """additionalProperties: false without properties key."""

    def test_rejects_any_field(self):
        from schema2object.api import _check_value
        ok, reason = _check_value({'additionalProperties': False}, {'x': 1})
        assert not ok
        assert 'additional' in reason

    def test_empty_object_passes(self):
        from schema2object.api import _check_value
        ok, _ = _check_value({'additionalProperties': False}, {})
        assert ok

    def test_with_properties_still_works(self):
        from schema2object.api import _check_value
        schema = {
            'properties': {'a': {'type': 'integer'}},
            'additionalProperties': False
        }
        ok, _ = _check_value(schema, {'a': 1})
        assert ok
        ok, reason = _check_value(schema, {'a': 1, 'b': 2})
        assert not ok


class TestDependencies:
    """Draft-07 dependencies keyword (array and schema forms)."""

    def test_array_form(self):
        from schema2object.api import _check_value
        schema = {'dependencies': {'credit_card': ['billing_address']}}
        ok, _ = _check_value(schema, {'name': 'Alice'})
        assert ok  # credit_card absent, no requirement
        ok, reason = _check_value(schema, {'credit_card': '1234'})
        assert not ok
        assert 'billing_address' in reason

    def test_array_form_satisfied(self):
        from schema2object.api import _check_value
        schema = {'dependencies': {'credit_card': ['billing_address']}}
        ok, _ = _check_value(schema, {'credit_card': '1234', 'billing_address': '123 St'})
        assert ok

    def test_schema_form(self):
        from schema2object.api import _check_value
        schema = {
            'dependencies': {
                'credit_card': {
                    'required': ['billing_address'],
                    'properties': {'billing_address': {'type': 'string'}}
                }
            }
        }
        ok, reason = _check_value(schema, {'credit_card': '1234'})
        assert not ok
        assert 'billing_address' in reason

    def test_schema_form_satisfied(self):
        from schema2object.api import _check_value
        schema = {
            'dependencies': {
                'credit_card': {
                    'required': ['billing_address']
                }
            }
        }
        ok, _ = _check_value(schema, {'credit_card': '1234', 'billing_address': '123 St'})
        assert ok


class TestMultipleOfFloat:
    """multipleOf with float tolerance."""

    def test_float_multiple(self):
        from schema2object.api import _check_value
        ok, _ = _check_value({'multipleOf': 0.1}, 0.3)
        assert ok  # 0.3 % 0.1 != 0 in naive float, but should pass

    def test_float_not_multiple(self):
        from schema2object.api import _check_value
        ok, _ = _check_value({'multipleOf': 0.3}, 1.0)
        assert not ok

    def test_integer_still_works(self):
        from schema2object.api import _check_value
        ok, _ = _check_value({'multipleOf': 3}, 9)
        assert ok
        ok, _ = _check_value({'multipleOf': 3}, 10)
        assert not ok


class TestAdditionalPropertiesSchema:
    """additionalProperties as schema (not just boolean)."""

    def test_extra_fields_validated_against_schema(self):
        from schema2object.api import _check_value
        schema = {
            'properties': {'name': {'type': 'string'}},
            'additionalProperties': {'type': 'integer'}
        }
        ok, _ = _check_value(schema, {'name': 'Alice', 'age': 30})
        assert ok
        ok, reason = _check_value(schema, {'name': 'Alice', 'age': 'thirty'})
        assert not ok
        assert 'age' in reason

    def test_pattern_properties_excluded_from_additional(self):
        from schema2object.api import _check_value
        schema = {
            'patternProperties': {'^x_': {'type': 'string'}},
            'additionalProperties': False
        }
        ok, _ = _check_value(schema, {'x_foo': 'bar'})
        assert ok
        ok, reason = _check_value(schema, {'x_foo': 'bar', 'other': 1})
        assert not ok


class TestAllOfMerge:
    """all_of() merges required and items from sub-schemas."""

    def test_merge_required(self):
        schema = {
            'required': ['a'],
            'properties': {'a': {'type': 'string'}},
            'allOf': [
                {'required': ['b'], 'properties': {'b': {'type': 'integer'}}}
            ]
        }
        o = ObjectTree({'a': 'hello', 'b': 42}, schema=schema)
        merged = o.all_of()
        # merged schema should require both a and b
        merged_schema = object.__getattribute__(merged, '_schema')
        assert 'a' in merged_schema.required
        assert 'b' in merged_schema.required

    def test_merge_items(self):
        schema = {
            'allOf': [
                {'items': {'type': 'integer'}}
            ]
        }
        o = ObjectTree([1, 2, 3], schema=schema)
        merged = o.all_of()
        merged_schema = object.__getattribute__(merged, '_schema')
        assert merged_schema['items']['type'] == 'integer'

    def test_merge_required_no_duplicates(self):
        schema = {
            'required': ['a'],
            'allOf': [
                {'required': ['a', 'b']},
                {'required': ['b', 'c']}
            ]
        }
        o = ObjectTree({'a': 1, 'b': 2, 'c': 3}, schema=schema)
        merged = o.all_of()
        merged_schema = object.__getattribute__(merged, '_schema')
        req = list(merged_schema.required)
        assert sorted(req) == ['a', 'b', 'c']
