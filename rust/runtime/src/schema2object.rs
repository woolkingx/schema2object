/**
 * schema2object — Rust
 * JSON Schema IS the object class.
 * Spec: docs/draft-07-spec.json
 *
 * Structure mirrors schema2object.mjs — single file, same section order.
 */

use regex::Regex;
use serde_json::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use unicode_segmentation::UnicodeSegmentation;

// ─── Shared ───────────────────────────────────────────────────────────────────

fn unicode_len(s: &str) -> usize {
    s.graphemes(true).count()
}

fn unescape_pointer(s: &str) -> String {
    s.replace("~1", "/").replace("~0", "~")
}

fn cached_regex(pattern: &str) -> Result<Regex, regex::Error> {
    thread_local! {
        static CACHE: RefCell<HashMap<String, Regex>> = RefCell::new(HashMap::new());
    }
    CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if let Some(re) = cache.get(pattern) { return Ok(re.clone()); }
        let re = Regex::new(pattern)?;
        cache.insert(pattern.to_string(), re.clone());
        Ok(re)
    })
}

// ─── Deep equality ────────────────────────────────────────────────────────────
// JSON Schema: 0 == 0.0, 1 == 1.0 etc.

fn deep_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(na), Value::Number(nb)) => match (na.as_f64(), nb.as_f64()) {
            (Some(fa), Some(fb)) => fa == fb,
            _ => na == nb,
        },
        (Value::Array(aa), Value::Array(ab)) => {
            aa.len() == ab.len() && aa.iter().zip(ab.iter()).all(|(x, y)| deep_equal(x, y))
        }
        (Value::Object(oa), Value::Object(ob)) => {
            oa.len() == ob.len()
                && oa.iter().all(|(k, v)| ob.get(k).map(|w| deep_equal(v, w)).unwrap_or(false))
        }
        _ => a == b,
    }
}

// ─── Deep schema merge ────────────────────────────────────────────────────────

fn deep_merge(a: Value, b: &Value) -> Value {
    match (a, b) {
        (Value::Object(mut ao), Value::Object(bo)) => {
            for (k, bv) in bo {
                match k.as_str() {
                    "properties" => {
                        let merged = match (ao.remove(k), bv) {
                            (Some(Value::Object(mut ap)), Value::Object(bp)) => {
                                for (pk, pv) in bp {
                                    let next = if let Some(existing) = ap.remove(pk) {
                                        deep_merge(existing, pv)
                                    } else {
                                        pv.clone()
                                    };
                                    ap.insert(pk.clone(), next);
                                }
                                Value::Object(ap)
                            }
                            (_, v) => v.clone(),
                        };
                        ao.insert(k.clone(), merged);
                    }
                    "required" => {
                        let mut seen = std::collections::HashSet::new();
                        let mut order: Vec<serde_json::Value> = vec![];
                        if let Some(Value::Array(ar)) = ao.remove(k) {
                            for v in ar { if let Some(s) = v.as_str() { if seen.insert(s.to_string()) { order.push(Value::String(s.to_string())); } } }
                        }
                        if let Value::Array(br) = bv {
                            for v in br { if let Some(s) = v.as_str() { if seen.insert(s.to_string()) { order.push(Value::String(s.to_string())); } } }
                        }
                        ao.insert(k.clone(), Value::Array(order));
                    }
                    _ => {
                        let merged = match ao.remove(k) {
                            Some(existing) => deep_merge(existing, bv),
                            None => bv.clone(),
                        };
                        ao.insert(k.clone(), merged);
                    }
                }
            }
            Value::Object(ao)
        }
        (_, v) => v.clone(),
    }
}

// ─── URI helpers ──────────────────────────────────────────────────────────────

fn resolve_uri(reference: &str, base: Option<&str>) -> String {
    if reference.starts_with('#') { return reference.to_string(); }
    if reference.contains("://") { return reference.to_string(); }
    // URN or other scheme without // (e.g. urn:uuid:...)
    if reference.contains(':') && !reference.starts_with('.') && !reference.starts_with('/') {
        return reference.to_string();
    }
    // absolute path — attach scheme+host from base
    if reference.starts_with('/') {
        if let Some(base) = base {
            let base_no_frag = base.split('#').next().unwrap_or(base);
            if let Some(after_scheme) = base_no_frag.find("://") {
                let scheme_host_end = base_no_frag[after_scheme + 3..]
                    .find('/').map(|i| after_scheme + 3 + i)
                    .unwrap_or(base_no_frag.len());
                let scheme_host = &base_no_frag[..scheme_host_end];
                return format!("{scheme_host}{reference}");
            }
        }
        return reference.to_string();
    }
    let Some(base) = base else { return reference.to_string(); };
    let base_no_frag = base.split('#').next().unwrap_or(base);
    let last_slash = base_no_frag.rfind('/').map(|i| i + 1).unwrap_or(base_no_frag.len());
    let dir = &base_no_frag[..last_slash];
    format!("{dir}{reference}")
}

fn split_fragment(uri: &str) -> (&str, Option<&str>) {
    if let Some(idx) = uri.find('#') {
        let frag = &uri[idx + 1..];
        (&uri[..idx], if frag.is_empty() { None } else { Some(frag) })
    } else {
        (uri, None)
    }
}

