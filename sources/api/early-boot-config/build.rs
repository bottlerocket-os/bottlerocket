// Automatically generate README.md from rustdoc.

use buildsys::{Variant, VARIANT_ENV};
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    let variant = match Variant::from_env() {
        Ok(variant) => variant,
        Err(e) => {
            eprintln!(
                "For local builds, you must set the '{}' environment variable so we know \
                which data provider to build. Valid values are the directories in \
                models/src/variants/, for example 'aws-ecs-1': {}",
                VARIANT_ENV, e,
            );
            std::process::exit(1);
        }
    };
    variant.emit_cfgs();

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
