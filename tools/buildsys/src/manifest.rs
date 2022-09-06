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

The `bundle-*` keys on `external-files` are a group of optional modifiers
and are used to untar an upstream external file archive, vendor any dependent
code, and produce an additional archive with those dependencies.
Only `bundle-modules` is required when bundling an archive's dependences.

`bundle-modules` is a list of module "paradigms" the external-file should
be vendored through. For example, if a project contains a `go.mod` and `go.sum`
file, adding "go" to the list will vendor the dependencies through go modules.
Currently, only "go" is supported.

`bundle-root-path` is an optional argument that provides the filepath
within the archive that contains the module. By default, the first top level
directory in the archive is used. So, for example, given a Go project that has
the necessary `go.mod` and `go.sum` files in the archive located at the
filepath `a/b/c`, this `bundle-root-path` value should be "a/b/c". Or, given an
archive with a single directory that contains a Go project that has `go.mod`
and `go.sum` files located in that top level directory, this option may be
omitted since the single top-level directory will authomatically be used.

`bundle-output-path` is an optional argument that provides the desired path of
the output archive. By default, this will use the name of the existing archive,
but pre-pended with "bundled-". For example, if "my-unique-archive-name.tar.gz"
is entered as the value for `bundle-output-path`, then the output directory
will be named `my-unique-archive-name.tar.gz`. Or, by default, given the name
of some upstream archive is "my-package.tar.gz", the output archive would be
named `bundled-my-package.tar.gz`. This output path may then be referenced
within an RPM spec or when creating a package in order to access the vendored
upstream dependencies during build time.
```
[[package.metadata.build-package.external-files]]
path = "foo"
url = "https://foo"
sha512 = "abcdef"
bundle-modules = [ "go" ]
bundle-root-path = "path/to/module"
bundle-output-path = "path/to/output.tar.gz"
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

`releases-url` is ignored by buildsys, but can be used by packager maintainers
to indicate a good URL for checking whether the software has had a new release.
```
[package.metadata.build-package]
releases-url = "https://www.example.com/releases"
```

## Metadata for variants

`included-packages` is a list of packages that should be included in a variant.
```
[package.metadata.build-variant]
included-packages = ["release"]
```

`image-format` is the desired format for the built images.
This can be `raw` (the default), `vmdk`, or `qcow2`.
```
[package.metadata.build-variant]
image-format = "vmdk"
```

`image-layout` is the desired layout for the built images.

`os-image-size-gib` is the desired size of the "os" disk image in GiB.
The specified size will be automatically divided into two banks, where each
bank contains the set of partitions needed for in-place upgrades. Roughly 40%
will be available for each root filesystem partition, with the rest allocated
to other essential system partitions.

`data-image-size-gib` is the desired size of the "data" disk image in GiB.
The full size will be used for the single data partition, except for the 2 MiB
overhead for the GPT labels and partition alignment. The data partition will be
automatically resized to fill the disk on boot, so it is usually not necessary
to increase this value.

`partition-plan` is the desired strategy for image partitioning.
This can be `split` (the default) for "os" and "data" images backed by separate
volumes, or `unified` to have "os" and "data" share the same volume.
```
[package.metadata.build-variant.image-layout]
os-image-size-gib = 2
data-image-size-gib = 1
partition-plan = "split"
```

`supported-arches` is the list of architectures the variant is able to run on.
The values can be `x86_64` and `aarch64`.
If not specified, the variant can run on any of those architectures.
```
[package.metadata.build-variant]
supported-arches = ["x86_64"]
```

`kernel-parameters` is a list of extra parameters to be added to the kernel command line.
The given parameters are inserted at the start of the command line.
```
[package.metadata.build-variant]
kernel-parameters = [
   "console=ttyS42",
]

