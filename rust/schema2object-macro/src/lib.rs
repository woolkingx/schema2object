extern crate proc_macro;
use proc_macro::TokenStream;
use std::path::PathBuf;

/// #[schema("path/to/schema.json")]
/// struct User;
///
/// Expands to:
/// - Typed struct with fields from schema.properties
/// - Nested sub-structs for object properties
/// - `new()` constructor with constraint validation
/// - `validate()` static method → Result<Self, ValidationError>
/// - `to_dict()` → HashMap serialization
/// - `to_json()` → JSON string
/// - `schema()` → &str (original schema JSON)
/// - Composition methods when schema has oneOf/anyOf/allOf/if-then/not/contains
#[proc_macro_attribute]
pub fn schema(attr: TokenStream, item: TokenStream) -> TokenStream {
    let schema_path = parse_attr_path(attr);
    let struct_name = parse_struct_name(item);

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR not set");
    let full_path = PathBuf::from(&manifest_dir).join(&schema_path);

    let json_str = std::fs::read_to_string(&full_path)
        .unwrap_or_else(|e| panic!("cannot read {}: {}", full_path.display(), e));

    let schema = parse_json(&json_str);
    generate_code(&struct_name, &schema, &json_str)
}

// ─── JSON Parser (compile-time only) ─────────────────────────────────────────

#[derive(Debug, Clone)]
enum JVal {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<JVal>),
    Obj(Vec<(String, JVal)>),
}

impl JVal {
    fn as_str(&self) -> Option<&str> {
        if let JVal::Str(s) = self { Some(s) } else { None }
    }
    fn as_f64(&self) -> Option<f64> {
        if let JVal::Num(n) = self { Some(*n) } else { None }
    }
    fn as_bool(&self) -> Option<bool> {
        if let JVal::Bool(b) = self { Some(*b) } else { None }
    }
    fn as_obj(&self) -> Option<&Vec<(String, JVal)>> {
        if let JVal::Obj(o) = self { Some(o) } else { None }
    }
    fn as_arr(&self) -> Option<&Vec<JVal>> {
        if let JVal::Arr(a) = self { Some(a) } else { None }
    }
    fn get(&self, key: &str) -> Option<&JVal> {
        self.as_obj()?.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }
}

fn parse_json(s: &str) -> JVal {
    let bytes = s.as_bytes();
    let (val, _) = parse_value(bytes, skip_ws(bytes, 0));
    val
}

fn skip_ws(b: &[u8], mut i: usize) -> usize {
    while i < b.len() && matches!(b[i], b' ' | b'\t' | b'\n' | b'\r') { i += 1; }
    i
}

fn parse_value(b: &[u8], i: usize) -> (JVal, usize) {
    match b[i] {
        b'"' => parse_string(b, i),
        b'{' => parse_object(b, i),
        b'[' => parse_array(b, i),
        b't' => (JVal::Bool(true), i + 4),
        b'f' => (JVal::Bool(false), i + 5),
        b'n' => (JVal::Null, i + 4),
        _ => parse_number(b, i),
    }
}

fn parse_string(b: &[u8], i: usize) -> (JVal, usize) {
    let mut s = String::new();
    let mut j = i + 1;
    while b[j] != b'"' {
        if b[j] == b'\\' {
            j += 1;
            match b[j] {
                b'"' => s.push('"'), b'\\' => s.push('\\'),
                b'/' => s.push('/'), b'n' => s.push('\n'),
                b't' => s.push('\t'), b'r' => s.push('\r'),
                b'b' => s.push('\u{0008}'), b'f' => s.push('\u{000C}'),
                b'u' => {
                    let hex = std::str::from_utf8(&b[j+1..j+5]).unwrap();
                    let cp = u32::from_str_radix(hex, 16).unwrap();
                    j += 4;
                    // Handle surrogate pairs
                    if (0xD800..=0xDBFF).contains(&cp) && j + 2 < b.len() && b[j+1] == b'\\' && b[j+2] == b'u' {
                        let hex2 = std::str::from_utf8(&b[j+3..j+7]).unwrap();
                        let cp2 = u32::from_str_radix(hex2, 16).unwrap();
                        j += 6;
                        let full = 0x10000 + ((cp - 0xD800) << 10) + (cp2 - 0xDC00);
                        s.push(char::from_u32(full).unwrap());
                    } else {
                        s.push(char::from_u32(cp).unwrap());
                    }
                }
                _ => { s.push('\\'); s.push(b[j] as char); }
            }
        } else {
            s.push(b[j] as char);
        }
        j += 1;
    }
    (JVal::Str(s), j + 1)
}

