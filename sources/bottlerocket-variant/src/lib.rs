/*!
This library provides a structure for representing a Bottlerocket variant as well as functionality
useful in build scripts and other tooling that is variant-aware.
*/

use error::Error;
use serde::de::Error as SerdeError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use snafu::{ensure, OptionExt, ResultExt};
use std::borrow::Borrow;
use std::convert::TryFrom;
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::str::FromStr;

/// The name of the environment variable that tells us the current variant. Variant-sensitive crates
/// will need to be rebuilt if this changes. `Makefile.toml` emits the variant string in the
/// `BUILDSYS_VARIANT` environment variable. This is then passed to crate builds by the `Dockerfile`
/// as `VARIANT`.
pub const VARIANT_ENV: &str = "VARIANT";

/// The default `variant_version`. If the third position of a variant string tuple does not exist,
/// then the `variant_version` is `"undefined"`.
pub const DEFAULT_VARIANT_VERSION: &str = "0";

/// The default `variant_flavor`. If the fourth position of a variant string tuple does not exist,
/// then the variant_flavor cfg will be `"none"`.
pub const DEFAULT_VARIANT_FLAVOR: &str = "none";

pub type Result<T> = std::result::Result<T, error::Error>;

pub mod error {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub enum Error {
        #[snafu(display(
            "The 'VARIANT' environment variable is missing or unable to be read: {}",
            source
        ))]
        VariantEnv { source: std::env::VarError },

        #[snafu(display("The '{}' segment of the variant '{}' is missing", part_name, variant))]
        VariantPart { part_name: String, variant: String },

        #[snafu(display("The '{}' segment of the variant '{}' is empty", part_name, variant))]
        VariantPartEmpty { part_name: String, variant: String },
    }
}

/// # Variant
///
/// Represents a Bottlerocket variant string. These are in the form
/// `platform-runtime-[variant_version][-variant_flavor]`.
///
/// For example, here are some valid variant strings:
/// - aws-ecs-1
/// - vmware-k8s-1.23
/// - metal-dev
/// - aws-k8s-1.24-nvidia
///
/// The `platform` and `runtime` values are required. `variant_version` and `variant_flavor` values
/// are optional and will default to `"0"` and `"none"` respectively.
///
/// In a `build.rs` file, you may use the function `emit_cfgs()` if you need to conditionally
/// compile code based on variant characteristics.
///
/// # Example
///
/// ```rust
/// use bottlerocket_variant::{Variant, VARIANT_ENV};
/// std::env::set_var(VARIANT_ENV, "metal-k8s-1.24");
/// let variant = Variant::from_env().unwrap();
///
/// assert_eq!(variant.version().unwrap(), "1.24");
///
/// // In a `build.rs` file, you may want to emit cfgs that you can use for conditional compilation.
/// variant.emit_cfgs();
/// ```
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Variant {
    variant: String,
    platform: String,
    runtime: String,
    family: String,
    version: Option<String>,
    variant_flavor: Option<String>,
}

impl Variant {
    /// Create a new `Variant` from a dash-delimited string. The first two tuple positions,
    /// `platform` and `runtime` are required. The next two, representing `variant_version` and
    /// `variant_flavor`, are optional.
    ///
    /// # Valid Values
    ///
    /// - `aws-dev`
    /// - `vmware-k8s-1.24`
    /// - `aws-k8s-1.24-nvidia`
    /// - `aws-k8s-1.24-nvidia-some-additional-ignored-tuple-positions`
    ///
    /// # Invalid Values
    ///
    /// - `aws`
    /// - `aws-dev-`
    ///
    /// # Example
    ///
    /// ```rust
    /// use bottlerocket_variant::Variant;
    /// let variant = Variant::new("aws-k8s").unwrap();
    /// assert_eq!(variant.family(), "aws-k8s");
    /// ```
    pub fn new<S: Into<String>>(value: S) -> Result<Self> {
        Self::parse(value)
    }

    /// Create a new `Variant` from the `VARIANT` environment variable's value. The environment
    /// variable must exist and its value must be a valid variant string tuple.
    pub fn from_env() -> Result<Self> {
        let value = std::env::var(VARIANT_ENV).context(error::VariantEnvSnafu)?;
        Variant::new(value)
    }