`grub-features` is a list of supported grub features.
This list allows us to conditionally use or exclude certain grub features in specific variants.
The only supported value at this time is `set-private-var`.
This value means that the grub config for the current variant includes the command to find the
BOTTLEROCKET_PRIVATE partition and set the appropriate `$private` variable for the grub to
consume.
Adding this value to `grub-features` enables the use of Boot Config.
```
[package.metadata.build-variant]
grub-features = [
   "set-private-var",
]
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
        let manifest_data =
            fs::read_to_string(path).context(error::ManifestFileReadSnafu { path })?;
        toml::from_str(&manifest_data).context(error::ManifestFileLoadSnafu { path })
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

    /// Convenience method to return the image layout, if specified.
    pub(crate) fn image_layout(&self) -> Option<&ImageLayout> {
        self.build_variant().and_then(|b| b.image_layout.as_ref())
    }

    /// Convenience method to return the supported architectures for this variant.
    pub(crate) fn supported_arches(&self) -> Option<&HashSet<SupportedArch>> {
        self.build_variant()
            .and_then(|b| b.supported_arches.as_ref())
    }

    /// Convenience method to return the kernel parameters for this variant.
    pub(crate) fn kernel_parameters(&self) -> Option<&Vec<String>> {
        self.build_variant()
            .and_then(|b| b.kernel_parameters.as_ref())
    }

    /// Convenience method to return the GRUB features for this variant.
    pub(crate) fn grub_features(&self) -> Option<&Vec<GrubFeature>> {
        self.build_variant().and_then(|b| b.grub_features.as_ref())
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
#[allow(dead_code)]
pub(crate) struct BuildPackage {
    pub(crate) external_files: Option<Vec<ExternalFile>>,
    pub(crate) package_name: Option<String>,
    pub(crate) releases_url: Option<String>,
    pub(crate) source_groups: Option<Vec<PathBuf>>,
    pub(crate) variant_sensitive: Option<bool>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct BuildVariant {
    pub(crate) included_packages: Option<Vec<String>>,
    pub(crate) image_format: Option<ImageFormat>,
    pub(crate) image_layout: Option<ImageLayout>,
    pub(crate) supported_arches: Option<HashSet<SupportedArch>>,
    pub(crate) kernel_parameters: Option<Vec<String>>,
    pub(crate) grub_features: Option<Vec<GrubFeature>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ImageFormat {
    Qcow2,
    Raw,
    Vmdk,
}

#[derive(Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct ImageLayout {
    #[serde(default = "ImageLayout::default_os_image_size_gib")]
    pub(crate) os_image_size_gib: u32,
    #[serde(default = "ImageLayout::default_data_image_size_gib")]
    pub(crate) data_image_size_gib: u32,
    #[serde(default = "ImageLayout::default_partition_plan")]
    pub(crate) partition_plan: PartitionPlan,
}

/// These are the historical defaults for all variants, before we added support
/// for customizing these properties.
static DEFAULT_OS_IMAGE_SIZE_GIB: u32 = 2;
static DEFAULT_DATA_IMAGE_SIZE_GIB: u32 = 1;
static DEFAULT_PARTITION_PLAN: PartitionPlan = PartitionPlan::Split;

impl ImageLayout {
    fn default_os_image_size_gib() -> u32 {
        DEFAULT_OS_IMAGE_SIZE_GIB
    }

    fn default_data_image_size_gib() -> u32 {
        DEFAULT_DATA_IMAGE_SIZE_GIB
    }

    fn default_partition_plan() -> PartitionPlan {
        DEFAULT_PARTITION_PLAN
    }
}

impl Default for ImageLayout {
    fn default() -> Self {
        Self {
            os_image_size_gib: Self::default_os_image_size_gib(),
            data_image_size_gib: Self::default_data_image_size_gib(),
            partition_plan: Self::default_partition_plan(),
        }
    }
}

#[derive(Deserialize, Debug, Copy, Clone)]
#[serde(rename_all = "lowercase")]
pub(crate) enum PartitionPlan {
    Split,
    Unified,
}

#[derive(Deserialize, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub(crate) enum SupportedArch {
    X86_64,
    Aarch64,
}

/// Map a Linux architecture into the corresponding Docker architecture.
impl SupportedArch {
    pub(crate) fn goarch(&self) -> &'static str {
        match self {
            SupportedArch::X86_64 => "amd64",
            SupportedArch::Aarch64 => "arm64",
        }
    }
}

#[derive(Deserialize, Debug, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum GrubFeature {
    SetPrivateVar,
}

impl fmt::Display for GrubFeature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GrubFeature::SetPrivateVar => write!(f, "GRUB_SET_PRIVATE_VAR"),
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub(crate) enum BundleModule {
    Go,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct ExternalFile {
    pub(crate) path: Option<PathBuf>,
    pub(crate) sha512: String,
    pub(crate) url: String,
    pub(crate) bundle_modules: Option<Vec<BundleModule>>,
    pub(crate) bundle_root_path: Option<PathBuf>,
    pub(crate) bundle_output_path: Option<PathBuf>,
}

impl fmt::Display for SupportedArch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SupportedArch::X86_64 => write!(f, "x86_64"),
            SupportedArch::Aarch64 => write!(f, "aarch64"),
        }
    }
}
