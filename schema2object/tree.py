"""ObjectTree - JSON 1:1 structure mapping + schema as object + methods as access path.

Structure layer: data + schema (both ObjectTree) + dot-access + protocols.
Logic layer provided by SchemaAPI mixin (api.py).
"""
from __future__ import annotations

import copy
import json
from collections.abc import Mapping, MutableMapping, Sequence
from typing import Any, Dict, ItemsView, Iterator, KeysView, List, Union, ValuesView

from .api import SchemaAPI


class ObjectTree(SchemaAPI, MutableMapping):
    """Dict wrapper with dot-access. Schema is also an ObjectTree.

    JSON Schema keyword = attribute (dot-access gets raw value).
    JSON Schema logic   = method (method call gets correct value).
    """

    __slots__ = ('_data', '_schema')

    def __init__(self, data: Union[Mapping, Sequence, Any] = None, *, schema: Mapping | None = None, **kwargs):
        # schema -> ObjectTree (if not already)
        if schema is not None and not isinstance(schema, ObjectTree):
            schema = ObjectTree(schema)
        object.__setattr__(self, '_schema', schema)

        if data is None:
            data = kwargs if kwargs else {}
        elif kwargs:
            raise TypeError("Cannot pass both positional data and keyword arguments")

        if isinstance(data, Mapping):
            if schema:
                data = self._apply_defaults(data, schema)

            store = {}
            for k, v in (data.items() if isinstance(data, Mapping) else dict(data).items()):
                store[k] = self._wrap(k, v)
            object.__setattr__(self, '_data', store)
        elif isinstance(data, Sequence) and not isinstance(data, (str, bytes, bytearray)):
            object.__setattr__(self, '_data', [self._wrap(None, item) for item in data])
        else:
            object.__setattr__(self, '_data', data)

    @staticmethod
    def _apply_defaults(data: Mapping, schema: 'ObjectTree') -> dict | Mapping:
        """Fill missing keys from schema.properties defaults."""
        props = schema.get('properties')
        if not isinstance(props, Mapping):
            return data
        defaults = {
            p: ps['default']
            for p, ps in props.items()
            if isinstance(ps, Mapping) and 'default' in ps and p not in data
        }
        if not defaults:
            return data
        if isinstance(data, ObjectTree):
            data = dict(data.to_dict())
        elif not isinstance(data, dict):
            data = dict(data)
        data.update(defaults)
        return data

    # --- Wrap/unwrap ---

    def _wrap(self, key: str | int | None, value: Any) -> Any:
        """Wrap value, propagating sub-schema if available."""
        if isinstance(value, ObjectTree):
            return value
        sub_schema = self._child_schema(key)
        if isinstance(value, Mapping):
            return ObjectTree(value, schema=sub_schema)
        if isinstance(value, Sequence) and not isinstance(value, (str, bytes, bytearray)):
            items_schema = None
            if isinstance(sub_schema, ObjectTree):
                items_schema = sub_schema.get('items')
            elif isinstance(sub_schema, Mapping):
                items_schema = sub_schema.get('items')
            return [self._wrap_static(item, items_schema) for item in value]
        return value

    @staticmethod
    def _wrap_static(value: Any, schema: Mapping | None = None) -> Any:
        """Wrap without parent context (for list items)."""
        if isinstance(value, ObjectTree):
            return value
        if isinstance(value, Mapping):
            return ObjectTree(value, schema=schema)
        if isinstance(value, Sequence) and not isinstance(value, (str, bytes, bytearray)):
            items_schema = None
            if isinstance(schema, Mapping):
                items_schema = schema.get('items')
            return [ObjectTree._wrap_static(item, items_schema) for item in value]
        return value

    def _child_schema(self, key: str | int | None) -> Mapping | None:
        """Get sub-schema for a child key via schema attributes."""
        schema = object.__getattribute__(self, '_schema')
        if not isinstance(schema, ObjectTree):
            return None
        if isinstance(key, str):
            props = schema.get('properties')
            if isinstance(props, Mapping) and key in props:
                return props[key]
        elif isinstance(key, int) or key is None:
            items = schema.get('items')
            if isinstance(items, Mapping):
                return items
        return None

    @staticmethod
    def _unwrap(value: Any) -> Any:
        if isinstance(value, ObjectTree):
            return value.to_dict()
        if isinstance(value, dict):
            return {k: ObjectTree._unwrap(v) for k, v in value.items()}
        if isinstance(value, list):
            return [ObjectTree._unwrap(item) for item in value]
        return value

    # --- Attribute access ---

    def __getattr__(self, key: str) -> Any:
        if key.startswith('_'):
            return object.__getattribute__(self, key)
        data = object.__getattribute__(self, '_data')
        if isinstance(data, dict) and key in data:
            return data[key]
        raise AttributeError(f"'{type(self).__name__}' has no attribute '{key}'")

    def __setattr__(self, key: str, value: Any) -> None:
        if key.startswith('_'):
            object.__setattr__(self, key, value)
            return
        data = object.__getattribute__(self, '_data')
        if not isinstance(data, dict):
            raise TypeError("Cannot set attribute on non-mapping ObjectTree")
        wrapped = self._wrap(key, value)
        self._bind_type_check(key, wrapped)
        data[key] = wrapped

    def __delattr__(self, key: str) -> None:
        if key.startswith('_'):
            object.__delattr__(self, key)
            return
        data = object.__getattribute__(self, '_data')
        if not isinstance(data, dict):
            raise TypeError("Cannot delete attribute on non-mapping ObjectTree")
        if key not in data:
            raise AttributeError(f"'{type(self).__name__}' has no attribute '{key}'")
        del data[key]

    # --- Mapping protocol ---

    def __getitem__(self, key: Union[str, int]) -> Any:
        data = object.__getattribute__(self, '_data')
        if isinstance(data, dict):
            return data[key]
        if isinstance(data, list):
            return data[key]
        raise TypeError(f"'{type(self).__name__}' is not subscriptable")

    def __setitem__(self, key: Union[str, int], value: Any) -> None:
        data = object.__getattribute__(self, '_data')
        if isinstance(data, (dict, list)):
            wrapped = self._wrap(key, value)
            self._bind_item_type_check(key, wrapped)
            data[key] = wrapped
            return
        raise TypeError(f"'{type(self).__name__}' does not support item assignment")

    def __delitem__(self, key: Union[str, int]) -> None:
        data = object.__getattribute__(self, '_data')
        if isinstance(data, (dict, list)):
            del data[key]
            return
        raise TypeError(f"'{type(self).__name__}' does not support item deletion")

    def __iter__(self) -> Iterator:
        data = object.__getattribute__(self, '_data')
        if isinstance(data, dict):
            return iter(data)
        raise TypeError("ObjectTree is not iterable as a mapping when wrapping a sequence/scalar")

    def __len__(self) -> int:
        data = object.__getattribute__(self, '_data')
        if isinstance(data, dict):
            return len(data)
        raise TypeError("ObjectTree has no mapping length when wrapping a sequence/scalar")

    def __contains__(self, key) -> bool:
        data = object.__getattribute__(self, '_data')
        if isinstance(data, dict):
            return key in data
        return False

    # --- Convenience ---

    def get(self, key: str, default: Any = None) -> Any:
        data = object.__getattribute__(self, '_data')
        if isinstance(data, dict):
            return data.get(key, default)
        return default

    def pop(self, key: str, *args) -> Any:
        data = object.__getattribute__(self, '_data')
        if not isinstance(data, dict):
            raise TypeError("pop() requires mapping ObjectTree")
        return data.pop(key, *args)

    def clear(self) -> None:
        data = object.__getattribute__(self, '_data')
        if not isinstance(data, dict):
            raise TypeError("clear() requires mapping ObjectTree")
        data.clear()

    def copy(self) -> 'ObjectTree':
        """Shallow copy."""
        return ObjectTree(self.to_dict(), schema=object.__getattribute__(self, '_schema'))

    def setdefault(self, key: str, default: Any = None) -> Any:
        data = object.__getattribute__(self, '_data')
        if not isinstance(data, dict):
            raise TypeError("setdefault() requires mapping ObjectTree")
        if key in data:
            return data[key]
        wrapped = self._wrap(key, default)
        self._bind_type_check(key, wrapped)
        data[key] = wrapped
        return data[key]

    def update(self, other: Union[Mapping, 'ObjectTree'] = None, **kwargs) -> None:
        data = object.__getattribute__(self, '_data')
        if not isinstance(data, dict):
            raise TypeError("Cannot update non-mapping ObjectTree")
        if other is not None:
            if isinstance(other, ObjectTree):
                other = other.to_dict()
            for k, v in dict(other).items():
                wrapped = self._wrap(k, v)
                self._bind_type_check(k, wrapped)
                data[k] = wrapped
        for k, v in kwargs.items():
            wrapped = self._wrap(k, v)
            self._bind_type_check(k, wrapped)
            data[k] = wrapped

    def keys(self) -> KeysView[str]:
        data = object.__getattribute__(self, '_data')
        if isinstance(data, dict):
            return data.keys()
        return {}.keys()

    def values(self) -> ValuesView[Any]:
        data = object.__getattribute__(self, '_data')
        if isinstance(data, dict):
            return data.values()
        return {}.values()

    def items(self) -> ItemsView[str, Any]:
        data = object.__getattribute__(self, '_data')
        if isinstance(data, dict):
            return data.items()
        return {}.items()

    # --- Conversion ---

    def to_dict(self) -> Union[Dict, List, Any]:
        data = object.__getattribute__(self, '_data')
        return self._unwrap(data)

    def to_native(self) -> Union[Dict, List, Any]:
        return self.to_dict()

    # --- Properties ---

    @property
    def is_mapping(self) -> bool:
        data = object.__getattribute__(self, '_data')
        return isinstance(data, dict)

    @property
    def is_sequence(self) -> bool:
        data = object.__getattribute__(self, '_data')
        return isinstance(data, list)

    # --- Constructors ---

    @classmethod
    def from_dict(cls, data: Mapping) -> 'ObjectTree':
        return cls(data)

    fromDict = from_dict

    @classmethod
    def fromkeys(cls, keys, value=None) -> 'ObjectTree':
        return cls({k: value for k in keys})

    # --- Python protocols ---

    def __repr__(self) -> str:
        data = object.__getattribute__(self, '_data')
        return f"ObjectTree({self._unwrap(data)!r})"

    def __str__(self) -> str:
        return str(self.to_dict())

    def __eq__(self, other) -> bool:
        if isinstance(other, ObjectTree):
            return self.to_dict() == other.to_dict()
        if isinstance(other, Mapping):
            return self.to_dict() == dict(other)
        return NotImplemented

    def __bool__(self) -> bool:
        data = object.__getattribute__(self, '_data')
        if isinstance(data, (dict, list)):
            return bool(data)
        return data is not None

    def __copy__(self) -> 'ObjectTree':
        return self.copy()

    def __deepcopy__(self, memo) -> 'ObjectTree':
        schema = object.__getattribute__(self, '_schema')
        return ObjectTree(
            copy.deepcopy(self.to_dict(), memo),
            schema=copy.deepcopy(schema, memo) if schema else None,
        )

    def __getstate__(self) -> dict:
        schema = object.__getattribute__(self, '_schema')
        return {
            'data': self.to_dict(),
            'schema': schema.to_dict() if isinstance(schema, ObjectTree) else schema,
        }

    def __setstate__(self, state: dict) -> None:
        self.__init__(state['data'], schema=state.get('schema'))

    def __or__(self, other) -> 'ObjectTree':
        if isinstance(other, ObjectTree):
            other = other.to_dict()
        if not isinstance(other, Mapping):
            return NotImplemented
        merged = {**self.to_dict(), **dict(other)}
        return ObjectTree(merged, schema=object.__getattribute__(self, '_schema'))

    def __ior__(self, other) -> 'ObjectTree':
        if isinstance(other, ObjectTree):
            other = other.to_dict()
        if not isinstance(other, Mapping):
            return NotImplemented
        self.update(other)
        return self

    # --- Type binding ---

    def _bind_type_check(self, key: str, value: Any) -> None:
        schema = object.__getattribute__(self, '_schema')
        if not isinstance(schema, ObjectTree):
            return
        props = schema.get('properties')
        if isinstance(props, Mapping) and key in props:
            self._validate_field(props[key], value, field=key)

    def _bind_item_type_check(self, key: Union[str, int], value: Any) -> None:
        schema = object.__getattribute__(self, '_schema')
        if not isinstance(schema, ObjectTree):
            return
        data = object.__getattribute__(self, '_data')
        if isinstance(data, dict) and isinstance(key, str):
            props = schema.get('properties')
            if isinstance(props, Mapping) and key in props:
                self._validate_field(props[key], value, field=key)
        elif isinstance(data, list):
            items = schema.get('items')
            if items:
                self._validate_field(items, value, field=f'[{key}]')


class ObjectTreeEncoder(json.JSONEncoder):
    """JSON encoder supporting ObjectTree."""

    def default(self, obj: Any) -> Any:
        if isinstance(obj, ObjectTree):
            return obj.to_dict()
        return super().default(obj)
