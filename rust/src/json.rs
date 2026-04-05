/// JSON parser — zero dependencies.
/// Parses JSON string into JsonNode AST.

use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum JsonNode {
    Object(HashMap<String, JsonNode>),
    Array(Vec<JsonNode>),
    String(String),
    Number(f64),
    Bool(bool),
    Null,
}

impl JsonNode {
    pub fn parse(input: &str) -> Result<JsonNode, ParseError> {
        let mut parser = Parser::new(input);
        let node = parser.parse_value()?;
        parser.skip_ws();
        if parser.pos < parser.input.len() {
            return Err(parser.error("unexpected trailing content"));
        }
        Ok(node)
    }

    pub fn get(&self, key: &str) -> Option<&JsonNode> {
        match self {
            JsonNode::Object(map) => map.get(key),
            _ => None,
        }
    }

    pub fn get_index(&self, i: usize) -> Option<&JsonNode> {
        match self {
            JsonNode::Array(arr) => arr.get(i),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self { JsonNode::String(s) => Some(s), _ => None }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self { JsonNode::Number(n) => Some(*n), _ => None }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            JsonNode::Number(n) => {
                let i = *n as i64;
                if (i as f64) == *n { Some(i) } else { None }
            }
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self { JsonNode::Bool(b) => Some(*b), _ => None }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, JsonNode::Null)
    }

    pub fn is_object(&self) -> bool {
        matches!(self, JsonNode::Object(_))
    }

    pub fn is_array(&self) -> bool {
        matches!(self, JsonNode::Array(_))
    }

    pub fn as_object(&self) -> Option<&HashMap<String, JsonNode>> {
        match self { JsonNode::Object(m) => Some(m), _ => None }
    }

    pub fn as_array(&self) -> Option<&Vec<JsonNode>> {
        match self { JsonNode::Array(a) => Some(a), _ => None }
    }
}

impl std::ops::Index<&str> for JsonNode {
    type Output = JsonNode;
    fn index(&self, key: &str) -> &JsonNode {
        static NULL: JsonNode = JsonNode::Null;
        self.get(key).unwrap_or(&NULL)
    }
}

impl std::ops::Index<usize> for JsonNode {
    type Output = JsonNode;
    fn index(&self, i: usize) -> &JsonNode {
        static NULL: JsonNode = JsonNode::Null;
        self.get_index(i).unwrap_or(&NULL)
    }
}

// ─── Display (JSON stringify) ────────────────────────────────────────────────

impl fmt::Display for JsonNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JsonNode::Null => write!(f, "null"),
            JsonNode::Bool(b) => write!(f, "{}", if *b { "true" } else { "false" }),
            JsonNode::Number(n) => {
                if *n == (*n as i64) as f64 && n.is_finite() {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{}", n)
                }
            }
            JsonNode::String(s) => {
                write!(f, "\"")?;
                for c in s.chars() {
                    match c {
                        '"' => write!(f, "\\\"")?,
                        '\\' => write!(f, "\\\\")?,
                        '\n' => write!(f, "\\n")?,
                        '\r' => write!(f, "\\r")?,
                        '\t' => write!(f, "\\t")?,
                        c if (c as u32) < 0x20 => write!(f, "\\u{:04x}", c as u32)?,
                        c => write!(f, "{}", c)?,
                    }
                }
                write!(f, "\"")
            }
            JsonNode::Array(arr) => {
                write!(f, "[")?;
                for (i, v) in arr.iter().enumerate() {
                    if i > 0 { write!(f, ",")?; }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
            JsonNode::Object(map) => {
                write!(f, "{{")?;
                let mut keys: Vec<&String> = map.keys().collect();
                keys.sort();
                for (i, k) in keys.iter().enumerate() {
                    if i > 0 { write!(f, ",")?; }
                    write!(f, "\"{}\":{}", k, map[*k])?;
                }
                write!(f, "}}")
            }
        }
    }
}

// ─── Error ───────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct ParseError {
    pub msg: String,
    pub pos: usize,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "JSON parse error at {}: {}", self.pos, self.msg)
    }
}

impl std::error::Error for ParseError {}

// ─── Parser ──────────────────────────────────────────────────────────────────

