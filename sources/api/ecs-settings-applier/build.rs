// Automatically generate README.md from rustdoc.

use std::env;

fn main() {
    println!("cargo:rerun-if-env-changed=VARIANT");
    if let Ok(variant) = env::var("VARIANT") {
        println!("cargo:rustc-cfg=variant=\"{}\"", variant);
        let parts = variant.split('-').collect::<Vec<&str>>();
        println!(
            "cargo:rustc-cfg=variant_family=\"{}\"",
            parts[0..2].join("-")
        );
        let variant_type = if parts.len() > 3 {
            parts[3]
        } else {
            "general_purpose"
        };
        println!("cargo:rustc-cfg=variant_type=\"{}\"", variant_type);
    }

    generate_readme::from_file("src/ecs.rs").unwrap();
}