fn json_pointer<'a>(root: &'a Value, pointer: &str) -> Option<&'a Value> {
    let mut node = root;
    for part in pointer.trim_start_matches('/').split('/') {
        let key = unescape_pointer(&percent_decode(part));
        node = match node {
            Value::Object(map) => map.get(&key)?,
            Value::Array(arr) => { let idx: usize = key.parse().ok()?; arr.get(idx)? }
            _ => return None,
        };
    }
    Some(node)
}

fn percent_decode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(hex) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                out.push(hex as char); i += 3; continue;
            }
        }
        out.push(bytes[i] as char); i += 1;
    }
    out
}

// ─── Loader ───────────────────────────────────────────────────────────────────
// Resolves $ref per JSON Schema Draft-07 URI semantics.
// Mirrors JS Loader class — same fields: root, id_map, scope_map, resolver, remote_cache.

#[derive(Clone)]
struct ScopeInfo {
    /// Base URI BEFORE this node's own $id — used when $ref present (sibling $id ignored).
    parent_scope: Option<String>,
    /// Nearest $id-bearing ancestor URI — '#' resolves within this resource.
    resource: Option<String>,
}

pub struct Loader {
    root: Box<Value>,                                // heap-pinned: address stable after move
    id_map: HashMap<String, *const Value>,           // URI → node ptr
    scope_map: HashMap<*const Value, ScopeInfo>,
    remote_cache: RefCell<HashMap<String, Box<Loader>>>,
    resolver: Option<String>,                        // directory path (string resolver)
}

// SAFETY: raw ptrs used as opaque HashMap keys only, never dereferenced across threads.
unsafe impl Send for Loader {}
unsafe impl Sync for Loader {}

impl Loader {
    pub fn new(root: Value, resolver: Option<String>) -> Self {
        Self::new_with_base(root, resolver, None)
    }

    pub fn new_with_base(root: Value, resolver: Option<String>, document_base: Option<String>) -> Self {
        let root = Box::new(root);
        let mut loader = Self {
            root,
            id_map: HashMap::new(),
            scope_map: HashMap::new(),
            remote_cache: RefCell::new(HashMap::new()),
            resolver,
        };
        let base = document_base.or_else(|| {
            loader.root.get("$id").and_then(|v| v.as_str())
                .map(|s| s.trim_end_matches('#').to_string())
        });
        let root_ptr = loader.root.as_ref() as *const Value;
        loader.scan_ids(root_ptr, base.clone(), base.clone());
        loader
    }

    fn scan_ids(&mut self, node_ptr: *const Value, base_uri: Option<String>, resource: Option<String>) {
        // SAFETY: node_ptr points into self.root which is owned and stable.
        let node = unsafe { &*node_ptr };
        if node.is_boolean() || node.is_null() || node.is_number() || node.is_string() { return; }
        if let Some(arr) = node.as_array() {
            for item in arr { self.scan_ids(item as *const Value, base_uri.clone(), resource.clone()); }
            return;
        }
        let obj = match node.as_object() { Some(o) => o, None => return };

        let mut current_base = base_uri.clone();
        let mut current_resource = resource.clone();

        if let Some(id_str) = obj.get("$id").and_then(|v| v.as_str()) {
            let resolved = resolve_uri(id_str, base_uri.as_deref());
            let normalized = resolved.trim_end_matches('#').to_string();
            self.id_map.insert(normalized.clone(), node_ptr);
            if normalized != resolved { self.id_map.insert(resolved.clone(), node_ptr); }
            if id_str.starts_with('#') { self.id_map.insert(id_str.to_string(), node_ptr); }
            current_base = Some(normalized.clone());
            current_resource = Some(normalized);
        }

        self.scope_map.insert(node_ptr, ScopeInfo {
            parent_scope: base_uri,
            resource: current_resource.clone(),
        });

        for (_k, v) in obj {
            self.scan_ids(v as *const Value, current_base.clone(), current_resource.clone());
        }
    }

    fn parent_scope_of(&self, node: &Value) -> Option<&str> {
        self.scope_map.get(&(node as *const Value))?.parent_scope.as_deref()
    }

    fn resource_of(&self, node: &Value) -> Option<&str> {
        self.scope_map.get(&(node as *const Value))?.resource.as_deref()
    }