struct Parser<'a> {
    input: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input: input.as_bytes(), pos: 0 }
    }

    fn error(&self, msg: &str) -> ParseError {
        ParseError { msg: msg.to_string(), pos: self.pos }
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<u8> {
        let b = self.input.get(self.pos).copied()?;
        self.pos += 1;
        Some(b)
    }

    fn expect(&mut self, expected: u8) -> Result<(), ParseError> {
        match self.advance() {
            Some(b) if b == expected => Ok(()),
            Some(b) => Err(self.error(&format!("expected '{}', got '{}'", expected as char, b as char))),
            None => Err(self.error(&format!("expected '{}', got EOF", expected as char))),
        }
    }

    fn skip_ws(&mut self) {
        while let Some(b) = self.peek() {
            match b {
                b' ' | b'\t' | b'\n' | b'\r' => { self.pos += 1; }
                _ => break,
            }
        }
    }

    fn parse_value(&mut self) -> Result<JsonNode, ParseError> {
        self.skip_ws();
        match self.peek() {
            Some(b'"') => self.parse_string().map(JsonNode::String),
            Some(b'{') => self.parse_object(),
            Some(b'[') => self.parse_array(),
            Some(b't') => self.parse_literal("true", JsonNode::Bool(true)),
            Some(b'f') => self.parse_literal("false", JsonNode::Bool(false)),
            Some(b'n') => self.parse_literal("null", JsonNode::Null),
            Some(b'-') | Some(b'0'..=b'9') => self.parse_number(),
            Some(b) => Err(self.error(&format!("unexpected character '{}'", b as char))),
            None => Err(self.error("unexpected EOF")),
        }
    }

    fn parse_string(&mut self) -> Result<String, ParseError> {
        self.expect(b'"')?;
        let mut s = String::new();
        loop {
            match self.advance() {
                Some(b'"') => return Ok(s),
                Some(b'\\') => {
                    match self.advance() {
                        Some(b'"') => s.push('"'),
                        Some(b'\\') => s.push('\\'),
                        Some(b'/') => s.push('/'),
                        Some(b'n') => s.push('\n'),
                        Some(b'r') => s.push('\r'),
                        Some(b't') => s.push('\t'),
                        Some(b'b') => s.push('\u{0008}'),
                        Some(b'f') => s.push('\u{000C}'),
                        Some(b'u') => {
                            let cp = self.parse_hex4()?;
                            // Handle surrogate pairs
                            if (0xD800..=0xDBFF).contains(&cp) {
                                self.expect(b'\\')?;
                                self.expect(b'u')?;
                                let lo = self.parse_hex4()?;
                                if !(0xDC00..=0xDFFF).contains(&lo) {
                                    return Err(self.error("invalid surrogate pair"));
                                }
                                let full = 0x10000 + ((cp - 0xD800) << 10) + (lo - 0xDC00);
                                s.push(char::from_u32(full).ok_or_else(|| self.error("invalid codepoint"))?);
                            } else {
                                s.push(char::from_u32(cp).ok_or_else(|| self.error("invalid codepoint"))?);
                            }
                        }
                        Some(b) => return Err(self.error(&format!("invalid escape '\\{}'", b as char))),
                        None => return Err(self.error("unexpected EOF in string")),
                    }
                }
                Some(b) if b < 0x20 => return Err(self.error("control character in string")),
                Some(b) => {
                    // Handle UTF-8 multi-byte
                    if b < 0x80 {
                        s.push(b as char);
                    } else {
                        self.pos -= 1;
                        let start = self.pos;
                        // Find UTF-8 char boundary
                        let rest = std::str::from_utf8(&self.input[start..])
                            .map_err(|_| self.error("invalid UTF-8"))?;
                        let c = rest.chars().next().ok_or_else(|| self.error("invalid UTF-8"))?;
                        self.pos += c.len_utf8();
                        s.push(c);
                    }
                }
                None => return Err(self.error("unterminated string")),
            }
        }
    }

    fn parse_hex4(&mut self) -> Result<u32, ParseError> {
        let mut n: u32 = 0;
        for _ in 0..4 {
            let b = self.advance().ok_or_else(|| self.error("unexpected EOF in \\u escape"))?;
            let digit = match b {
                b'0'..=b'9' => b - b'0',
                b'a'..=b'f' => b - b'a' + 10,
                b'A'..=b'F' => b - b'A' + 10,
                _ => return Err(self.error("invalid hex digit")),
            };
            n = (n << 4) | digit as u32;
        }
        Ok(n)
    }

    fn parse_number(&mut self) -> Result<JsonNode, ParseError> {
        let start = self.pos;
        // optional minus
        if self.peek() == Some(b'-') { self.pos += 1; }
        // integer part
        match self.peek() {
            Some(b'0') => { self.pos += 1; }
            Some(b'1'..=b'9') => {
                self.pos += 1;
                while let Some(b'0'..=b'9') = self.peek() { self.pos += 1; }
            }
            _ => return Err(self.error("invalid number")),
        }
        // fraction
        if self.peek() == Some(b'.') {
            self.pos += 1;
            let frac_start = self.pos;
            while let Some(b'0'..=b'9') = self.peek() { self.pos += 1; }
            if self.pos == frac_start {
                return Err(self.error("invalid number: no digits after '.'"));
            }
        }
        // exponent
        if let Some(b'e') | Some(b'E') = self.peek() {
            self.pos += 1;
            if let Some(b'+') | Some(b'-') = self.peek() { self.pos += 1; }
            let exp_start = self.pos;
            while let Some(b'0'..=b'9') = self.peek() { self.pos += 1; }
            if self.pos == exp_start {
                return Err(self.error("invalid number: no digits in exponent"));
            }
        }
        let num_str = std::str::from_utf8(&self.input[start..self.pos])
            .map_err(|_| self.error("invalid UTF-8 in number"))?;
        let n: f64 = num_str.parse()
            .map_err(|_| self.error("invalid number"))?;
        Ok(JsonNode::Number(n))
    }

    fn parse_object(&mut self) -> Result<JsonNode, ParseError> {
        self.expect(b'{')?;
        self.skip_ws();
        let mut map = HashMap::new();
        if self.peek() == Some(b'}') {
            self.pos += 1;
            return Ok(JsonNode::Object(map));
        }
        loop {
            self.skip_ws();
            let key = self.parse_string()?;
            self.skip_ws();
            self.expect(b':')?;
            let val = self.parse_value()?;
            map.insert(key, val);
            self.skip_ws();
            match self.peek() {
                Some(b',') => { self.pos += 1; }
                Some(b'}') => { self.pos += 1; return Ok(JsonNode::Object(map)); }
                _ => return Err(self.error("expected ',' or '}'")),
            }
        }
    }

    fn parse_array(&mut self) -> Result<JsonNode, ParseError> {
        self.expect(b'[')?;
        self.skip_ws();
        let mut arr = Vec::new();
        if self.peek() == Some(b']') {
            self.pos += 1;
            return Ok(JsonNode::Array(arr));
        }
        loop {
            arr.push(self.parse_value()?);
            self.skip_ws();
            match self.peek() {
                Some(b',') => { self.pos += 1; }
                Some(b']') => { self.pos += 1; return Ok(JsonNode::Array(arr)); }
                _ => return Err(self.error("expected ',' or ']'")),
            }
        }
    }

    fn parse_literal(&mut self, expected: &str, node: JsonNode) -> Result<JsonNode, ParseError> {
        for b in expected.bytes() {
            self.expect(b)?;
        }
        Ok(node)
    }
}

