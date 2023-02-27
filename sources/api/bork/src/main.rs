use rand::{thread_rng, Rng};

fn main() {
    let mut rng = thread_rng();
    let val = rng.gen_range(0..2048);

    // sundog expects JSON-serialized output so that many types can be represented, allowing the
    // API model to use more accurate types.
    let output = serde_json::to_string(&val).expect("Unable to serialize val '{}' to JSON");

    println!("{}", output);
}
