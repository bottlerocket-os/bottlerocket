// Automatically generate README.md from rustdoc.

use buildsys::{generate_readme, ReadmeSource};
use std::env;
use std::fs;
use std::path::Path;

fn generate_constants() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let contents = format!("const ARCH: &str = \"{}\";", arch);
    let path = Path::new(&out_dir).join("constants.rs");
    fs::write(path, contents).unwrap();
}

fn main() {
    generate_readme(ReadmeSource::Lib).unwrap();
    generate_constants();
}
