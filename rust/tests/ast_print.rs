use schema2object::{JsonNode, ObjectTree};

#[test]
fn build_and_access() {
    let schema = JsonNode::parse(r#"{
        "type": "object",
        "properties": {
            "name": { "type": "string" },
            "age": { "type": "integer", "minimum": 0, "default": 0 },
            "address": {
                "type": "object",
                "properties": {
                    "city": { "type": "string" },
                    "zip": { "type": "string" }
                },
                "required": ["city"]
            }
        },
        "required": ["name"]
    }"#).unwrap();

    let data = JsonNode::parse(r#"{
        "name": "Alice",
        "age": 30,
        "address": { "city": "New York", "zip": "10001" }
    }"#).unwrap();

    let tree = ObjectTree::new(data, schema).unwrap();

    // dot access via []
    println!("name         = {}", tree["name"]);
    println!("age          = {}", tree["age"]);
    println!("address.city = {}", tree["address"]["city"]);
    println!("address.zip  = {}", tree["address"]["zip"]);

    // typed access
    assert_eq!(tree["name"].as_str(), Some("Alice"));
    assert_eq!(tree["age"].as_i64(), Some(30));
    assert_eq!(tree["address"]["city"].as_str(), Some("New York"));
}

#[test]
fn defaults_applied() {
    let schema = JsonNode::parse(r#"{
        "type": "object",
        "properties": {
            "name": { "type": "string" },
            "role": { "type": "string", "default": "user" }
        }
    }"#).unwrap();

    let data = JsonNode::parse(r#"{ "name": "Bob" }"#).unwrap();
    let tree = ObjectTree::new(data, schema).unwrap();

    println!("name = {}", tree["name"]);
    println!("role = {}", tree["role"]);
    assert_eq!(tree["role"].as_str(), Some("user"));
}

#[test]
fn type_error() {
    let schema = JsonNode::parse(r#"{
        "type": "object",
        "properties": {
            "age": { "type": "integer" }
        }
    }"#).unwrap();

    let data = JsonNode::parse(r#"{ "age": "not a number" }"#).unwrap();
    let err = ObjectTree::new(data, schema).unwrap_err();
    println!("error: {}", err);
    assert!(err.msg.contains("expected integer"));
}

#[test]
fn constraint_error() {
    let schema = JsonNode::parse(r#"{
        "type": "object",
        "properties": {
            "age": { "type": "integer", "minimum": 0 }
        }
    }"#).unwrap();

    let data = JsonNode::parse(r#"{ "age": -1 }"#).unwrap();
    let err = ObjectTree::new(data, schema).unwrap_err();
    println!("error: {}", err);
    assert!(err.msg.contains(">= 0"));
}

#[test]
fn required_error() {
    let schema = JsonNode::parse(r#"{
        "type": "object",
        "properties": {
            "name": { "type": "string" }
        },
        "required": ["name"]
    }"#).unwrap();

    let data = JsonNode::parse(r#"{}"#).unwrap();
    let err = ObjectTree::new(data, schema).unwrap_err();
    println!("error: {}", err);
    assert!(err.msg.contains("name"));
}

#[test]
fn to_value_roundtrip() {
    let schema = JsonNode::parse(r#"{
        "type": "object",
        "properties": {
            "name": { "type": "string" },
            "age": { "type": "integer", "default": 0 }
        }
    }"#).unwrap();

    let data = JsonNode::parse(r#"{ "name": "Alice" }"#).unwrap();
    let tree = ObjectTree::new(data, schema).unwrap();
    let value = tree.to_value();
    println!("value = {}", value);
    assert_eq!(value["name"].as_str(), Some("Alice"));
    assert_eq!(value["age"].as_i64(), Some(0));
}