    fn resolve<'a>(&'a self, reference: &str, base_uri: Option<&str>, resource_uri: Option<&str>)
        -> Result<(&'a Value, &'a Loader), ValidationError>
    {
        // '#' → root of nearest $id-bearing resource
        if reference == "#" {
            let node = resource_uri.and_then(|u| self.id_map.get(u))
                .map(|p| unsafe { &**p }).unwrap_or(&self.root);
            return Ok((node, self));
        }

        // '#/path' → JSON Pointer within nearest resource
        if reference.starts_with("#/") {
            let res_node = resource_uri.and_then(|u| self.id_map.get(u))
                .map(|p| unsafe { &**p }).unwrap_or(&self.root);
            let node = json_pointer(res_node, &reference[1..])
                .ok_or_else(|| ValidationError::new("$", ErrorKind::TypeError,
                    format!("$ref not found: {reference}")))?;
            return Ok((node, self));
        }

        // '#anchor' — named anchor
        if reference.starts_with('#') {
            if let Some(ptr) = self.id_map.get(reference) {
                return Ok((unsafe { &**ptr }, self));
            }
            return Err(ValidationError::new("$", ErrorKind::TypeError,
                format!("$ref anchor not found: {reference}")));
        }

        // Resolve relative ref against base URI
        let resolved = resolve_uri(reference, base_uri);
        let (doc_uri, fragment) = split_fragment(&resolved);

        // Check $id map
        if let Some(ptr) = self.id_map.get(doc_uri).or_else(|| self.id_map.get(resolved.as_str())) {
            let doc_node = unsafe { &**ptr };
            if fragment.is_none() { return Ok((doc_node, self)); }
            let frag = fragment.unwrap();
            if frag.starts_with('/') {
                let node = json_pointer(doc_node, frag)
                    .ok_or_else(|| ValidationError::new("$", ErrorKind::TypeError,
                        format!("$ref not found: {reference}")))?;
                return Ok((node, self));
            }
            let anchor_ref = format!("#{frag}");
            if let Some(ptr2) = self.id_map.get(&anchor_ref) {
                return Ok((unsafe { &**ptr2 }, self));
            }
        }

        // External: http(s) or any ref with resolver
        if resolved.starts_with("http://") || resolved.starts_with("https://") || self.resolver.is_some() {
            return self.resolve_remote(&resolved, fragment, reference);
        }

        Err(ValidationError::new("$", ErrorKind::TypeError, format!("Unsupported $ref: {reference}")))
    }

    fn resolve_remote<'a>(&'a self, resolved: &str, fragment: Option<&str>, _original: &str)
        -> Result<(&'a Value, &'a Loader), ValidationError>
    {
        let (doc_uri, _) = split_fragment(resolved);
        let doc_uri = doc_uri.to_string();

        if !self.remote_cache.borrow().contains_key(&doc_uri) {
            let schema = self.load_schema(&doc_uri)?;
            let resolver = self.resolver.clone();
            let sub = Box::new(Loader::new_with_base(schema, resolver, Some(doc_uri.clone())));
            self.remote_cache.borrow_mut().insert(doc_uri.clone(), sub);
        }

        // SAFETY: Box stable on heap after insert.
        let sub_loader = unsafe {
            let ptr: *const Loader = self.remote_cache.borrow().get(&doc_uri).unwrap().as_ref();
            &*ptr
        };

        match fragment {
            None => Ok((&sub_loader.root, sub_loader)),
            Some(frag) => sub_loader.resolve(&format!("#{frag}"), None, None),
        }
    }

    fn load_schema(&self, doc_uri: &str) -> Result<Value, ValidationError> {
        let Some(dir) = &self.resolver else {
            return Err(ValidationError::new("$", ErrorKind::TypeError,
                format!("$ref remote not supported (no resolver): {doc_uri}")));
        };
        // Strip scheme+host from any http(s) URI
        let path_part = if let Some(rest) = doc_uri.strip_prefix("http://") {
            rest.splitn(2, '/').nth(1).unwrap_or(rest)
        } else if let Some(rest) = doc_uri.strip_prefix("https://") {
            rest.splitn(2, '/').nth(1).unwrap_or(rest)
        } else { doc_uri };

        let full_path = format!("{dir}/{path_part}");
        let content = std::fs::read_to_string(&full_path)
            .or_else(|_| std::fs::read_to_string(format!("{full_path}.json")))
            .map_err(|_| ValidationError::new("$", ErrorKind::TypeError,
                format!("$ref file not found: {full_path}")))?;
        serde_json::from_str(&content).map_err(|e| ValidationError::new("$", ErrorKind::TypeError,
            format!("$ref JSON parse error: {e}")))
    }

    pub fn root(&self) -> &Value { &self.root }
    pub fn resolver(&self) -> Option<String> { self.resolver.clone() }
}

// ─── Soft validate ────────────────────────────────────────────────────────────

fn soft_validate(value: &Value, schema: &Value, loader: &Loader) -> bool {
    validate_node(value, schema, "$", loader).is_ok()
}

// ─── Core validator ───────────────────────────────────────────────────────────

