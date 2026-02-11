"""SchemaAPI — Draft-07 keywords as object methods.

Schema is ObjectTree. Keywords are attributes. Logic is methods.

  schema.oneOf           -> attribute -> raw list
  data.one_of()          -> method    -> correct branch (XOR)
  data.any_of()          -> method    -> matching branches (OR)
  data.all_of()          -> method    -> merged schema (AND)
  data.not_of()          -> method    -> exclusion check (EXCEPT)
  data.if_then()         -> method    -> conditional branch (CASE WHEN)
  data.project()         -> method    -> schema-defined fields (SELECT)
  data.contains()        -> method    -> array element check (EXISTS)
"""
from __future__ import annotations

import math
import re
from collections.abc import Mapping, Sequence
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from .tree import ObjectTree

# --- Helpers ---

def _raw(v: Any) -> Any:
    """Unwrap ObjectTree to plain value."""
    return v.to_dict() if hasattr(v, 'to_dict') else v


# --- Validation keywords: schema attribute -> value check ---

_TYPE_MAP = {
    'string': str,
    'integer': int,
    'number': (int, float),
    'boolean': bool,
    'array': list,
    'object': dict,
    'null': type(None),
}


def _check_value(schema: Mapping, value: Any) -> tuple[bool, str]:
    """Check value against Draft-07 schema constraints.

    Returns (ok, reason). reason is empty on success.
    """
    if not isinstance(schema, Mapping):
        return True, ''

    # --- type (supports string or array of strings) ---
    st = schema.get('type')
    if st is not None:
        types = st if isinstance(st, (list, tuple)) else [st]
        numeric_types = {'integer', 'number'}
        if isinstance(value, bool) and 'boolean' not in types and (numeric_types & set(types)):
            return False, f"expected type {st!r}, got bool"
        allowed = []
        for s in types:
            mapped = _TYPE_MAP.get(s)
            if mapped is not None:
                if isinstance(mapped, tuple):
                    allowed.extend(mapped)
                else:
                    allowed.append(mapped)
        if allowed and not isinstance(value, tuple(allowed)):
            return False, f"expected type {st!r}, got {type(value).__name__}"

    # --- const ---
    if 'const' in schema:
        c = _raw(schema['const'])
        if value != c:
            return False, f"expected {c!r}, got {value!r}"

    # --- enum ---
    en = schema.get('enum')
    if isinstance(en, (list, tuple)):
        raw_en = [_raw(e) for e in en]
        if value not in raw_en:
            return False, f"{value!r} not in {raw_en!r}"

    # --- numeric: minimum / maximum / exclusive / multipleOf ---
    if isinstance(value, (int, float)) and not isinstance(value, bool):
        mn = schema.get('minimum')
        if mn is not None and value < _raw(mn):
            return False, f"{value} < minimum {_raw(mn)}"

        mx = schema.get('maximum')
        if mx is not None and value > _raw(mx):
            return False, f"{value} > maximum {_raw(mx)}"

        emn = schema.get('exclusiveMinimum')
        if emn is not None and value <= _raw(emn):
            return False, f"{value} <= exclusiveMinimum {_raw(emn)}"

        emx = schema.get('exclusiveMaximum')
        if emx is not None and value >= _raw(emx):
            return False, f"{value} >= exclusiveMaximum {_raw(emx)}"

        mo = schema.get('multipleOf')
        if mo is not None:
            raw_mo = _raw(mo)
            if raw_mo != 0:
                remainder = math.remainder(value, raw_mo)
                if not math.isclose(remainder, 0, abs_tol=1e-9):
                    return False, f"{value} is not a multiple of {raw_mo}"

    # --- string: minLength / maxLength / pattern ---
    if isinstance(value, str):
        mnl = schema.get('minLength')
        if mnl is not None and len(value) < _raw(mnl):
            return False, f"length {len(value)} < minLength {_raw(mnl)}"

        mxl = schema.get('maxLength')
        if mxl is not None and len(value) > _raw(mxl):
            return False, f"length {len(value)} > maxLength {_raw(mxl)}"

        pat = schema.get('pattern')
        if pat is not None:
            raw_pat = _raw(pat)
            if not re.search(raw_pat, value):
                return False, f"{value!r} does not match pattern {raw_pat!r}"

    # --- array: minItems / maxItems / uniqueItems ---
    if isinstance(value, list):
        mni = schema.get('minItems')
        if mni is not None and len(value) < _raw(mni):
            return False, f"array length {len(value)} < minItems {_raw(mni)}"

        mxi = schema.get('maxItems')
        if mxi is not None and len(value) > _raw(mxi):
            return False, f"array length {len(value)} > maxItems {_raw(mxi)}"

        if schema.get('uniqueItems'):
            seen = []
            for item in value:
                if item in seen:
                    return False, f"duplicate item {item!r} in array"
                seen.append(item)

    # --- object: required / properties / additionalProperties / min/maxProperties ---
    if isinstance(value, Mapping):
        req = schema.get('required')
        if isinstance(req, (list, tuple)):
            for r in req:
                rk = _raw(r)
                if rk not in value:
                    return False, f"missing required field '{rk}'"

        mnp = schema.get('minProperties')
        if mnp is not None and len(value) < _raw(mnp):
            return False, f"object has {len(value)} properties, minProperties is {_raw(mnp)}"

        mxp = schema.get('maxProperties')
        if mxp is not None and len(value) > _raw(mxp):
            return False, f"object has {len(value)} properties, maxProperties is {_raw(mxp)}"

        props = schema.get('properties')
        if isinstance(props, Mapping):
            for pk, ps in props.items():
                if pk in value and isinstance(ps, Mapping):
                    ok, reason = _check_value(ps, value[pk])
                    if not ok:
                        return False, f".{pk}: {reason}"

        # patternProperties: regex keys -> sub-schema
        pp = schema.get('patternProperties')
        if isinstance(pp, Mapping):
            for pat_key, pat_schema in pp.items():
                raw_pat = _raw(pat_key)
                if not isinstance(pat_schema, Mapping):
                    continue
                for vk, vv in value.items():
                    if re.search(raw_pat, str(vk)):
                        ok, reason = _check_value(pat_schema, vv)
                        if not ok:
                            return False, f".{vk} (pattern '{raw_pat}'): {reason}"

        # additionalProperties: False or schema
        ap = schema.get('additionalProperties')
        if ap is not None and ap is not True:
            defined = set(props.keys()) if isinstance(props, Mapping) else set()
            pp_patterns = [_raw(k) for k in pp.keys()] if isinstance(pp, Mapping) else []
            extra = set()
            for vk in value.keys():
                if vk in defined:
                    continue
                if any(re.search(pat, str(vk)) for pat in pp_patterns):
                    continue
                extra.add(vk)
            if extra:
                if ap is False:
                    return False, f"additional properties not allowed: {extra}"
                if isinstance(ap, Mapping):
                    for ek in extra:
                        ok, reason = _check_value(ap, value[ek])
                        if not ok:
                            return False, f".{ek} (additionalProperties): {reason}"

        # dependencies (Draft-07): array form = required keys, schema form = sub-schema
        # dependentRequired (2019-09): array form only (also supported)
        for dep_kw in ('dependencies', 'dependentRequired'):
            dr = schema.get(dep_kw)
            if not isinstance(dr, Mapping):
                continue
            for dk, deps in dr.items():
                raw_dk = _raw(dk)
                if raw_dk not in value:
                    continue
                if isinstance(deps, (list, tuple)):
                    for dep in deps:
                        raw_dep = _raw(dep)
                        if raw_dep not in value:
                            return False, f"'{raw_dk}' requires '{raw_dep}' to be present"
                elif isinstance(deps, Mapping):
                    ok, reason = _check_value(deps, value)
                    if not ok:
                        return False, f"dependency '{raw_dk}': {reason}"

    return True, ''


