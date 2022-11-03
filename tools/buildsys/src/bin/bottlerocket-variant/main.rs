use bottlerocket_variant::Variant;
use buildsys::manifest::ManifestInfo;
use snafu::ResultExt;
use std::path::PathBuf;
use std::{env, process};

// Returning a Result from main makes it print a Debug representation of the error, but with Snafu
// we have nice Display representations of the error, so we wrap "main" (run) and print any error.
// https://github.com/shepmaster/snafu/issues/110
fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        process::exit(1);
    }
}

/// Read `BUILDSYS_VARIANT` from the environment, parse into its components, and emit related
/// environment variables to set (or export). Do the same for features defined in the variant
/// manifest.
fn run() -> Result<()> {
    let env = getenv("BUILDSYS_VARIANT")?;
    let variant = Variant::new(&env).context(error::VariantParseSnafu)?;
    println!("BUILDSYS_VARIANT_PLATFORM={}", variant.platform());
    println!("BUILDSYS_VARIANT_RUNTIME={}", variant.runtime());
    println!("BUILDSYS_VARIANT_FAMILY={}", variant.family());
    println!(
        "BUILDSYS_VARIANT_FLAVOR={}",
        variant.variant_flavor().unwrap_or("''")
    );
    let manifest = PathBuf::from(getenv("BUILDSYS_ROOT_DIR")?)
        .join("variants")
        .join(&env)
        .join("Cargo.toml");
    let variant_manifest = ManifestInfo::new(manifest).context(error::ManifestParseSnafu)?;
    if let Some(image_features) = variant_manifest.image_features() {
        for image_feature in image_features {
            println!("export BUILDSYS_VARIANT_IMAGE_FEATURE_{}=1", image_feature);
        }
    }
    Ok(())
}

/// Retrieve a variable that we expect to be set in the environment.
fn getenv(var: &str) -> Result<String> {
    env::var(var).context(error::EnvironmentSnafu { var })
}

mod error {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub(super) enum Error {
        VariantParse {
            source: bottlerocket_variant::error::Error,
        },

        ManifestParse {
            source: buildsys::manifest::Error,
        },

        #[snafu(display("Missing environment variable '{}'", var))]
        Environment {
            var: String,
            source: std::env::VarError,
        },
    }
}

type Result<T> = std::result::Result<T, error::Error>;