    /// The variant's platform. This is the first member of the tuple. For example, in `vmware-dev`,
    /// `vmware` is the platform.
    pub fn platform(&self) -> &str {
        &self.platform
    }

    /// The variant's runtime. This is the second member of the tuple. For example, in
    /// `metal-k8s-1.24`, `k8s` is the `runtime`.
    pub fn runtime(&self) -> &str {
        &self.runtime
    }

    /// The variant's family. This is the `platform` and `runtime` together. For example, in
    /// `aws-k8s-1.24`, `aws-k8s` is the `family`.
    pub fn family(&self) -> &str {
        &self.family
    }

    /// The variant's version. This is the optional third value in the variant string tuple. For
    /// example for `aws-ecs-1` the `version` is `1`. If the `version` does not exist,
    /// [`DEFAULT_VARIANT_VERSION`] is returned.
    pub fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }

    /// The variant's flavor. This is the optional fourth value in the variant string tuple. For
    /// example for `aws-k8s-1.24-nvidia` the `variant_flavor` is `nvidia`.
    pub fn variant_flavor(&self) -> Option<&str> {
        self.variant_flavor.as_deref()
    }

    /// This can be used in a `build.rs` file to tell cargo that the crate needs to be rebuilt if
    /// the variant changes.
    pub fn rerun_if_changed() {
        println!("cargo:rerun-if-env-changed={}", VARIANT_ENV);
    }

    /// This can be used in a `build.rs` file to emit `cfg` values that can be used for conditional
    /// compilation based on variant characteristics. This function also emits rerun-if-changed so
    /// that variant-sensitive builds will rebuild if the variant changes.
    ///
    /// # Example
    ///
    /// Given a variant `aws-k8s-1.24`, if this function has been called in `build.rs`, then
    /// all of the following conditional complition checks would evaluate to `true`.
    ///
    /// `#[cfg(variant = "aws-k8s-1.24")]`
    /// `#[cfg(variant_platform = "aws")]`
    /// `#[cfg(variant_runtime = "k8s")]`
    /// `#[cfg(variant_family = "aws-k8s")]`
    /// `#[cfg(variant_version = "1.24")]`
    /// `#[cfg(variant_flavor = "none")]`
    pub fn emit_cfgs(&self) {
        Self::rerun_if_changed();
        println!("cargo:rustc-cfg=variant=\"{}\"", self);
        println!("cargo:rustc-cfg=variant_platform=\"{}\"", self.platform());
        println!("cargo:rustc-cfg=variant_runtime=\"{}\"", self.runtime());
        println!("cargo:rustc-cfg=variant_family=\"{}\"", self.family());
        println!(
            "cargo:rustc-cfg=variant_version=\"{}\"",
            self.version().unwrap_or(DEFAULT_VARIANT_VERSION)
        );
        println!(
            "cargo:rustc-cfg=variant_flavor=\"{}\"",
            self.variant_flavor().unwrap_or(DEFAULT_VARIANT_FLAVOR)
        );
    }

    fn parse<S: Into<String>>(value: S) -> Result<Self> {
        let variant = value.into();
        let mut parts = variant.split('-');
        let platform = parts
            .next()
            .with_context(|| error::VariantPartSnafu {
                part_name: "platform",
                variant: variant.clone(),
            })?
            .to_string();
        ensure!(
            !platform.is_empty(),
            error::VariantPartEmptySnafu {
                part_name: "platform",
                variant: variant.clone()
            }
        );
        let runtime = parts
            .next()
            .with_context(|| error::VariantPartSnafu {
                part_name: "runtime",
                variant: variant.clone(),
            })?
            .to_string();
        ensure!(
            !runtime.is_empty(),
            error::VariantPartEmptySnafu {
                part_name: "runtime",
                variant: variant.clone()
            }
        );
        let variant_family = format!("{}-{}", platform, runtime);
        let variant_version = parts.next().map(|s| s.to_string());
        if let Some(value) = variant_version.as_ref() {
            ensure!(
                !value.is_empty(),
                error::VariantPartEmptySnafu {
                    part_name: "variant_version",
                    variant: variant.clone()
                }
            );
        }
        let variant_flavor = parts.next().map(|s| s.to_string());
        if let Some(value) = variant_flavor.as_ref() {
            ensure!(
                !value.is_empty(),
                error::VariantPartEmptySnafu {
                    part_name: "variant_flavor",
                    variant: variant.clone()
                }
            );
        }
        Ok(Self {
            variant,
            platform,
            runtime,
            family: variant_family,
            version: variant_version,
            variant_flavor,
        })
    }
}

