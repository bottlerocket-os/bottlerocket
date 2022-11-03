/*!
This tool carries out a package or variant build using Docker.

It is meant to be called by a Cargo build script. To keep those scripts simple,
all of the configuration is taken from the environment, with the build type
specified as a command line argument.

The implementation is closely tied to the top-level Dockerfile.

*/
mod builder;
mod cache;
mod gomod;
mod project;
mod spec;

use builder::{PackageBuilder, VariantBuilder};
use buildsys::manifest::{BundleModule, ManifestInfo, SupportedArch};
use cache::LookasideCache;
use gomod::GoMod;
use project::ProjectInfo;
use serde::Deserialize;
use snafu::{ensure, ResultExt};
use spec::SpecInfo;
use std::env;
use std::path::PathBuf;
use std::process;

mod error {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub(super) enum Error {
        ManifestParse {
            source: buildsys::manifest::Error,
        },

        SpecParse {
            source: super::spec::error::Error,
        },

        ExternalFileFetch {
            source: super::cache::error::Error,
        },

        GoMod {
            source: super::gomod::error::Error,
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

        #[snafu(display("Unknown architecture: '{}'", arch))]
        UnknownArch {
            arch: String,
            source: serde_plain::Error,
        },

        #[snafu(display(
            "Unsupported architecture {}, this variant supports {}",
            arch,
            supported_arches.join(", ")
        ))]
        UnsupportedArch {
            arch: String,
            supported_arches: Vec<String>,
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
    // Not actually redundant for a diverging function.
    #[allow(clippy::redundant_closure)]
    let command_str = std::env::args().nth(1).unwrap_or_else(|| usage());
    let command = serde_plain::from_str::<Command>(&command_str).unwrap_or_else(|_| usage());
    match command {
        Command::BuildPackage => build_package()?,
        Command::BuildVariant => build_variant()?,
    }
    Ok(())
}

fn build_package() -> Result<()> {
    let manifest_file = "Cargo.toml";
    println!("cargo:rerun-if-changed={}", manifest_file);

    let root_dir: PathBuf = getenv("BUILDSYS_ROOT_DIR")?.into();
    let variant = getenv("BUILDSYS_VARIANT")?;
    let variant_manifest_path = root_dir.join("variants").join(variant).join(manifest_file);
    let variant_manifest =
        ManifestInfo::new(variant_manifest_path).context(error::ManifestParseSnafu)?;
    supported_arch(&variant_manifest)?;
    let mut image_features = variant_manifest.image_features();

    let manifest_dir: PathBuf = getenv("CARGO_MANIFEST_DIR")?.into();
    let manifest =
        ManifestInfo::new(manifest_dir.join(manifest_file)).context(error::ManifestParseSnafu)?;
    let package_features = manifest.package_features();

    // For any package feature specified in the package manifest, track the corresponding
    // environment variable for changes to the ambient set of image features for the current
    // variant.
    if let Some(package_features) = &package_features {
        for package_feature in package_features {
            println!(
                "cargo:rerun-if-env-changed=BUILDSYS_VARIANT_IMAGE_FEATURE_{}",
                package_feature
            );
        }
    }

    // Keep only the image features that the package has indicated that it tracks, if any.
    if let Some(image_features) = &mut image_features {
        match package_features {
            Some(package_features) => image_features.retain(|k| package_features.contains(k)),
            None => image_features.clear(),
        }
    }

    // If manifest has package.metadata.build-package.variant-sensitive set, then track the
    // appropriate environment variable for changes.
    if let Some(sensitivity) = manifest.variant_sensitive() {
        use buildsys::manifest::{SensitivityType::*, VariantSensitivity::*};
        fn emit_variant_env(suffix: Option<&str>) {
            if let Some(suffix) = suffix {
                println!(
                    "cargo:rerun-if-env-changed=BUILDSYS_VARIANT_{}",
                    suffix.to_uppercase()
                );
            } else {
                println!("cargo:rerun-if-env-changed=BUILDSYS_VARIANT");
            }
        }
        match sensitivity {
            Any(false) => (),
            Any(true) => emit_variant_env(None),
            Specific(Platform) => emit_variant_env(Some("platform")),
            Specific(Runtime) => emit_variant_env(Some("runtime")),
            Specific(Family) => emit_variant_env(Some("family")),
            Specific(Flavor) => emit_variant_env(Some("flavor")),
        }
    }

    if let Some(files) = manifest.external_files() {
        LookasideCache::fetch(files).context(error::ExternalFileFetchSnafu)?;
        for f in files {
            if f.bundle_modules.is_none() {
                continue;
            }

            for b in f.bundle_modules.as_ref().unwrap() {
                match b {
                    BundleModule::Go => {
                        GoMod::vendor(&root_dir, &manifest_dir, f).context(error::GoModSnafu)?
                    }
                }
            }
        }
    }

    if let Some(groups) = manifest.source_groups() {
        let var = "BUILDSYS_SOURCES_DIR";
        let root: PathBuf = getenv(var)?.into();
        println!("cargo:rerun-if-env-changed={}", var);

        let dirs = groups.iter().map(|d| root.join(d)).collect::<Vec<_>>();
        let info = ProjectInfo::crawl(&dirs).context(error::ProjectCrawlSnafu)?;
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

    let info = SpecInfo::new(PathBuf::from(&spec)).context(error::SpecParseSnafu)?;

    for f in info.sources {
        println!("cargo:rerun-if-changed={}", f.display());
    }

    for f in info.patches {
        println!("cargo:rerun-if-changed={}", f.display());
    }

    PackageBuilder::build(&package, image_features).context(error::BuildAttemptSnafu)?;

    Ok(())
}

fn build_variant() -> Result<()> {
    let manifest_dir: PathBuf = getenv("CARGO_MANIFEST_DIR")?.into();
    let manifest_file = "Cargo.toml";
    println!("cargo:rerun-if-changed={}", manifest_file);

    let manifest =
        ManifestInfo::new(manifest_dir.join(manifest_file)).context(error::ManifestParseSnafu)?;

    supported_arch(&manifest)?;

    if let Some(packages) = manifest.included_packages() {
        let image_format = manifest.image_format();
        let image_layout = manifest.image_layout();
        let kernel_parameters = manifest.kernel_parameters();
        let image_features = manifest.image_features();
        VariantBuilder::build(
            packages,
            image_format,
            image_layout,
            kernel_parameters,
            image_features,
        )
        .context(error::BuildAttemptSnafu)?;
    } else {
        println!("cargo:warning=No included packages in manifest. Skipping variant build.");
    }

    Ok(())
}

/// Ensure that the current arch is supported by the current variant
fn supported_arch(manifest: &ManifestInfo) -> Result<()> {
    if let Some(supported_arches) = manifest.supported_arches() {
        let arch = getenv("BUILDSYS_ARCH")?;
        let current_arch: SupportedArch =
            serde_plain::from_str(&arch).context(error::UnknownArchSnafu { arch: &arch })?;

        ensure!(
            supported_arches.contains(&current_arch),
            error::UnsupportedArchSnafu {
                arch: &arch,
                supported_arches: supported_arches
                    .iter()
                    .map(|a| a.to_string())
                    .collect::<Vec<String>>()
            }
        )
    }
    Ok(())
}

/// Retrieve a variable that we expect to be set in the environment.
fn getenv(var: &str) -> Result<String> {
    env::var(var).context(error::EnvironmentSnafu { var })
}