def _check_match(schema: Mapping, data: Any) -> bool:
    """Check if data matches a sub-schema. Unwraps ObjectTree values."""
    from .tree import ObjectTree
    unwrapped = data.to_dict() if isinstance(data, ObjectTree) else data
    ok, _ = _check_value(schema, unwrapped)
    return ok


class SchemaAPI:
    """Draft-07 logic keywords as object methods (mixin for ObjectTree)."""

    # --- Branch matching ---

    def _match_branches(self, subs: list | tuple) -> list[int]:
        """Return indices of sub-schemas that match current data."""
        data = object.__getattribute__(self, '_data')
        matches = []
        for i, sub in enumerate(subs):
            if not isinstance(sub, Mapping):
                continue
            if _check_match(sub, data):
                matches.append(i)
        return matches

    # --- Applicator methods ---

    def one_of(self) -> 'ObjectTree':
        """oneOf (XOR): select unique matching branch.

        Raises TypeError if zero or multiple branches match.
        """
        from .tree import ObjectTree
        schema = object.__getattribute__(self, '_schema')
        if not isinstance(schema, ObjectTree):
            return self
        subs = schema.get('oneOf')
        if not isinstance(subs, list) or not subs:
            return self
        matches = self._match_branches(subs)
        if len(matches) != 1:
            raise TypeError(f"oneOf: expected 1 match, got {len(matches)}")
        return ObjectTree(self.to_dict(), schema=subs[matches[0]])

    def any_of(self) -> list['ObjectTree']:
        """anyOf (OR): return all matching branches.

        Raises TypeError if no branches match.
        """
        from .tree import ObjectTree
        schema = object.__getattribute__(self, '_schema')
        if not isinstance(schema, ObjectTree):
            return [self]
        subs = schema.get('anyOf')
        if not isinstance(subs, list) or not subs:
            return [self]
        matches = self._match_branches(subs)
        if not matches:
            raise TypeError("anyOf: no matching branch")
        return [ObjectTree(self.to_dict(), schema=subs[i]) for i in matches]

    def all_of(self) -> 'ObjectTree':
        """allOf (AND): merge all sub-schemas into one.

        Merges properties, required, and items from top-level and allOf sub-schemas.
        Later sub-schemas override earlier ones on key conflict.
        """
        from .tree import ObjectTree
        schema = object.__getattribute__(self, '_schema')
        if not isinstance(schema, ObjectTree):
            return self
        subs = schema.get('allOf')
        if not isinstance(subs, list) or not subs:
            return self

        merged = {}
        sources = [schema] + [s for s in subs if isinstance(s, Mapping)]
        for src in sources:
            sp = src.get('properties') if isinstance(src, Mapping) else None
            if isinstance(sp, Mapping):
                merged.setdefault('properties', {})
                for k, v in sp.items():
                    merged['properties'][k] = _raw(v)
            sr = src.get('required') if isinstance(src, Mapping) else None
            if isinstance(sr, (list, tuple)):
                merged.setdefault('required', [])
                for r in sr:
                    rv = _raw(r)
                    if rv not in merged['required']:
                        merged['required'].append(rv)
            si = src.get('items') if isinstance(src, Mapping) else None
            if isinstance(si, Mapping):
                merged['items'] = _raw(si)

        return ObjectTree(self.to_dict(), schema=merged)

    def not_of(self, schema: Mapping | None = None) -> bool:
        """not (EXCEPT): True if data does NOT match schema.

        Uses schema.not if no explicit schema provided.
        """
        own_schema = object.__getattribute__(self, '_schema')
        target = schema
        if target is None and isinstance(own_schema, Mapping):
            target = own_schema.get('not')
        if not isinstance(target, Mapping):
            return True
        data = object.__getattribute__(self, '_data')
        return not _check_match(target, data)

    def if_then(self) -> 'ObjectTree':
        """if/then/else (CASE WHEN): conditional branch.

        Evaluates if-schema against data, returns then/else branch accordingly.
        Returns self if no if-schema or no matching branch exists.
        """
        from .tree import ObjectTree
        schema = object.__getattribute__(self, '_schema')
        if not isinstance(schema, ObjectTree):
            return self
        if_schema = schema.get('if')
        if not isinstance(if_schema, Mapping):
            return self
        data = object.__getattribute__(self, '_data')
        if _check_match(if_schema, data):
            branch = schema.get('then')
        else:
            branch = schema.get('else')
        if isinstance(branch, Mapping):
            return ObjectTree(self.to_dict(), schema=branch)
        return self

    def project(self) -> 'ObjectTree':
        """properties (SELECT): keep only schema-defined fields.

        For oneOf/anyOf with exactly 1 match, auto-resolves.
        Raises TypeError on multiple matches — call one_of()/any_of() first.
        """
        from .tree import ObjectTree
        schema = object.__getattribute__(self, '_schema')
        if not isinstance(schema, ObjectTree):
            return self
        resolved = schema
        for keyword in ('oneOf', 'anyOf'):
            subs = schema.get(keyword)
            if isinstance(subs, list) and subs:
                matches = self._match_branches(subs)
                if len(matches) == 1:
                    resolved = subs[matches[0]]
                elif len(matches) > 1:
                    raise TypeError(
                        f"project: {keyword} has {len(matches)} matches, "
                        f"use {keyword.replace('Of', '_of')}() first to disambiguate"
                    )
                break
        props = resolved.get('properties')
        if not isinstance(props, Mapping):
            return self
        data = self.to_dict()
        if not isinstance(data, Mapping):
            return self
        filtered = {k: data[k] for k in props if k in data}
        return ObjectTree(filtered, schema=resolved)

    def contains(self, schema: Mapping | None = None) -> bool:
        """contains (EXISTS): True if any array element matches schema.

        Uses schema.contains if no explicit schema provided.
        """
        own_schema = object.__getattribute__(self, '_schema')
        target = schema
        if target is None and isinstance(own_schema, Mapping):
            target = own_schema.get('contains')
        if not isinstance(target, Mapping):
            return False
        data = object.__getattribute__(self, '_data')
        if not isinstance(data, list):
            return False
        for item in data:
            if _check_match(target, item):
                return True
        return False

    # --- Field validation (for type binding) ---

    @staticmethod
    def _validate_field(
        field_schema: Mapping, value: Any, *, field: str = ''
    ) -> None:
        """Validate a value against its schema. Raises TypeError on failure.

        Args:
            field_schema: The JSON Schema for this field.
            value: The value to validate (ObjectTree values are auto-unwrapped).
            field: Optional field name for error context.
        """
        if not isinstance(field_schema, Mapping):
            return
        from .tree import ObjectTree
        unwrapped = value.to_dict() if isinstance(value, ObjectTree) else value
        ok, reason = _check_value(field_schema, unwrapped)
        if not ok:
            prefix = f"'{field}': " if field else ''
            raise TypeError(f"{prefix}{reason}")
