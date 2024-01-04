fn main() {
    let val = settings_extension_updates::generate::generate_seed();

    // sundog expects JSON-serialized output so that many types can be represented, allowing the
    // API model to use more accurate types.
    let output = serde_json::to_string(&val).expect("Unable to serialize val '{}' to JSON");

    println!("{}", output);
}