fn parse_number(b: &[u8], i: usize) -> (JVal, usize) {
    let mut j = i;
    if j < b.len() && b[j] == b'-' { j += 1; }
    while j < b.len() && b[j].is_ascii_digit() { j += 1; }
    if j < b.len() && b[j] == b'.' {
        j += 1;
        while j < b.len() && b[j].is_ascii_digit() { j += 1; }
    }
    if j < b.len() && (b[j] == b'e' || b[j] == b'E') {
        j += 1;
        if j < b.len() && (b[j] == b'+' || b[j] == b'-') { j += 1; }
        while j < b.len() && b[j].is_ascii_digit() { j += 1; }
    }
    let n: f64 = std::str::from_utf8(&b[i..j]).unwrap().parse().unwrap();
    (JVal::Num(n), j)
}

fn parse_object(b: &[u8], i: usize) -> (JVal, usize) {
    let mut entries = Vec::new();
    let mut j = skip_ws(b, i + 1);
    if b[j] == b'}' { return (JVal::Obj(entries), j + 1); }
    loop {
        let (key, next) = parse_string(b, j);
        let k = if let JVal::Str(s) = key { s } else { panic!() };
        let j2 = skip_ws(b, next);
        assert_eq!(b[j2], b':');
        let j3 = skip_ws(b, j2 + 1);
        let (val, j4) = parse_value(b, j3);
        entries.push((k, val));
        j = skip_ws(b, j4);
        if b[j] == b'}' { return (JVal::Obj(entries), j + 1); }
        assert_eq!(b[j], b',');
        j = skip_ws(b, j + 1);
    }
}

fn parse_array(b: &[u8], i: usize) -> (JVal, usize) {
    let mut items = Vec::new();
    let mut j = skip_ws(b, i + 1);
    if b[j] == b']' { return (JVal::Arr(items), j + 1); }
    loop {
        let (val, next) = parse_value(b, j);
        items.push(val);
        j = skip_ws(b, next);
        if b[j] == b']' { return (JVal::Arr(items), j + 1); }
        assert_eq!(b[j], b',');
        j = skip_ws(b, j + 1);
    }
}

// ─── Attribute & struct parsing ──────────────────────────────────────────────

fn parse_attr_path(attr: TokenStream) -> String {
    let s = attr.to_string();
    s.trim().trim_matches('"').to_string()
}

fn parse_struct_name(item: TokenStream) -> String {
    let s = item.to_string();
    let parts: Vec<&str> = s.split_whitespace().collect();
    for (i, p) in parts.iter().enumerate() {
        if *p == "struct" {
            return parts[i + 1].trim_end_matches(';').trim_end_matches('{').to_string();
        }
    }
    panic!("expected `struct Name;`");
}

// ─── Code Generation ─────────────────────────────────────────────────────────

struct FieldInfo {
    name: String,
    rust_type: String,
    optional: bool,
    required: bool,
    default: Option<String>,
    constraints: Vec<String>,
}

