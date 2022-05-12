// Automatically generate README.md from rustdoc.

use buildsys::{generate_readme, ReadmeSource, Variant, VARIANT_ENV};

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

    generate_readme(ReadmeSource::Main).unwrap()
}