// ─── From file ───────────────────────────────────────────────────────────────

impl JsonNode {
    pub fn from_file(path: &str) -> Result<JsonNode, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        Ok(JsonNode::parse(&content)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_null() {
        assert_eq!(JsonNode::parse("null").unwrap(), JsonNode::Null);
    }

    #[test]
    fn parse_bool() {
        assert_eq!(JsonNode::parse("true").unwrap(), JsonNode::Bool(true));
        assert_eq!(JsonNode::parse("false").unwrap(), JsonNode::Bool(false));
    }

    #[test]
    fn parse_number() {
        assert_eq!(JsonNode::parse("42").unwrap().as_i64(), Some(42));
        assert_eq!(JsonNode::parse("-3.14").unwrap().as_f64(), Some(-3.14));
        assert_eq!(JsonNode::parse("1e10").unwrap().as_f64(), Some(1e10));
    }

    #[test]
    fn parse_string() {
        assert_eq!(JsonNode::parse(r#""hello""#).unwrap().as_str(), Some("hello"));
        assert_eq!(JsonNode::parse(r#""a\nb""#).unwrap().as_str(), Some("a\nb"));
        assert_eq!(JsonNode::parse(r#""a\\b""#).unwrap().as_str(), Some("a\\b"));
        assert_eq!(JsonNode::parse(r#""\u0041""#).unwrap().as_str(), Some("A"));
    }

    #[test]
    fn parse_surrogate_pair() {
        // 𝄞 = U+1D11E = surrogate pair D834 DD1E
        assert_eq!(JsonNode::parse(r#""\uD834\uDD1E""#).unwrap().as_str(), Some("𝄞"));
    }

    #[test]
    fn parse_array() {
        let node = JsonNode::parse("[1, 2, 3]").unwrap();
        let arr = node.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].as_i64(), Some(1));
    }

    #[test]
    fn parse_object() {
        let node = JsonNode::parse(r#"{"name": "Alice", "age": 30}"#).unwrap();
        assert_eq!(node["name"].as_str(), Some("Alice"));
        assert_eq!(node["age"].as_i64(), Some(30));
    }

    #[test]
    fn parse_nested() {
        let node = JsonNode::parse(r#"{"user": {"address": {"city": "NY"}}}"#).unwrap();
        assert_eq!(node["user"]["address"]["city"].as_str(), Some("NY"));
    }

    #[test]
    fn parse_empty() {
        assert!(JsonNode::parse("{}").unwrap().is_object());
        assert!(JsonNode::parse("[]").unwrap().is_array());
    }

    #[test]
    fn stringify_roundtrip() {
        let input = r#"{"a":1,"b":[true,null,"hello"]}"#;
        let node = JsonNode::parse(input).unwrap();
        let output = node.to_string();
        let reparsed = JsonNode::parse(&output).unwrap();
        assert_eq!(node, reparsed);
    }

    #[test]
    fn parse_error() {
        assert!(JsonNode::parse("").is_err());
        assert!(JsonNode::parse("{").is_err());
        assert!(JsonNode::parse("[1,]").is_err());
    }
}