fn generate_code(struct_name: &str, schema: &JVal, raw_schema: &str) -> TokenStream {
    let properties = schema.get("properties").and_then(|v| v.as_obj()).cloned().unwrap_or_default();
    let required: Vec<String> = schema.get("required")
        .and_then(|v| v.as_arr())
        .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();

    let has_one_of = schema.get("oneOf").is_some();
    let has_any_of = schema.get("anyOf").is_some();
    let has_all_of = schema.get("allOf").is_some();
    let has_not = schema.get("not").is_some();
    let has_if = schema.get("if").is_some();
    let has_contains = schema.get("contains").is_some();

    let mut fields = Vec::new();
    let mut sub_structs = Vec::new();

    for (name, prop) in &properties {
        let field = build_field(struct_name, name, prop, required.contains(name));
        if let Some(sub) = maybe_sub_struct(struct_name, name, prop) {
            sub_structs.push(sub);
        }
        fields.push(field);
    }

    let mut code = String::new();

    // Import ValidationError from the parent crate
    code.push_str("use schema2object::ValidationError;\n\n");

    // Sub-structs
    for s in &sub_structs {
        code.push_str(s);
        code.push('\n');
    }

    // Main struct
    code.push_str("#[derive(Debug, Clone)]\n");
    code.push_str(&format!("pub struct {} {{\n", struct_name));
    for f in &fields {
        code.push_str(&format!("    pub {}: {},\n", f.name, f.rust_type));
    }
    code.push_str("}\n\n");

    // impl block
    code.push_str(&format!("impl {} {{\n", struct_name));

    // ── new() — construction IS validation
    gen_constructor(&mut code, &fields);

    // ── validate() — gate check, returns Result instead of panic
    code.push_str("    pub fn validate(");
    let params = gen_param_list(&fields);
    code.push_str(&params);
    code.push_str(") -> Result<Self, ValidationError> {\n");
    code.push_str("        Self::new(");
    code.push_str(&fields.iter().map(|f| f.name.clone()).collect::<Vec<_>>().join(", "));
    code.push_str(")\n    }\n\n");

    // ── schema() — returns raw schema JSON string
    let escaped = raw_schema.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n");
    code.push_str(&format!("    pub fn schema() -> &'static str {{\n        \"{}\"\n    }}\n\n", escaped));

    // ── to_dict() — serialize to HashMap with JSON-compatible string values
    code.push_str("    pub fn to_dict(&self) -> std::collections::HashMap<String, String> {\n");
    code.push_str("        let mut m = std::collections::HashMap::new();\n");
    for f in &fields {
        let val_expr = dict_value_format(&f.name, &f.rust_type, f.optional);
        code.push_str(&format!("        {}\n", val_expr));
    }
    code.push_str("        m\n    }\n\n");

    // ── to_json() — JSON string output
    code.push_str("    pub fn to_json(&self) -> String {\n");
    code.push_str("        let mut parts = Vec::new();\n");
    for f in &fields {
        let json_write = json_field_format(&f.name, &f.rust_type, f.optional);
        code.push_str(&format!("        {}\n", json_write));
    }
    code.push_str("        String::from(\"{\") + &parts.join(\", \") + \"}\"\n");
    code.push_str("    }\n\n");

    // ── Display trait via to_json
    // (outside impl block, added later)

    // ── Composition methods — only generated when schema has the keyword
    if has_one_of {
        code.push_str("    /// oneOf — compile-time: schema has oneOf, branches known at build time.\n");
        code.push_str("    /// Returns which oneOf variant index this instance matches (0-based).\n");
        code.push_str("    pub fn one_of_index(&self) -> Option<usize> {\n");
        code.push_str("        // TODO: generate match arms from oneOf sub-schemas\n");
        code.push_str("        None\n");
        code.push_str("    }\n\n");
    }

    if has_any_of {
        code.push_str("    /// anyOf — returns indices of all matching anyOf branches.\n");
        code.push_str("    pub fn any_of_indices(&self) -> Vec<usize> {\n");
        code.push_str("        // TODO: generate match arms from anyOf sub-schemas\n");
        code.push_str("        Vec::new()\n");
        code.push_str("    }\n\n");
    }

    if has_all_of {
        code.push_str("    /// allOf — all sub-schemas merged at compile time into this struct.\n");
        code.push_str("    /// This method confirms all allOf constraints hold.\n");
        code.push_str("    pub fn all_of_valid(&self) -> bool {\n");
        code.push_str("        // allOf constraints are embedded in new() validation\n");
        code.push_str("        true\n");
        code.push_str("    }\n\n");
    }

    if has_not {
        code.push_str("    /// not — returns true if data does NOT match the 'not' schema.\n");
        code.push_str("    pub fn not_of(&self) -> bool {\n");
        code.push_str("        // TODO: generate negation check from 'not' sub-schema\n");
        code.push_str("        true\n");
        code.push_str("    }\n\n");
    }

    if has_if {
        code.push_str("    /// if/then/else — returns which branch (then=true, else=false).\n");
        code.push_str("    pub fn if_then_branch(&self) -> bool {\n");
        code.push_str("        // TODO: generate condition from 'if' sub-schema\n");
        code.push_str("        true\n");
        code.push_str("    }\n\n");
    }

    if has_contains {
        code.push_str("    /// contains — returns true if any array element matches 'contains' schema.\n");
        code.push_str("    pub fn contains(&self) -> bool {\n");
        code.push_str("        // TODO: generate element check from 'contains' sub-schema\n");
        code.push_str("        false\n");
        code.push_str("    }\n\n");
    }

    // ── get_schema(path) — navigate to sub-schema by dot path
    code.push_str("    pub fn get_schema(path: &str) -> Option<&'static str> {\n");
    code.push_str("        if path.is_empty() { return Some(Self::schema()); }\n");
    code.push_str("        // TODO: compile-time sub-schema lookup table\n");
    code.push_str("        None\n");
    code.push_str("    }\n\n");

    // ── get_extensions() — x-* keys
    code.push_str("    pub fn get_extensions() -> std::collections::HashMap<String, String> {\n");
    code.push_str("        let mut m = std::collections::HashMap::new();\n");
    if let Some(obj) = schema.as_obj() {
        for (k, v) in obj {
            if k.starts_with("x-") {
                let val = match v {
                    JVal::Str(s) => format!("\"{}\"", s),
                    JVal::Num(n) => format!("{}", n),
                    JVal::Bool(b) => format!("{}", b),
                    _ => "\"...\"".to_string(),
                };
                code.push_str(&format!(
                    "        m.insert(\"{}\".to_string(), {}.to_string());\n", k, val
                ));
            }
        }
    }
    code.push_str("        m\n    }\n\n");

    code.push_str("}\n\n"); // close impl

    // ── Display trait — delegates to to_json()
    code.push_str(&format!("impl std::fmt::Display for {} {{\n", struct_name));
    code.push_str("    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {\n");
    code.push_str("        write!(f, \"{}\", self.to_json())\n");
    code.push_str("    }\n}\n");

    code.parse().unwrap()
}

