// Automatically generate README.md from rustdoc.

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

const VARIANT_GROUPS: &[&str] = &["aws-k8s", "aws-ecs"];

fn main() {
    println!("cargo:rerun-if-env-changed=VARIANT");
    if let Ok(variant) = env::var("VARIANT") {
        println!("cargo:rustc-cfg=variant=\"{}\"", variant);
        for &variant_group in VARIANT_GROUPS {
            if variant.starts_with(variant_group) {
                println!("cargo:rustc-cfg=variant_group=\"{}\"", variant_group);
                break;
            }
        }
    }

    // Check for environment variable "SKIP_README". If it is set,
    // skip README generation
    if env::var_os("SKIP_README").is_some() {
        return;
    }

    let mut source = File::open("src/main.rs").unwrap();
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
