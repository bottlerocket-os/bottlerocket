// Automatically generate README.md from rustdoc.

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

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

    // Check for environment variable "SKIP_README". If it is set,
    // skip README generation
    if env::var_os("SKIP_README").is_some() {
        return;
    }

    let mut source = File::open("src/ecs.rs").unwrap();
    let mut template = File::open("README.tpl").unwrap();

    let content = cargo_readme::generate_readme(
        &PathBuf::from("."), // root
        &mut source,         // source
        Some(&mut template), // template
        // The "add x" arguments don't apply when using a template.
        true,  // add title
        false, // add badges
        false, // add license
        true,  // indent headings
    )
    .unwrap();

    let mut readme = File::create("README.md").unwrap();
    readme.write_all(content.as_bytes()).unwrap();
}