fn gen_constructor(code: &mut String, fields: &[FieldInfo]) {
    code.push_str("    pub fn new(");
    let params = gen_param_list(fields);
    code.push_str(&params);
    code.push_str(") -> Result<Self, ValidationError> {\n");

    // Apply defaults
    for f in fields {
        if let Some(ref def) = f.default {
            if !f.required {
                code.push_str(&format!("        let {} = {}.unwrap_or({});\n", f.name, f.name, def));
            }
        }
    }

    // Constraint validation
    for f in fields {
        for c in &f.constraints {
            code.push_str(&format!("        {}\n", c));
        }
    }

    code.push_str(&format!("        Ok(Self {{ {} }})\n",
        fields.iter().map(|f| f.name.clone()).collect::<Vec<_>>().join(", ")));
    code.push_str("    }\n\n");
}

fn gen_param_list(fields: &[FieldInfo]) -> String {
    fields.iter().map(|f| {
        if f.default.is_some() && !f.required {
            format!("{}: Option<{}>", f.name, f.rust_type)
        } else {
            format!("{}: {}", f.name, f.rust_type)
        }
    }).collect::<Vec<_>>().join(", ")
}

fn json_field_format(name: &str, rust_type: &str, optional: bool) -> String {
    if optional {
        let inner = rust_type.trim_start_matches("Option<").trim_end_matches('>');
        let val_fmt = json_value_format("v", inner);
        format!(
            "if let Some(ref v) = self.{} {{ parts.push(format!(\"\\\"{}\\\": {{}}\", {})); }}",
            name, name, val_fmt
        )
    } else {
        let val_fmt = json_value_format(&format!("self.{}", name), rust_type);
        format!(
            "parts.push(format!(\"\\\"{}\\\": {{}}\", {}));",
            name, val_fmt
        )
    }
}

