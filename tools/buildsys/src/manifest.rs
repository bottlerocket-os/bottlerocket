/*!
# Build system metadata

This module provides deserialization and convenience methods for build system
metadata located in `Cargo.toml`.

Cargo ignores the `package.metadata` table in its manifest, so it can be used
to store configuration for other tools. We recognize the following keys.

## Metadata for packages

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

`package-name` lets you override the package name in Cargo.toml; this is useful
if you have a package with "." in its name, for example, which Cargo doesn't
allow.  This means the directory name and spec file name can use your preferred
naming.
```
[package.metadata.build-package]
package-name = "better.name"
```

`variant-sensitive` lets you specify whether the package should be rebuilt when
building a new variant, and defaults to false; set it to true if a package is
using the variant to affect its build process.  (Typically this means that it
reads BUILDSYS_VARIANT.)
```
[package.metadata.build-package]
variant-sensitive = true
```

## Metadata for variants

`included-packages` is a list of packages that should be included in a variant.
```
[package.metadata.build-variant]
included-packages = ["release"]
```
*/

pub(crate) mod error;
use error::Result;

use serde::Deserialize;
use snafu::ResultExt;
use std::collections::HashSet;
use std::fmt;
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

    /// Convenience method to return the package name override, if any.
    pub(crate) fn package_name(&self) -> Option<&String> {
        self.build_package().and_then(|b| b.package_name.as_ref())
    }

    /// Convenience method to find whether the package is sensitive to variant changes.
    pub(crate) fn variant_sensitive(&self) -> Option<bool> {
        self.build_package().and_then(|b| b.variant_sensitive)
    }

    /// Convenience method to return the list of included packages.
    pub(crate) fn included_packages(&self) -> Option<&Vec<String>> {
        self.build_variant()
            .and_then(|b| b.included_packages.as_ref())
    }

    /// Convenience method to return the image format override, if any.
    pub(crate) fn image_format(&self) -> Option<&ImageFormat> {
        self.build_variant().and_then(|b| b.image_format.as_ref())
    }

    /// Convenience method to return the supported architectures for this variant.
    pub(crate) fn supported_arches(&self) -> Option<&HashSet<SupportedArch>> {
        self.build_variant()
            .and_then(|b| b.supported_arches.as_ref())
    }

    /// Helper methods to navigate the series of optional struct fields.
    fn build_package(&self) -> Option<&BuildPackage> {
        self.package
            .metadata
            .as_ref()
            .and_then(|m| m.build_package.as_ref())
    }

    fn build_variant(&self) -> Option<&BuildVariant> {
        self.package
            .metadata
            .as_ref()
            .and_then(|m| m.build_variant.as_ref())
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
    build_variant: Option<BuildVariant>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct BuildPackage {
    pub(crate) external_files: Option<Vec<ExternalFile>>,
    pub(crate) package_name: Option<String>,
    pub(crate) source_groups: Option<Vec<PathBuf>>,
    pub(crate) variant_sensitive: Option<bool>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct BuildVariant {
    pub(crate) included_packages: Option<Vec<String>>,
    pub(crate) image_format: Option<ImageFormat>,
    pub(crate) supported_arches: Option<HashSet<SupportedArch>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ImageFormat {
    Qcow2,
    Raw,
    Vmdk,
}

#[derive(Deserialize, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub(crate) enum SupportedArch {
    X86_64,
    Aarch64,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct ExternalFile {
    pub(crate) path: Option<PathBuf>,
    pub(crate) sha512: String,
    pub(crate) url: String,
}

impl fmt::Display for SupportedArch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SupportedArch::X86_64 => write!(f, "x86_64"),
            SupportedArch::Aarch64 => write!(f, "aarch64"),
        }
    }
}
