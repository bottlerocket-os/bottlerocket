/*!
This library initiates an rpm or image build by running the BuildKit CLI inside
a Docker container.

It is meant to be called by a Cargo build script. To keep those scripts simple,
all of the configuration is taken from the environment.

*/
mod builder;
mod cache;
mod manifest;
mod project;
mod spec;

use builder::{ImageBuilder, PackageBuilder};
use cache::LookasideCache;
use manifest::ManifestInfo;
use project::ProjectInfo;
use snafu::ResultExt;
use spec::SpecInfo;
use std::env;
use std::path::PathBuf;

pub mod error {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility = "pub")]
    pub enum Error {
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

pub type Result<T> = std::result::Result<T, error::Error>;

pub fn build_package() -> Result<()> {
    let manifest_dir: PathBuf = getenv("CARGO_MANIFEST_DIR")?.into();
    let manifest_file = "Cargo.toml";
    println!("cargo:rerun-if-changed={}", manifest_file);

    let manifest =
        ManifestInfo::new(manifest_dir.join(manifest_file)).context(error::ManifestParse)?;

    if let Some(files) = manifest.external_files() {
        LookasideCache::fetch(&files).context(error::ExternalFileFetch)?;
    }

    // Stop after build has fetched its external files in the context of fetch
    // cargo-make tasks.
    println!("cargo:rerun-if-env-changed=BUILDSYS_BUILD_FETCH_ONLY");
    if let Ok(val) = getenv("BUILDSYS_BUILD_FETCH_ONLY") {
        if val == "true" {
            return Ok(());
        }
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

    let package = getenv("CARGO_PKG_NAME")?;
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

pub fn build_image() -> Result<()> {
    let manifest_dir: PathBuf = getenv("CARGO_MANIFEST_DIR")?.into();
    let manifest_file = "Cargo.toml";
    println!("cargo:rerun-if-changed={}", manifest_file);

    let manifest =
        ManifestInfo::new(manifest_dir.join(manifest_file)).context(error::ManifestParse)?;

    if let Some(packages) = manifest.included_packages() {
        ImageBuilder::build(&packages).context(error::BuildAttempt)?;
    } else {
        println!("cargo:warning=No included packages in manifest. Skipping image build.");
    }

    Ok(())
}

/// Retrieve a variable that we expect to be set in the environment.
fn getenv(var: &str) -> Result<String> {
    env::var(var).context(error::Environment { var })
}