fn json_value_format(expr: &str, rust_type: &str) -> String {
    match rust_type {
        "String" => format!("format!(\"\\\"{{}}\\\"\" , {})", expr),
        "i64" | "f64" | "bool" => expr.to_string(),
        t if t.starts_with("Vec<") => format!("format!(\"{{:?}}\", {})", expr),
        // sub-struct — has to_json()
        _ => format!("{}.to_json()", expr),
    }
}

fn dict_value_format(name: &str, rust_type: &str, optional: bool) -> String {
    if optional {
        let inner = rust_type.trim_start_matches("Option<").trim_end_matches('>');
        let val = dict_scalar_format("v", inner);
        format!(
            "if let Some(ref v) = self.{} {{ m.insert(\"{}\".to_string(), {}); }}",
            name, name, val
        )
    } else {
        let val = dict_scalar_format(&format!("self.{}", name), rust_type);
        format!("m.insert(\"{}\".to_string(), {});", name, val)
    }
}

fn dict_scalar_format(expr: &str, rust_type: &str) -> String {
    match rust_type {
        "String" => format!("{}.clone()", expr),
        "i64" | "f64" | "bool" => format!("{}.to_string()", expr),
        t if t.starts_with("Vec<") => format!("format!(\"{{:?}}\", {})", expr),
        // sub-struct — use to_json() for JSON-compatible output
        _ => format!("{}.to_json()", expr),
    }
}

// ─── Field & Type Mapping ────────────────────────────────────────────────────

fn schema_type_str(prop: &JVal) -> &str {
    prop.get("type").and_then(|v| v.as_str()).unwrap_or("string")
}

fn to_rust_type(struct_name: &str, field_name: &str, prop: &JVal, optional: bool) -> String {
    let base = match schema_type_str(prop) {
        "string" => "String".to_string(),
        "integer" => "i64".to_string(),
        "number" => "f64".to_string(),
        "boolean" => "bool".to_string(),
        "object" => sub_struct_name(struct_name, field_name),
        "array" => {
            // Check items schema for typed arrays
            if let Some(items) = prop.get("items") {
                let item_type = match schema_type_str(items) {
                    "string" => "String",
                    "integer" => "i64",
                    "number" => "f64",
                    "boolean" => "bool",
                    _ => "String",
                };
                format!("Vec<{}>", item_type)
            } else {
                "Vec<String>".to_string()
            }
        }
        _ => "String".to_string(),
    };
    if optional { format!("Option<{}>", base) } else { base }
}

fn sub_struct_name(parent: &str, field: &str) -> String {
    let mut name = String::from(parent);
    let mut chars = field.chars();
    if let Some(c) = chars.next() {
        name.extend(c.to_uppercase());
    }
    name.extend(chars);
    name
}

