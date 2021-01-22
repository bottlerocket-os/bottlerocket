/*!
This tool carries out a package or variant build using Docker.

It is meant to be called by a Cargo build script. To keep those scripts simple,
all of the configuration is taken from the environment, with the build type
specified as a command line argument.

The implementation is closely tied to the top-level Dockerfile.

*/
mod builder;
mod cache;
mod manifest;
mod project;
mod spec;

use builder::{PackageBuilder, VariantBuilder};
use cache::LookasideCache;
use manifest::ManifestInfo;
use project::ProjectInfo;
use serde::Deserialize;
use snafu::ResultExt;
use spec::SpecInfo;
use std::env;
use std::path::PathBuf;
use std::process;

mod error {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility = "pub(super)")]
    pub(super) enum Error {
        ManifestParse {
            source: super::manifest::error::Error,
        },

        SpecParse {
            source: super::spec::error::Error,
        },

        ExternalFileFetch {
            source: super::cache::error::Error,
        },

        ProjectCrawl {
            source: super::project::error::Error,
        },

        BuildAttempt {
            source: super::builder::error::Error,
        },

        #[snafu(display("Missing environment variable '{}'", var))]
        Environment {
            var: String,
            source: std::env::VarError,
        },
    }
}

type Result<T> = std::result::Result<T, error::Error>;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum Command {
    BuildPackage,
    BuildVariant,
}

fn usage() -> ! {
    eprintln!(
        "\
USAGE:
    buildsys <SUBCOMMAND>

SUBCOMMANDS:
    build-package           Build RPMs from a spec file and sources.
    build-variant           Build filesystem and disk images from RPMs."
    );
    process::exit(1)
}

// Returning a Result from main makes it print a Debug representation of the error, but with Snafu
// we have nice Display representations of the error, so we wrap "main" (run) and print any error.
// https://github.com/shepmaster/snafu/issues/110
fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let command_str = std::env::args().nth(1).unwrap_or_else(|| usage());
    let command = serde_plain::from_str::<Command>(&command_str).unwrap_or_else(|_| usage());
    match command {
        Command::BuildPackage => build_package()?,
        Command::BuildVariant => build_variant()?,
    }
    Ok(())
}

fn build_package() -> Result<()> {
    let manifest_dir: PathBuf = getenv("CARGO_MANIFEST_DIR")?.into();
    let manifest_file = "Cargo.toml";
    println!("cargo:rerun-if-changed={}", manifest_file);

    let manifest =
        ManifestInfo::new(manifest_dir.join(manifest_file)).context(error::ManifestParse)?;

    // if manifest has package.metadata.build-package.variant-specific = true, then println rerun-if-env-changed
    if let Some(sensitive) = manifest.variant_sensitive() {
        if sensitive {
            println!("cargo:rerun-if-env-changed=BUILDSYS_VARIANT");
        }
    }

    if let Some(files) = manifest.external_files() {
        LookasideCache::fetch(&files).context(error::ExternalFileFetch)?;
    }

    if let Some(groups) = manifest.source_groups() {
        let var = "BUILDSYS_SOURCES_DIR";
        let root: PathBuf = getenv(var)?.into();
        println!("cargo:rerun-if-env-changed={}", var);

        let dirs = groups.iter().map(|d| root.join(d)).collect::<Vec<_>>();
        let info = ProjectInfo::crawl(&dirs).context(error::ProjectCrawl)?;
        for f in info.files {
            println!("cargo:rerun-if-changed={}", f.display());
        }
    }

    // Package developer can override name of package if desired, e.g. to name package with
    // characters invalid in Cargo crate names
    let package = if let Some(name_override) = manifest.package_name() {
        name_override.clone()
    } else {
        getenv("CARGO_PKG_NAME")?
    };
    let spec = format!("{}.spec", package);
    println!("cargo:rerun-if-changed={}", spec);

    let info = SpecInfo::new(PathBuf::from(&spec)).context(error::SpecParse)?;

    for f in info.sources {
        println!("cargo:rerun-if-changed={}", f.display());
    }

    for f in info.patches {
        println!("cargo:rerun-if-changed={}", f.display());
    }

    PackageBuilder::build(&package).context(error::BuildAttempt)?;

    Ok(())
}

fn build_variant() -> Result<()> {
    let manifest_dir: PathBuf = getenv("CARGO_MANIFEST_DIR")?.into();
    let manifest_file = "Cargo.toml";
    println!("cargo:rerun-if-changed={}", manifest_file);

    let manifest =
        ManifestInfo::new(manifest_dir.join(manifest_file)).context(error::ManifestParse)?;

    if let Some(packages) = manifest.included_packages() {
        let image_format = manifest.image_format();
        VariantBuilder::build(&packages, image_format).context(error::BuildAttempt)?;
    } else {
        println!("cargo:warning=No included packages in manifest. Skipping variant build.");
    }

    Ok(())
}

/// Retrieve a variable that we expect to be set in the environment.
fn getenv(var: &str) -> Result<String> {
    env::var(var).context(error::Environment { var })
}