fn validate_node(value: &Value, schema: &Value, path: &str, loader: &Loader) -> Result<(), ValidationError> {
    // Boolean schema
    if let Some(b) = schema.as_bool() {
        return if b { Ok(()) } else {
            Err(ValidationError::new(path, ErrorKind::TypeError, "schema is false"))
        };
    }

    // $ref → resolve and delegate; sibling keywords ignored (Draft 4-7)
    if let Some(ref_str) = schema.get("$ref").and_then(|v| v.as_str()) {
        let base_uri = loader.parent_scope_of(schema);
        let resource_uri = loader.resource_of(schema);
        let (ref_node, ref_loader) = loader.resolve(ref_str, base_uri, resource_uri)
            .map_err(|e| ValidationError::new(path, e.kind, e.message))?;
        return validate_node(value, ref_node, path, ref_loader);
    }

    let obj = match schema.as_object() {
        Some(o) => o,
        None => return Ok(()),
    };

    // --- type ---
    if let Some(type_spec) = obj.get("type") { check_type(type_spec, value, path)?; }

    // --- const ---
    if let Some(c) = obj.get("const") {
        if !deep_equal(value, c) {
            return Err(ValidationError::new(path, ErrorKind::ConstMismatch,
                format!("expected {c}, got {value}")));
        }
    }

    // --- enum ---
    if let Some(Value::Array(variants)) = obj.get("enum") {
        if !variants.iter().any(|e| deep_equal(e, value)) {
            return Err(ValidationError::new(path, ErrorKind::EnumMismatch,
                format!("{value} not in enum")));
        }
    }

    // --- numeric ---
    if !value.is_boolean() {
        if let Some(n) = value.as_f64() { check_numeric(obj, n, path)?; }
    }

    // --- string ---
    if let Some(s) = value.as_str() { check_string(obj, s, path)?; }

    // --- array ---
    if let Some(arr) = value.as_array() { check_array(obj, arr, path, loader)?; }

    // --- object ---
    if let Some(map) = value.as_object() { check_object(obj, map, value, path, loader)?; }

    // --- not ---
    if let Some(not_schema) = obj.get("not") {
        if soft_validate(value, not_schema, loader) {
            return Err(ValidationError::new(path, ErrorKind::TypeError, "value matches 'not' schema"));
        }
    }

    // --- allOf ---
    if let Some(Value::Array(subs)) = obj.get("allOf") {
        for (i, sub) in subs.iter().enumerate() {
            validate_node(value, sub, path, loader).map_err(|e| {
                ValidationError::new(path, e.kind, format!("allOf[{i}]: {}", e.message))
            })?;
        }
    }

    // --- anyOf ---
    if let Some(Value::Array(subs)) = obj.get("anyOf") {
        if !subs.iter().any(|s| soft_validate(value, s, loader)) {
            return Err(ValidationError::new(path, ErrorKind::AnyOfNoneMatch, "anyOf: no branch matches"));
        }
    }

    // --- oneOf ---
    if let Some(Value::Array(subs)) = obj.get("oneOf") {
        let n = subs.iter().filter(|s| soft_validate(value, s, loader)).count();
        if n == 0 { return Err(ValidationError::new(path, ErrorKind::OneOfNoneMatch, "oneOf: no branch matches")); }
        if n > 1 { return Err(ValidationError::new(path, ErrorKind::OneOfMultipleMatch,
            format!("oneOf: expected 1 match, got {n}"))); }
    }

    // --- if/then/else ---
    if let Some(if_schema) = obj.get("if") {
        if soft_validate(value, if_schema, loader) {
            if let Some(then_schema) = obj.get("then") {
                validate_node(value, then_schema, path, loader).map_err(|e| {
                    ValidationError::new(path, ErrorKind::IfThenFailed, format!("if/then: {}", e.message))
                })?;
            }
        } else if let Some(else_schema) = obj.get("else") {
            validate_node(value, else_schema, path, loader).map_err(|e| {
                ValidationError::new(path, ErrorKind::IfThenFailed, format!("if/else: {}", e.message))
            })?;
        }
    }

    Ok(())
}

// ─── Type checking ────────────────────────────────────────────────────────────

fn check_type(type_spec: &Value, value: &Value, path: &str) -> Result<(), ValidationError> {
    let types: Vec<&str> = match type_spec {
        Value::String(s) => vec![s.as_str()],
        Value::Array(arr) => arr.iter().filter_map(|v| v.as_str()).collect(),
        _ => return Ok(()),
    };
    // Draft-07: bool is NOT integer/number
    if value.is_boolean() && !types.contains(&"boolean") {
        if types.contains(&"integer") || types.contains(&"number") {
            return Err(ValidationError::new(path, ErrorKind::TypeError,
                format!("expected type {type_spec}, got boolean")));
        }
    }
    let matched = types.iter().any(|t| match *t {
        "null"    => value.is_null(),
        "boolean" => value.is_boolean(),
        "integer" => !value.is_boolean() && (value.is_i64() || value.is_u64() || is_whole_f64(value)),
        "number"  => !value.is_boolean() && value.is_number(),
        "string"  => value.is_string(),
        "array"   => value.is_array(),
        "object"  => value.is_object(),
        _ => false,
    });
    if !matched {
        return Err(ValidationError::new(path, ErrorKind::TypeError,
            format!("expected type {type_spec}, got {}", value_type_name(value))));
    }
    Ok(())
}

// ─── Numeric ──────────────────────────────────────────────────────────────────

fn check_numeric(schema: &serde_json::Map<String, Value>, n: f64, path: &str) -> Result<(), ValidationError> {
    if let Some(min) = schema.get("minimum").and_then(|v| v.as_f64()) {
        if n < min { return Err(ValidationError::new(path, ErrorKind::Minimum, format!("{n} < minimum {min}"))); }
    }
    if let Some(max) = schema.get("maximum").and_then(|v| v.as_f64()) {
        if n > max { return Err(ValidationError::new(path, ErrorKind::Maximum, format!("{n} > maximum {max}"))); }
    }
    if let Some(emin) = schema.get("exclusiveMinimum").and_then(|v| v.as_f64()) {
        if n <= emin { return Err(ValidationError::new(path, ErrorKind::ExclusiveMinimum,
            format!("{n} <= exclusiveMinimum {emin}"))); }
    }
    if let Some(emax) = schema.get("exclusiveMaximum").and_then(|v| v.as_f64()) {
        if n >= emax { return Err(ValidationError::new(path, ErrorKind::ExclusiveMaximum,
            format!("{n} >= exclusiveMaximum {emax}"))); }
    }
    if let Some(mo) = schema.get("multipleOf").and_then(|v| v.as_f64()) {
        if mo != 0.0 {
            let q = n / mo;
            if !q.is_finite() || (q - q.round()).abs() > 1e-9 * f64::max(1.0, q.abs()) {
                return Err(ValidationError::new(path, ErrorKind::MultipleOf,
                    format!("{n} is not a multiple of {mo}")));
            }
        }
    }
    Ok(())
}