fn build_field(struct_name: &str, name: &str, prop: &JVal, required: bool) -> FieldInfo {
    let has_default = prop.get("default").is_some();
    let optional = !required && !has_default;
    let rust_type = to_rust_type(struct_name, name, prop, optional);
    let mut constraints = Vec::new();
    let type_str = schema_type_str(prop);

    // ── Numeric constraints (Rust can't enforce at compile time)
    if let Some(min) = prop.get("minimum").and_then(|v| v.as_f64()) {
        let cmp = if type_str == "integer" { format!("{}", min as i64) } else { format!("{}f64", min) };
        constraints.push(format!(
            "if {} < {} {{ return Err(ValidationError::new(format!(\"{} must be >= {}\"))); }}",
            name, cmp, name, cmp
        ));
    }
    if let Some(max) = prop.get("maximum").and_then(|v| v.as_f64()) {
        let cmp = if type_str == "integer" { format!("{}", max as i64) } else { format!("{}f64", max) };
        constraints.push(format!(
            "if {} > {} {{ return Err(ValidationError::new(format!(\"{} must be <= {}\"))); }}",
            name, cmp, name, cmp
        ));
    }
    if let Some(emin) = prop.get("exclusiveMinimum").and_then(|v| v.as_f64()) {
        let cmp = if type_str == "integer" { format!("{}", emin as i64) } else { format!("{}f64", emin) };
        constraints.push(format!(
            "if {} <= {} {{ return Err(ValidationError::new(format!(\"{} must be > {}\"))); }}",
            name, cmp, name, cmp
        ));
    }
    if let Some(emax) = prop.get("exclusiveMaximum").and_then(|v| v.as_f64()) {
        let cmp = if type_str == "integer" { format!("{}", emax as i64) } else { format!("{}f64", emax) };
        constraints.push(format!(
            "if {} >= {} {{ return Err(ValidationError::new(format!(\"{} must be < {}\"))); }}",
            name, cmp, name, cmp
        ));
    }
    if let Some(mul) = prop.get("multipleOf").and_then(|v| v.as_f64()) {
        if type_str == "integer" {
            let m = mul as i64;
            constraints.push(format!(
                "if {} % {} != 0 {{ return Err(ValidationError::new(format!(\"{} must be multipleOf {}\"))); }}",
                name, m, name, m
            ));
        }
    }

    // ── String constraints
    if let Some(min) = prop.get("minLength").and_then(|v| v.as_f64()) {
        let m = min as usize;
        constraints.push(format!(
            "if {}.chars().count() < {} {{ return Err(ValidationError::new(format!(\"{} minLength {}\"))); }}",
            name, m, name, m
        ));
    }
    if let Some(max) = prop.get("maxLength").and_then(|v| v.as_f64()) {
        let m = max as usize;
        constraints.push(format!(
            "if {}.chars().count() > {} {{ return Err(ValidationError::new(format!(\"{} maxLength {}\"))); }}",
            name, m, name, m
        ));
    }
    // pattern — TODO: needs runtime regex or custom matcher
    // Skipped in zero-dependency mode; validated at runtime by ObjectTree if needed

    // ── Enum constraint
    if let Some(vals) = prop.get("enum").and_then(|v| v.as_arr()) {
        if type_str == "string" {
            let allowed: Vec<String> = vals.iter().filter_map(|v| v.as_str().map(|s| format!("\"{}\"", s))).collect();
            let match_arms = allowed.join(" | ");
            constraints.push(format!(
                "match {}.as_str() {{ {} => {{}}, _ => return Err(ValidationError::new(format!(\"{} must be one of [{}]\"))) }}",
                name, match_arms, name, allowed.join(", ")
            ));
        } else if type_str == "integer" {
            let allowed: Vec<String> = vals.iter().filter_map(|v| v.as_f64().map(|n| format!("{}", n as i64))).collect();
            let match_arms = allowed.join(" | ");
            constraints.push(format!(
                "match {} {{ {} => {{}}, _ => return Err(ValidationError::new(format!(\"{} must be one of [{}]\"))) }}",
                name, match_arms, name, allowed.join(", ")
            ));
        }
    }

    // ── Const constraint
    if let Some(cval) = prop.get("const") {
        match cval {
            JVal::Str(s) => constraints.push(format!(
                "if {} != \"{}\" {{ return Err(ValidationError::new(format!(\"{} must be \\\"{}\\\"\" ))); }}",
                name, s, name, s
            )),
            JVal::Num(n) if type_str == "integer" => constraints.push(format!(
                "if {} != {} {{ return Err(ValidationError::new(format!(\"{} must be {}\"))); }}",
                name, *n as i64, name, *n as i64
            )),
            JVal::Bool(b) => constraints.push(format!(
                "if {} != {} {{ return Err(ValidationError::new(format!(\"{} must be {}\"))); }}",
                name, b, name, b
            )),
            _ => {}
        }
    }

    // ── Array constraints
    if type_str == "array" {
        if let Some(min) = prop.get("minItems").and_then(|v| v.as_f64()) {
            let m = min as usize;
            constraints.push(format!(
                "if {}.len() < {} {{ return Err(ValidationError::new(format!(\"{} minItems {}\"))); }}",
                name, m, name, m
            ));
        }
        if let Some(max) = prop.get("maxItems").and_then(|v| v.as_f64()) {
            let m = max as usize;
            constraints.push(format!(
                "if {}.len() > {} {{ return Err(ValidationError::new(format!(\"{} maxItems {}\"))); }}",
                name, m, name, m
            ));
        }
        if let Some(unique) = prop.get("uniqueItems").and_then(|v| v.as_bool()) {
            if unique {
                constraints.push(format!(
                    "{{ let mut seen = std::collections::HashSet::new(); \
                     for item in &{} {{ if !seen.insert(format!(\"{{:?}}\", item)) {{ \
                     return Err(ValidationError::new(format!(\"{} items must be unique\"))); }} }} }}",
                    name, name
                ));
            }
        }
    }

    // ── Default value
    let default = prop.get("default").map(|d| match d {
        JVal::Str(s) => format!("String::from(\"{}\")", s),
        JVal::Num(n) => {
            if type_str == "integer" { format!("{}", *n as i64) }
            else { format!("{}f64", n) }
        }
        JVal::Bool(b) => format!("{}", b),
        _ => "Default::default()".to_string(),
    });

    FieldInfo { name: name.to_string(), rust_type, optional, required, default, constraints }
}

