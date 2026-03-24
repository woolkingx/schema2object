"""pytest tests for schema2object Python — new API (ObjectTree + validate)."""
import pytest
from schema2object import ObjectTree, validate


# ─── validate() ───────────────────────────────────────────────────────────────

class TestValidate:
    def test_valid_integer(self):
        r = validate(42, {'type': 'integer'})
        assert r['valid'] is True
        assert 'error' not in r

    def test_invalid_type(self):
        r = validate('hello', {'type': 'integer'})
        assert r['valid'] is False
        assert isinstance(r['error'], str)

    def test_valid_string(self):
        assert validate('hi', {'type': 'string', 'minLength': 2})['valid'] is True

    def test_below_minimum(self):
        assert validate(5, {'type': 'integer', 'minimum': 10})['valid'] is False

    def test_valid_object(self):
        schema = {'type': 'object', 'properties': {'name': {'type': 'string'}}, 'required': ['name']}
        assert validate({'name': 'Alice'}, schema)['valid'] is True

    def test_missing_required(self):
        schema = {'type': 'object', 'required': ['name']}
        assert validate({}, schema)['valid'] is False

    def test_boolean_schema_true(self):
        assert validate(42, True)['valid'] is True

    def test_boolean_schema_false(self):
        assert validate(42, False)['valid'] is False

    def test_ref_via_map(self):
        addr = {'type': 'object', 'properties': {'city': {'type': 'string'}}, 'required': ['city']}
        person = {'type': 'object', 'properties': {'address': {'$ref': 'addr'}}}
        assert validate({'address': {'city': 'Taipei'}}, person, {'addr': addr})['valid'] is True

    def test_ref_via_map_invalid(self):
        addr = {'type': 'object', 'properties': {'city': {'type': 'string'}}, 'required': ['city']}
        person = {'type': 'object', 'properties': {'address': {'$ref': 'addr'}}}
        assert validate({'address': {'city': 123}}, person, {'addr': addr})['valid'] is False


# ─── ObjectTree construction ───────────────────────────────────────────────────

class TestConstruction:
    def test_valid_data(self):
        schema = {'type': 'object', 'properties': {'name': {'type': 'string'}}}
        o = ObjectTree({'name': 'Alice'}, schema)
        assert o.name == 'Alice'

    def test_invalid_data_throws(self):
        with pytest.raises(TypeError):
            ObjectTree('hello', {'type': 'integer'})

    def test_defaults_applied(self):
        schema = {
            'type': 'object',
            'properties': {
                'status': {'type': 'string', 'default': 'pending'},
                'count':  {'type': 'integer', 'default': 0},
            }
        }
        o = ObjectTree({}, schema)
        assert o.status == 'pending'
        assert o.count == 0

    def test_boolean_schema_false_throws(self):
        with pytest.raises(TypeError):
            ObjectTree(42, False)

    def test_boolean_schema_true(self):
        o = ObjectTree(42, True)
        assert o.value == 42


# ─── Property access and validation ───────────────────────────────────────────

class TestProperties:
    def setup_method(self):
        self.schema = {
            'type': 'object',
            'properties': {
                'name': {'type': 'string'},
                'age':  {'type': 'integer', 'minimum': 0},
            }
        }

    def test_get_property(self):
        o = ObjectTree({'name': 'Bob'}, self.schema)
        assert o.name == 'Bob'

    def test_set_property_valid(self):
        o = ObjectTree({'name': 'Bob'}, self.schema)
        o.name = 'Alice'
        assert o.name == 'Alice'

    def test_set_property_invalid(self):
        o = ObjectTree({'name': 'Bob'}, self.schema)
        with pytest.raises(TypeError):
            o.name = 123

    def test_set_value(self):
        o = ObjectTree({}, self.schema)
        o.value = {'name': 'Alice', 'age': 30}
        assert o.name == 'Alice'

    def test_set_value_invalid(self):
        o = ObjectTree({}, self.schema)
        with pytest.raises(TypeError):
            o.value = {'age': 'not-a-number'}


# ─── to_dict ──────────────────────────────────────────────────────────────────

class TestToDict:
    def test_excludes_unknown_fields(self):
        schema = {'type': 'object', 'properties': {'name': {'type': 'string'}}}
        o = ObjectTree({'name': 'Alice', 'extra': 'ignored'}, schema)
        assert o.to_dict() == {'name': 'Alice'}

    def test_includes_only_present_defined_fields(self):
        schema = {'type': 'object', 'properties': {
            'a': {'type': 'string'},
            'b': {'type': 'string'},
        }}
        o = ObjectTree({'a': 'x'}, schema)
        assert o.to_dict() == {'a': 'x'}

    def test_scalar(self):
        o = ObjectTree(42, {'type': 'integer'})
        assert o.to_dict() == 42


# ─── value get/set ────────────────────────────────────────────────────────────

class TestValue:
    def test_scalar_get(self):
        o = ObjectTree(42, {'type': 'integer'})
        assert o.value == 42

    def test_object_get(self):
        schema = {'type': 'object', 'properties': {'x': {'type': 'integer'}}}
        o = ObjectTree({'x': 1}, schema)
        assert o.value == {'x': 1}