// ─── String ───────────────────────────────────────────────────────────────────

fn check_string(schema: &serde_json::Map<String, Value>, s: &str, path: &str) -> Result<(), ValidationError> {
    let len = unicode_len(s);
    if let Some(min) = schema.get("minLength").and_then(as_usize) {
        if len < min { return Err(ValidationError::new(path, ErrorKind::MinLength,
            format!("length {len} < minLength {min}"))); }
    }
    if let Some(max) = schema.get("maxLength").and_then(as_usize) {
        if len > max { return Err(ValidationError::new(path, ErrorKind::MaxLength,
            format!("length {len} > maxLength {max}"))); }
    }
    if let Some(pat) = schema.get("pattern").and_then(|v| v.as_str()) {
        if let Ok(re) = cached_regex(pat) {
            if !re.is_match(s) {
                return Err(ValidationError::new(path, ErrorKind::Pattern,
                    format!("{s:?} does not match pattern {pat:?}")));
            }
        }
    }
    Ok(())
}

// ─── Array ────────────────────────────────────────────────────────────────────

fn check_array(
    schema: &serde_json::Map<String, Value>,
    arr: &[Value],
    path: &str,
    loader: &Loader,
) -> Result<(), ValidationError> {
    if let Some(min) = schema.get("minItems").and_then(as_usize) {
        if arr.len() < min { return Err(ValidationError::new(path, ErrorKind::MinItems,
            format!("array length {} < minItems {min}", arr.len()))); }
    }
    if let Some(max) = schema.get("maxItems").and_then(as_usize) {
        if arr.len() > max { return Err(ValidationError::new(path, ErrorKind::MaxItems,
            format!("array length {} > maxItems {max}", arr.len()))); }
    }
    if schema.get("uniqueItems").and_then(|v| v.as_bool()) == Some(true) {
        for i in 0..arr.len() {
            for j in (i + 1)..arr.len() {
                if deep_equal(&arr[i], &arr[j]) {
                    return Err(ValidationError::new(path, ErrorKind::UniqueItems,
                        format!("duplicate items at [{i}] and [{j}]")));
                }
            }
        }
    }
    if let Some(items) = schema.get("items") {
        if let Some(b) = items.as_bool() {
            if !b && !arr.is_empty() {
                return Err(ValidationError::new(path, ErrorKind::TypeError, "items schema is false — no items allowed"));
            }
        } else if items.is_object() {
            for (i, item) in arr.iter().enumerate() {
                validate_node(item, items, &format!("{path}[{i}]"), loader)?;
            }
        } else if let Some(item_schemas) = items.as_array() {
            for (i, item) in arr.iter().enumerate() {
                if let Some(item_schema) = item_schemas.get(i) {
                    validate_node(item, item_schema, &format!("{path}[{i}]"), loader)?;
                }
            }
            if arr.len() > item_schemas.len() {
                if let Some(additional) = schema.get("additionalItems") {
                    if additional == &Value::Bool(false) {
                        return Err(ValidationError::new(path, ErrorKind::MaxItems,
                            format!("array has {} items but tuple allows {}", arr.len(), item_schemas.len())));
                    }
                    if additional.is_object() {
                        for (i, item) in arr.iter().enumerate().skip(item_schemas.len()) {
                            validate_node(item, additional, &format!("{path}[{i}]"), loader)?;
                        }
                    }
                }
            }
        }
    }
    if let Some(contains_schema) = schema.get("contains") {
        if let Some(b) = contains_schema.as_bool() {
            if !b { return Err(ValidationError::new(path, ErrorKind::Contains, "contains schema is false")); }
            else if arr.is_empty() { return Err(ValidationError::new(path, ErrorKind::Contains, "contains: array is empty")); }
        } else if !arr.iter().any(|item| soft_validate(item, contains_schema, loader)) {
            return Err(ValidationError::new(path, ErrorKind::Contains, "no array element matches 'contains' schema"));
        }
    }
    Ok(())
}

// ─── Object ───────────────────────────────────────────────────────────────────

