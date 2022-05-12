// Automatically generate README.md from rustdoc.

use buildsys::{generate_readme, ReadmeSource};

fn main() {
    generate_readme(ReadmeSource::Lib).unwrap()
}
