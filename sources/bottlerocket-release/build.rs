use std::path::Path;
use std::{env, fs};

fn generate_readme() {
    generate_readme::from_lib().unwrap();
}

fn generate_constants() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let contents = format!("const ARCH: &str = \"{}\";", arch);
    let path = Path::new(&out_dir).join("constants.rs");
    fs::write(path, contents).unwrap();
}

fn main() {
    generate_readme();
    generate_constants();
}