fn check_object(
    schema: &serde_json::Map<String, Value>,
    map: &serde_json::Map<String, Value>,
    full_value: &Value,
    path: &str,
    loader: &Loader,
) -> Result<(), ValidationError> {
    if let Some(Value::Array(req)) = schema.get("required") {
        for r in req {
            if let Some(field) = r.as_str() {
                if !map.contains_key(field) {
                    return Err(ValidationError::new(path, ErrorKind::Required,
                        format!("missing required field '{field}'")));
                }
            }
        }
    }
    if let Some(min) = schema.get("minProperties").and_then(as_usize) {
        if map.len() < min { return Err(ValidationError::new(path, ErrorKind::MinProperties,
            format!("object has {} properties, minProperties is {min}", map.len()))); }
    }
    if let Some(max) = schema.get("maxProperties").and_then(as_usize) {
        if map.len() > max { return Err(ValidationError::new(path, ErrorKind::MaxProperties,
            format!("object has {} properties, maxProperties is {max}", map.len()))); }
    }
    if let Some(pn_schema) = schema.get("propertyNames") {
        for key in map.keys() {
            let key_val = Value::String(key.clone());
            validate_node(&key_val, pn_schema, &format!("{path}.<key:{key}>"), loader)?;
        }
    }
    let props = schema.get("properties").and_then(|v| v.as_object());
    if let Some(props) = props {
        for (pk, ps) in props {
            if let Some(pv) = map.get(pk) {
                validate_node(pv, ps, &format!("{path}.{pk}"), loader)?;
            }
        }
    }
    let pattern_props = schema.get("patternProperties").and_then(|v| v.as_object());
    if let Some(pp) = pattern_props {
        for (pat, pat_schema) in pp {
            if let Ok(re) = cached_regex(pat) {
                for (vk, vv) in map {
                    if re.is_match(vk) { validate_node(vv, pat_schema, &format!("{path}.{vk}"), loader)?; }
                }
            }
        }
    }
    if let Some(ap) = schema.get("additionalProperties") {
        if ap != &Value::Bool(true) {
            let defined: std::collections::HashSet<&str> = props
                .map(|p| p.keys().map(|k| k.as_str()).collect()).unwrap_or_default();
            let pp_patterns: Vec<Regex> = pattern_props
                .map(|pp| pp.keys().filter_map(|k| cached_regex(k).ok()).collect()).unwrap_or_default();
            for (vk, vv) in map {
                if defined.contains(vk.as_str()) { continue; }
                if pp_patterns.iter().any(|re| re.is_match(vk)) { continue; }
                if ap == &Value::Bool(false) {
                    return Err(ValidationError::new(path, ErrorKind::AdditionalProperties,
                        format!("additional property '{vk}' not allowed")));
                }
                if ap.is_object() { validate_node(vv, ap, &format!("{path}.{vk}"), loader)?; }
            }
        }
    }
    for dep_kw in ["dependencies", "dependentRequired"] {
        if let Some(deps_obj) = schema.get(dep_kw).and_then(|v| v.as_object()) {
            for (dk, deps) in deps_obj {
                if !map.contains_key(dk) { continue; }
                match deps {
                    Value::Array(required) => {
                        for dep in required {
                            if let Some(dep_key) = dep.as_str() {
                                if !map.contains_key(dep_key) {
                                    return Err(ValidationError::new(path, ErrorKind::DependentRequired,
                                        format!("'{dk}' requires '{dep_key}'")));
                                }
                            }
                        }
                    }
                    dep_schema if dep_schema.is_object() || dep_schema.is_boolean() => {
                        validate_node(full_value, dep_schema, path, loader).map_err(|e| {
                            ValidationError::new(path, ErrorKind::DependencySchema,
                                format!("dependency '{dk}': {}", e.message))
                        })?;
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

// ─── ObjectTree ───────────────────────────────────────────────────────────────

pub struct ObjectTree {
    data: Value,
    loader: Loader,
}

unsafe impl Send for ObjectTree {}
unsafe impl Sync for ObjectTree {}

impl ObjectTree {
    /// Construct and validate. Panics if data is invalid against schema.
    pub fn new(data: Value, schema: Value) -> Self {
        Self::with_resolver(data, schema, None)
    }

    /// Construct with a remote schema directory for $ref resolution.
    pub fn with_resolver(data: Value, schema: Value, resolver: Option<String>) -> Self {
        let loader = Loader::new(schema, resolver);
        if let Err(e) = validate_node(&data, loader.root(), "$", &loader) {
            panic!("{}", e);
        }
        Self { data, loader }
    }

    /// Try to construct — returns Err if data is invalid. Does not panic.
    /// Validates first, then fills schema defaults (without re-validating).
    pub fn try_new(data: Value, schema: Value, resolver: Option<String>) -> Result<Self, ValidationError> {
        let loader = Loader::new(schema, resolver);
        validate_node(&data, loader.root(), "$", &loader)?;
        let mut data = data;
        apply_defaults(&mut data, loader.root());
        Ok(Self { data, loader })
    }

    fn unchecked(data: Value, loader: Loader) -> Self {
        Self { data, loader }
    }

    // ─── value ────────────────────────────────────────────────────────────────

    pub fn value(&self) -> &Value { &self.data }

    pub fn set_value(&mut self, v: Value) -> Result<(), ValidationError> {
        validate_node(&v, self.loader.root(), "$", &self.loader)?;
        self.data = v;
        Ok(())
    }

    // ─── get / set ────────────────────────────────────────────────────────────

    pub fn get(&self, key: &str) -> Option<&Value> { self.data.get(key) }

    pub fn set(&mut self, key: &str, value: Value) -> Result<(), ValidationError> {
        let sub_schema = self.loader.root()
            .get("properties").and_then(|p| p.as_object()).and_then(|p| p.get(key));
        if let Some(ss) = sub_schema {
            validate_node(&value, ss, &format!("$.{key}"), &self.loader)?;
        }
        if let Some(obj) = self.data.as_object_mut() {
            obj.insert(key.to_string(), value);
            Ok(())
        } else {
            Err(ValidationError::new("$", ErrorKind::NotAnObject, "data is not an object"))
        }
    }

    // ─── to_dict ──────────────────────────────────────────────────────────────

    pub fn to_dict(&self) -> Value {
        let schema = self.loader.root();
        if let Some(props) = schema.get("properties").and_then(|p| p.as_object()) {
            if let Some(obj) = self.data.as_object() {
                let mut out = serde_json::Map::new();
                for key in props.keys() {
                    if let Some(v) = obj.get(key) { out.insert(key.clone(), v.clone()); }
                }
                return Value::Object(out);
            }
        }
        self.data.clone()
    }

    // ─── schema access ────────────────────────────────────────────────────────

    pub fn schema(&self) -> Value { schema_to_dict(self.loader.root()) }

    pub fn get_schema(&self, path: &str) -> Option<Value> {
        let schema = self.loader.root();
        if path.is_empty() { return Some(schema_to_dict(schema)); }
        let mut node = schema;
        for part in path.split('.').filter(|p| !p.is_empty()) {
            let props = node.get("properties")?.as_object()?;
            node = props.get(part)?;
        }
        Some(schema_to_dict(node))
    }

    pub fn get_extensions(&self, path: &str) -> serde_json::Map<String, Value> {
        let mut out = serde_json::Map::new();
        let node = if path.is_empty() { Some(self.schema()) } else { self.get_schema(path) };
        if let Some(Value::Object(obj)) = node {
            for (k, v) in obj { if k.starts_with("x-") { out.insert(k, v); } }
        }
        out
    }

    // ─── with_defaults ────────────────────────────────────────────────────────

    pub fn with_defaults(self) -> Self {
        let mut data = self.data.clone();
        apply_defaults(&mut data, self.loader.root());
        Self::unchecked(data, self.loader)
    }

    // ─── validate ─────────────────────────────────────────────────────────────

    pub fn validate(&self) -> Result<(), ValidationError> {
        validate_node(&self.data, self.loader.root(), "$", &self.loader)
    }

    // ─── oneOf ────────────────────────────────────────────────────────────────

    pub fn one_of(&self) -> Result<ObjectTree, ValidationError> {
        let subs = self.loader.root().get("oneOf").and_then(|v| v.as_array())
            .ok_or_else(|| ValidationError::new("$", ErrorKind::TypeError, "schema has no oneOf"))?;
        let matches: Vec<&Value> = subs.iter()
            .filter(|s| validate_node(&self.data, s, "$", &self.loader).is_ok()).collect();
        match matches.len() {
            1 => Ok(Self::unchecked(self.data.clone(), Loader::new(matches[0].clone(), self.loader.resolver()))),
            n => Err(ValidationError::new("$", ErrorKind::OneOfNoneMatch,
                format!("oneOf: expected 1 match, got {n}"))),
        }
    }

    // ─── anyOf ────────────────────────────────────────────────────────────────

    pub fn any_of(&self) -> Result<Vec<ObjectTree>, ValidationError> {
        let subs = self.loader.root().get("anyOf").and_then(|v| v.as_array())
            .ok_or_else(|| ValidationError::new("$", ErrorKind::TypeError, "schema has no anyOf"))?;
        let results: Vec<ObjectTree> = subs.iter()
            .filter(|s| validate_node(&self.data, s, "$", &self.loader).is_ok())
            .map(|s| Self::unchecked(self.data.clone(), Loader::new(s.clone(), self.loader.resolver())))
            .collect();
        if results.is_empty() {
            Err(ValidationError::new("$", ErrorKind::AnyOfNoneMatch, "anyOf: no branch matches"))
        } else { Ok(results) }
    }

    // ─── allOf ────────────────────────────────────────────────────────────────

    pub fn all_of(&self) -> ObjectTree {
        let subs = match self.loader.root().get("allOf").and_then(|v| v.as_array()) {
            Some(a) => a,
            None => return Self::unchecked(self.data.clone(),
                Loader::new(self.loader.root().clone(), self.loader.resolver())),
        };
        let merged = subs.iter().fold(Value::Object(serde_json::Map::new()), deep_merge);
        Self::unchecked(self.data.clone(), Loader::new(merged, self.loader.resolver()))
    }

    // ─── notOf ────────────────────────────────────────────────────────────────

    pub fn not_of(&self) -> bool {
        match self.loader.root().get("not") {
            Some(not_schema) => validate_node(&self.data, not_schema, "$", &self.loader).is_err(),
            None => true,
        }
    }

    // ─── ifThen ───────────────────────────────────────────────────────────────

    pub fn if_then(&self) -> ObjectTree {
        let schema = self.loader.root();
        let if_schema = match schema.get("if") {
            Some(s) => s,
            None => return Self::unchecked(self.data.clone(), Loader::new(schema.clone(), self.loader.resolver())),
        };
        let branch = if validate_node(&self.data, if_schema, "$", &self.loader).is_ok() {
            schema.get("then")
        } else { schema.get("else") };
        match branch {
            Some(b) => Self::unchecked(self.data.clone(), Loader::new(b.clone(), self.loader.resolver())),
            None => Self::unchecked(self.data.clone(), Loader::new(schema.clone(), self.loader.resolver())),
        }
    }

    // ─── project ──────────────────────────────────────────────────────────────

    pub fn project(&self) -> Result<ObjectTree, ValidationError> {
        let data = self.data.as_object()
            .ok_or_else(|| ValidationError::new("$", ErrorKind::TypeError, "project: data must be an object"))?;
        let schema = self.loader.root();
        let props = schema.get("properties").and_then(|p| p.as_object());
        let filtered: serde_json::Map<String, Value> = match props {
            Some(props) => props.keys()
                .filter_map(|k| data.get(k).map(|v| (k.clone(), v.clone()))).collect(),
            None => data.clone(),
        };
        Ok(Self::unchecked(Value::Object(filtered), Loader::new(schema.clone(), self.loader.resolver())))
    }

    // ─── contains ─────────────────────────────────────────────────────────────

    pub fn contains(&self) -> Option<bool> {
        let arr = self.data.as_array()?;
        let cs = self.loader.root().get("contains")?;
        Some(arr.iter().any(|item| validate_node(item, cs, "$", &self.loader).is_ok()))
    }

    pub fn to_value(&self) -> Value { self.data.clone() }
}

impl fmt::Display for ObjectTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.data) }
}

impl fmt::Debug for ObjectTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "ObjectTree({})", self.data) }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn schema_to_dict(schema: &Value) -> Value {
    match schema {
        Value::Object(obj) => {
            let mut out = serde_json::Map::new();
            for (k, v) in obj {
                out.insert(k.clone(), if k == "properties" {
                    if let Value::Object(props) = v {
                        Value::Object(props.iter().map(|(pk, pv)| (pk.clone(), schema_to_dict(pv))).collect())
                    } else { v.clone() }
                } else if let Value::Array(arr) = v {
                    Value::Array(arr.iter().map(schema_to_dict).collect())
                } else if v.is_object() {
                    schema_to_dict(v)
                } else { v.clone() });
            }
            Value::Object(out)
        }
        _ => schema.clone(),
    }
}

fn apply_defaults(data: &mut Value, schema: &Value) {
    let props = match schema.get("properties").and_then(|p| p.as_object()) {
        Some(p) => p, None => return,
    };
    let obj = match data.as_object_mut() { Some(o) => o, None => return };
    for (key, prop_schema) in props {
        if obj.contains_key(key) {
            if let Some(child) = obj.get_mut(key) {
                if child.is_object() && prop_schema.is_object() {
                    apply_defaults(child, prop_schema);
                }
            }
        } else if let Some(default) = prop_schema.get("default") {
            obj.insert(key.clone(), default.clone());
        }
    }
}

// ─── validate (pub export) ────────────────────────────────────────────────────

pub fn validate(data: &Value, schema: &Value, resolver: Option<String>) -> Result<(), ValidationError> {
    let loader = Loader::new(schema.clone(), resolver);
    validate_node(data, schema, "$", &loader)
}

// ─── Error types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub path: String,
    pub kind: ErrorKind,
    pub message: String,
}

