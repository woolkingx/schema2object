use schema2object::{JsonNode, ObjectTree};

fn main() {
    // Schema is the definition — like a .h file
    let schema = JsonNode::from_file("examples/user.schema.json").unwrap();

    // Data comes from runtime (API input, file read, user input, etc.)
    let data = JsonNode::parse(r#"{
        "name": "Alice",
        "email": "alice@example.com",
        "age": 30,
        "address": { "city": "New York", "zip": "10001" }
    }"#).unwrap();

    let user = ObjectTree::new(data, schema).unwrap();

    println!("name:         {}", user["name"]);
    println!("email:        {}", user["email"]);
    println!("age:          {}", user["age"]);
    println!("address.city: {}", user["address"]["city"]);
    println!("address.zip:  {}", user["address"]["zip"]);
}