# ─── Composition methods ──────────────────────────────────────────────────────

class TestOneOf:
    def test_one_of_match(self):
        schema = {'oneOf': [{'type': 'string'}, {'type': 'integer'}]}
        o = ObjectTree(42, schema)
        result = o.one_of()
        assert isinstance(result, ObjectTree)
        assert result.value == 42

    def test_one_of_no_match(self):
        # construction validates oneOf — throws when no branch matches
        schema = {'oneOf': [{'type': 'string'}, {'type': 'array'}]}
        with pytest.raises(TypeError):
            ObjectTree(42, schema)


class TestAnyOf:
    def test_any_of_match(self):
        schema = {'anyOf': [{'type': 'string'}, {'type': 'integer'}]}
        o = ObjectTree(42, schema)
        results = o.any_of()
        assert len(results) == 1
        assert results[0].value == 42

    def test_any_of_no_match(self):
        # construction validates anyOf — throws when no branch matches
        schema = {'anyOf': [{'type': 'string'}, {'type': 'array'}]}
        with pytest.raises(TypeError):
            ObjectTree(42, schema)


class TestAllOf:
    def test_all_of_merge(self):
        schema = {'allOf': [
            {'type': 'object', 'properties': {'a': {'type': 'string'}}},
            {'type': 'object', 'properties': {'b': {'type': 'integer'}}},
        ]}
        o = ObjectTree({'a': 'x', 'b': 1}, schema)
        result = o.all_of()
        assert isinstance(result, ObjectTree)


class TestNotOf:
    def test_not_of_passes(self):
        o = ObjectTree(42, {'not': {'type': 'string'}})
        assert o.not_of() is True

    def test_not_of_fails(self):
        # construction validates not — throws when data matches the not-schema
        with pytest.raises(TypeError):
            ObjectTree('hello', {'not': {'type': 'string'}})


class TestIfThen:
    def test_then_branch(self):
        schema = {
            'if':   {'properties': {'country': {'const': 'US'}}},
            'then': {'properties': {'zip': {'type': 'string'}}},
            'else': {'properties': {'zip': {'type': 'integer'}}},
        }
        o = ObjectTree({'country': 'US', 'zip': '12345'}, schema)
        result = o.if_then()
        assert isinstance(result, ObjectTree)

    def test_no_if_returns_self(self):
        o = ObjectTree(42, {'type': 'integer'})
        assert o.if_then() is o


# ─── project / with_defaults ──────────────────────────────────────────────────

class TestProject:
    def test_project_filters_to_schema(self):
        schema = {'type': 'object', 'properties': {'a': {'type': 'string'}}}
        o = ObjectTree({'a': 'x', 'b': 'y'}, schema)
        p = o.project()
        assert p.to_dict() == {'a': 'x'}

    def test_project_non_object_raises(self):
        o = ObjectTree(42, {'type': 'integer'})
        with pytest.raises(TypeError):
            o.project()


class TestWithDefaults:
    def test_with_defaults_fills_missing(self):
        schema = {'type': 'object', 'properties': {
            'status': {'type': 'string', 'default': 'active'},
        }}
        o = ObjectTree({}, schema)
        result = o.with_defaults()
        assert result.to_dict()['status'] == 'active'


# ─── get_schema / get_extensions ─────────────────────────────────────────────

class TestGetSchema:
    def test_root_schema(self):
        schema = {'type': 'object', 'properties': {'name': {'type': 'string'}}}
        o = ObjectTree({}, schema)
        assert o.get_schema()['type'] == 'object'

    def test_nested_schema(self):
        schema = {'type': 'object', 'properties': {'name': {'type': 'string', 'minLength': 1}}}
        o = ObjectTree({'name': 'x'}, schema)
        s = o.get_schema('name')
        assert s['type'] == 'string'
        assert s['minLength'] == 1

    def test_extensions(self):
        schema = {'type': 'object', 'x-foo': 'bar', 'properties': {}}
        o = ObjectTree({}, schema)
        ext = o.get_extensions()
        assert ext.get('x-foo') == 'bar'


# ─── $ref internal ───────────────────────────────────────────────────────────

class TestRef:
    def test_ref_definitions(self):
        schema = {
            'type': 'object',
            'properties': {'addr': {'$ref': '#/definitions/Address'}},
            'definitions': {
                'Address': {'type': 'object', 'properties': {'city': {'type': 'string'}}}
            }
        }
        o = ObjectTree({'addr': {'city': 'Taipei'}}, schema)
        assert o.value['addr'] == {'city': 'Taipei'}

    def test_ref_invalid(self):
        schema = {
            'type': 'object',
            'properties': {'n': {'$ref': '#/definitions/Int'}},
            'definitions': {'Int': {'type': 'integer'}}
        }
        with pytest.raises(TypeError):
            ObjectTree({'n': 'not-int'}, schema)

    def test_recursive_ref(self):
        schema = {
            'type': 'object',
            'properties': {
                'name': {'type': 'string'},
                'child': {'$ref': '#'},
            }
        }
        o = ObjectTree({'name': 'root', 'child': {'name': 'leaf'}}, schema)
        assert o.value['name'] == 'root'