impl ValidationError {
    pub fn new(path: impl Into<String>, kind: ErrorKind, message: impl Into<String>) -> Self {
        Self { path: path.into(), kind, message: message.into() }
    }
    pub fn root(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self::new("", kind, message)
    }
    pub fn at_field(mut self, field: &str) -> Self {
        self.path = if self.path.is_empty() { format!(".{field}") } else { format!(".{field}{}", self.path) };
        self
    }
    pub fn at_index(mut self, idx: usize) -> Self {
        self.path = if self.path.is_empty() { format!("[{idx}]") } else { format!("[{idx}]{}", self.path) };
        self
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.path.is_empty() { write!(f, "{}", self.message) }
        else { write!(f, "{}: {}", self.path, self.message) }
    }
}

impl std::error::Error for ValidationError {}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorKind {
    TypeError,
    ConstMismatch,
    EnumMismatch,
    Minimum, Maximum, ExclusiveMinimum, ExclusiveMaximum, MultipleOf,
    MinLength, MaxLength, Pattern,
    MinItems, MaxItems, UniqueItems, Contains,
    Required, AdditionalProperties, MinProperties, MaxProperties,
    DependentRequired, DependencySchema,
    OneOfNoneMatch, OneOfMultipleMatch,
    AnyOfNoneMatch,
    IfThenFailed,
    NotAnObject,
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn as_usize(v: &Value) -> Option<usize> {
    if let Some(n) = v.as_u64() { return Some(n as usize); }
    v.as_f64().filter(|f| f.fract() == 0.0 && *f >= 0.0).map(|f| f as usize)
}

fn is_whole_f64(v: &Value) -> bool {
    v.as_f64().map(|f| f.fract() == 0.0).unwrap_or(false)
}

fn value_type_name(v: &Value) -> &'static str {
    match v {
        Value::Null => "null", Value::Bool(_) => "boolean", Value::Number(_) => "number",
        Value::String(_) => "string", Value::Array(_) => "array", Value::Object(_) => "object",
    }
}
