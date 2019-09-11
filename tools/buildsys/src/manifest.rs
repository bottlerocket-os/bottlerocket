/*!
This module provides deserialization and convenience methods for build system
metadata located in `Cargo.toml`.

Cargo ignores the `package.metadata` table in its manifest, so it can be used
to store configuration for other tools. We recognize the following keys.

`source-groups` is a list of directories in the top-level `sources` directory,
each of which contains a set of related Rust projects. Changes to files in
these groups should trigger a rebuild.
```
[package.metadata.build-package]
source-groups = ["api"]
```

`external-files` is a list of out-of-tree files that should be retrieved
as additional dependencies for the build. If the path for the external
file name is not provided, it will be taken from the last path component
of the URL.
```
[[package.metadata.build-package.external-files]]
path = "foo"
url = "https://foo"
sha512 = "abcdef"

[[package.metadata.build-package.external-files]]
path = "bar"
url = "https://bar"
sha512 = "123456"
```

`included-packages` is a list of packages that should be included in an image.
```
[package.metadata.build-image]
included-packages = ["release"]
```
*/

pub(crate) mod error;
use error::Result;

use serde::Deserialize;
use snafu::ResultExt;
use std::fs;
use std::path::{Path, PathBuf};

/// The nested structures here are somewhat complex, but they make it trivial
/// to deserialize the structure we expect to find in the manifest.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct ManifestInfo {
    package: Package,
}

impl ManifestInfo {
    /// Extract the settings we understand from `Cargo.toml`.
    pub(crate) fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let manifest_data = fs::read_to_string(path).context(error::ManifestFileRead { path })?;
        toml::from_str(&manifest_data).context(error::ManifestFileLoad { path })
    }

    /// Convenience method to return the list of source groups.
    pub(crate) fn source_groups(&self) -> Option<&Vec<PathBuf>> {
        self.build_package().and_then(|b| b.source_groups.as_ref())
    }

    /// Convenience method to return the list of external files.
    pub(crate) fn external_files(&self) -> Option<&Vec<ExternalFile>> {
        self.build_package().and_then(|b| b.external_files.as_ref())
    }

    /// Convenience method to return the list of included packages.
    pub(crate) fn included_packages(&self) -> Option<&Vec<String>> {
        self.build_image()
            .and_then(|b| b.included_packages.as_ref())
    }

    /// Helper methods to navigate the series of optional struct fields.
    fn build_package(&self) -> Option<&BuildPackage> {
        self.package
            .metadata
            .as_ref()
            .and_then(|m| m.build_package.as_ref())
    }

    fn build_image(&self) -> Option<&BuildImage> {
        self.package
            .metadata
            .as_ref()
            .and_then(|m| m.build_image.as_ref())
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
struct Package {
    metadata: Option<Metadata>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
struct Metadata {
    build_package: Option<BuildPackage>,
    build_image: Option<BuildImage>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct BuildPackage {
    pub(crate) source_groups: Option<Vec<PathBuf>>,
    pub(crate) external_files: Option<Vec<ExternalFile>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct BuildImage {
    pub(crate) included_packages: Option<Vec<String>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct ExternalFile {
    pub(crate) path: Option<PathBuf>,
    pub(crate) sha512: String,
    pub(crate) url: String,
}