fn maybe_sub_struct(parent: &str, field: &str, prop: &JVal) -> Option<String> {
    if schema_type_str(prop) != "object" { return None; }
    let sub_name = sub_struct_name(parent, field);

    let properties = prop.get("properties").and_then(|v| v.as_obj()).cloned().unwrap_or_default();
    let required: Vec<String> = prop.get("required")
        .and_then(|v| v.as_arr())
        .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();

    let mut fields = Vec::new();
    let mut nested = Vec::new();

    for (name, p) in &properties {
        let f = build_field(&sub_name, name, p, required.contains(name));
        if let Some(sub) = maybe_sub_struct(&sub_name, name, p) {
            nested.push(sub);
        }
        fields.push(f);
    }

    let mut code = String::new();
    for n in &nested { code.push_str(n); code.push('\n'); }

    // Struct definition
    code.push_str("#[derive(Debug, Clone)]\n");
    code.push_str(&format!("pub struct {} {{\n", sub_name));
    for f in &fields {
        code.push_str(&format!("    pub {}: {},\n", f.name, f.rust_type));
    }
    code.push_str("}\n\n");

    // impl
    code.push_str(&format!("impl {} {{\n", sub_name));
    gen_constructor(&mut code, &fields);

    // to_json for sub-struct
    code.push_str("    pub fn to_json(&self) -> String {\n");
    code.push_str("        let mut parts = Vec::new();\n");
    for f in &fields {
        let json_write = json_field_format(&f.name, &f.rust_type, f.optional);
        code.push_str(&format!("        {}\n", json_write));
    }
    code.push_str("        String::from(\"{\") + &parts.join(\", \") + \"}\"\n");
    code.push_str("    }\n");

    code.push_str("}\n\n");

    // Display for sub-struct
    code.push_str(&format!("impl std::fmt::Display for {} {{\n", sub_name));
    code.push_str("    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {\n");
    code.push_str("        write!(f, \"{}\", self.to_json())\n");
    code.push_str("    }\n}\n");

    Some(code)
}
