use schema2object::schema;

#[schema("examples/user.schema.json")]
struct User;

fn main() {
    // Construction IS validation
    let addr = UserAddress::new(
        "New York".to_string(),
        Some("10001".to_string()),
    ).unwrap();

    let user = User::new(
        "Alice".to_string(),
        Some(30),
        "alice@example.com".to_string(),
        Some(addr),
    ).unwrap();

    // Dot access — typed, compile-time safe
    println!("name:         {}", user.name);
    println!("email:        {}", user.email);
    println!("age:          {}", user.age);

    // to_json() — JSON string output
    println!("\nJSON:\n{}", user.to_json());

    // schema() — raw schema string
    println!("\nSchema available: {} bytes", User::schema().len());

    // to_dict() — HashMap
    println!("\nDict: {:?}", user.to_dict());

    // validate() — gate check (same as new, returns Result)
    let result = User::validate(
        "Bob".to_string(),
        Some(-1),  // minimum: 0
        "bob@example.com".to_string(),
        None,
    );
    println!("\nValidation error: {}", result.unwrap_err());

    // get_extensions()
    println!("Extensions: {:?}", User::get_extensions());
}
