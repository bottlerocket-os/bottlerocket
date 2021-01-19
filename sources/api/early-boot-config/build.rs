// Automatically generate README.md from rustdoc.

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process;

fn main() {
    // The code below emits `cfg` operators to conditionally compile this program based on the
    // current variant.  This approach is only meant to be in use for the short term; when the
    // build system supports ideas like "variant families" we should be able to drive the
    // conditional compilation in a less brittle way.
    println!("cargo:rerun-if-env-changed=VARIANT");
    if let Ok(variant) = env::var("VARIANT") {
        // The aws-dev variant includes the ability to read user data from local file
        if variant == "aws-dev" {
            println!("cargo:rustc-cfg=bottlerocket_platform=\"aws-dev\"");
        } else if variant.starts_with("aws") {
            println!("cargo:rustc-cfg=bottlerocket_platform=\"aws\"");
        } else {
            eprintln!(
            "For local builds, you must set the 'VARIANT' environment variable so we know which data \
            provider to build. Valid values are the directories in models/src/variants/, for \
            example 'aws-k8s-1.17'."
            );
            process::exit(1);
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
