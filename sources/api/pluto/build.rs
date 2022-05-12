// Automatically generate README.md from rustdoc.

use buildsys::{generate_readme, ReadmeSource, Variant};

fn main() {
    let variant = Variant::from_env().unwrap();
    variant.emit_cfgs();
    generate_readme(ReadmeSource::Main).unwrap()
}