impl FromStr for Variant {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Variant::new(s)
    }
}

impl TryFrom<String> for Variant {
    type Error = Error;

    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        Variant::new(value)
    }
}

impl TryFrom<&str> for Variant {
    type Error = Error;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        Variant::new(value)
    }
}

impl Serialize for Variant {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.variant)
    }
}

impl<'de> Deserialize<'de> for Variant {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Variant, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Variant::new(value).map_err(|e| D::Error::custom(format!("Error parsing variant: {}", e)))
    }
}

impl Deref for Variant {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.variant
    }
}

impl Borrow<String> for Variant {
    fn borrow(&self) -> &String {
        &self.variant
    }
}

impl Borrow<str> for Variant {
    fn borrow(&self) -> &str {
        &self.variant
    }
}

impl AsRef<str> for Variant {
    fn as_ref(&self) -> &str {
        &self.variant
    }
}

impl Display for Variant {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.variant, f)
    }
}

impl From<Variant> for String {
    fn from(x: Variant) -> Self {
        x.variant
    }
}

impl PartialEq<str> for Variant {
    fn eq(&self, other: &str) -> bool {
        self.variant == other
    }
}

impl PartialEq<String> for Variant {
    fn eq(&self, other: &String) -> bool {
        &self.variant == other
    }
}

impl PartialEq<&str> for Variant {
    fn eq(&self, other: &&str) -> bool {
        &self.variant == other
    }
}

impl PartialEq<Variant> for str {
    fn eq(&self, other: &Variant) -> bool {
        self == other.variant
    }
}

impl PartialEq<Variant> for String {
    fn eq(&self, other: &Variant) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<Variant> for &str {
    fn eq(&self, other: &Variant) -> bool {
        self == &other.variant
    }
}

#[test]
fn parse_ok() {
    struct Test {
        input: &'static str,
        platform: &'static str,
        runtime: &'static str,
        variant_family: &'static str,
        variant_version: Option<&'static str>,
        variant_flavor: Option<&'static str>,
    }

    let tests = vec![
        Test {
            input: "aws-k8s-1.21",
            platform: "aws",
            runtime: "k8s",
            variant_family: "aws-k8s",
            variant_version: Some("1.21"),
            variant_flavor: None,
        },
        Test {
            input: "metal-dev",
            platform: "metal",
            runtime: "dev",
            variant_family: "metal-dev",
            variant_version: None,
            variant_flavor: None,
        },
        Test {
            input: "aws-ecs-1",
            platform: "aws",
            runtime: "ecs",
            variant_family: "aws-ecs",
            variant_version: Some("1"),
            variant_flavor: None,
        },
        Test {
            input: "aws-k8s-1.24-nvidia-some-additional-ignored-tuple-positions",
            platform: "aws",
            runtime: "k8s",
            variant_family: "aws-k8s",
            variant_version: Some("1.24"),
            variant_flavor: Some("nvidia"),
        },
    ];

    for test in tests {
        let parsed = Variant::new(test.input).unwrap();
        assert_eq!(parsed, test.input);
        assert_eq!(test.input, parsed);
        assert_eq!(parsed.platform(), test.platform.to_string());
        assert_eq!(parsed.runtime(), test.runtime);
        assert_eq!(parsed.family(), test.variant_family);
        assert_eq!(parsed.version(), test.variant_version);
        assert_eq!(parsed.variant_flavor(), test.variant_flavor);
    }
}

#[test]
fn parse_err() {
    let tests = vec!["aws", "aws-", "aws-dev-", "aws-k8s-1.24-"];
    for test in tests {
        let result = Variant::new(test);
        assert!(
            result.is_err(),
            "Expected Variant::new(\"{}\") to return an error",
            test
        );
    }
}
