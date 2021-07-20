//! The config module owns the definition and loading process for our configuration sources.
pub mod vmware;

use crate::vmware::VmwareConfig;
use chrono::Duration;
use parse_datetime::parse_offset;
use serde::{Deserialize, Deserializer, Serialize};
use snafu::ResultExt;
use std::collections::{HashMap, VecDeque};
use std::convert::TryFrom;
use std::fs;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use url::Url;

/// Configuration needed to load and create repos
#[derive(Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct InfraConfig {
    // Repo subcommand config
    pub repo: Option<HashMap<String, RepoConfig>>,

    // Config for AWS specific subcommands
    pub aws: Option<AwsConfig>,

    // Config for VMware specific subcommands
    pub vmware: Option<VmwareConfig>,
}

impl InfraConfig {
    /// Deserializes an InfraConfig from a given path
    pub fn from_path<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let infra_config_str = fs::read_to_string(path).context(error::File { path })?;
        toml::from_str(&infra_config_str).context(error::InvalidToml { path })
    }

    /// Deserializes an InfraConfig from a given path, if it exists, otherwise builds a default
    /// config
    pub fn from_path_or_default<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        if path.as_ref().exists() {
            Self::from_path(path)
        } else {
            Ok(Self::default())
        }
    }

    /// Deserializes an InfraConfig from a Infra.lock file at a given path
    pub fn from_lock_path<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let infra_config_str = fs::read_to_string(path).context(error::File { path })?;
        serde_yaml::from_str(&infra_config_str).context(error::InvalidLock { path })
    }
}

/// S3-specific TUF infrastructure configuration
#[derive(Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct S3Config {
    pub region: Option<String>,
    #[serde(default)]
    pub s3_prefix: String,
    pub vpc_endpoint_id: Option<String>,
    pub stack_arn: Option<String>,
    pub bucket_name: Option<String>,
}

/// AWS-specific infrastructure configuration
#[derive(Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AwsConfig {
    #[serde(default)]
    pub regions: VecDeque<String>,
    pub role: Option<String>,
    pub profile: Option<String>,
    #[serde(default)]
    pub region: HashMap<String, AwsRegionConfig>,
    pub ssm_prefix: Option<String>,
    pub s3: Option<HashMap<String, S3Config>>,
}

/// AWS region-specific configuration
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AwsRegionConfig {
    pub role: Option<String>,
    pub endpoint: Option<String>,
}

/// Location of signing keys
// These variant names are lowercase because they have to match the text in Infra.toml, and it's
// more common for TOML config to be lowercase.
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum SigningKeyConfig {
    file {
        path: PathBuf,
    },
    kms {
        key_id: Option<String>,
        #[serde(flatten)]
        config: Option<KMSKeyConfig>,
    },
    ssm {
        parameter: String,
    },
}

/// AWS region-specific configuration
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
//#[serde(deny_unknown_fields)]
pub struct KMSKeyConfig {
    #[serde(default)]
    pub available_keys: HashMap<String, String>,
    pub key_alias: Option<String>,
    #[serde(default)]
    pub regions: VecDeque<String>,
    #[serde(default)]
    pub key_stack_arns: HashMap<String, String>,
}

impl TryFrom<SigningKeyConfig> for Url {
    type Error = ();
    fn try_from(key: SigningKeyConfig) -> std::result::Result<Self, Self::Error> {
        match key {
            SigningKeyConfig::file { path } => Url::from_file_path(path),
            // We don't support passing profiles to tough in the name of the key/parameter, so for
            // KMS and SSM we prepend a slash if there isn't one present.
            SigningKeyConfig::kms { key_id, .. } => {
                let mut key_id = key_id.unwrap_or_else(Default::default);
                key_id = if key_id.starts_with('/') {
                    key_id.to_string()
                } else {
                    format!("/{}", key_id)
                };
                Url::parse(&format!("aws-kms://{}", key_id)).map_err(|_| ())
            }
            SigningKeyConfig::ssm { parameter } => {
                let parameter = if parameter.starts_with('/') {
                    parameter.to_string()
                } else {
                    format!("/{}", parameter)
                };
                Url::parse(&format!("aws-ssm://{}", parameter)).map_err(|_| ())
            }
        }
    }
}

/// Represents a Bottlerocket repo's location and the metadata needed to update the repo
#[derive(Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RepoConfig {
    pub root_role_url: Option<Url>,
    pub root_role_sha512: Option<String>,
    pub signing_keys: Option<SigningKeyConfig>,
    pub root_keys: Option<SigningKeyConfig>,
    pub metadata_base_url: Option<Url>,
    pub targets_url: Option<Url>,
    pub file_hosting_config_name: Option<String>,
    pub root_key_threshold: Option<NonZeroUsize>,
    pub pub_key_threshold: Option<NonZeroUsize>,
}

/// How long it takes for each metadata type to expire
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RepoExpirationPolicy {
    #[serde(deserialize_with = "deserialize_offset")]
    pub snapshot_expiration: Duration,
    #[serde(deserialize_with = "deserialize_offset")]
    pub targets_expiration: Duration,
    #[serde(deserialize_with = "deserialize_offset")]
    pub timestamp_expiration: Duration,
}

impl RepoExpirationPolicy {
    /// Deserializes a RepoExpirationPolicy from a given path
    pub fn from_path<P>(path: P) -> Result<RepoExpirationPolicy>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let expiration_str = fs::read_to_string(path).context(error::File { path })?;
        toml::from_str(&expiration_str).context(error::InvalidToml { path })
    }
}

/// Deserializes a Duration in the form of "in X hours/days/weeks"
fn deserialize_offset<'de, D>(deserializer: D) -> std::result::Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    parse_offset(s).map_err(serde::de::Error::custom)
}

mod error {
    use snafu::Snafu;
    use std::io;
    use std::path::PathBuf;

    #[derive(Debug, Snafu)]
    #[snafu(visibility = "pub(super)")]
    pub enum Error {
        #[snafu(display("Failed to read '{}': {}", path.display(), source))]
        File { path: PathBuf, source: io::Error },

        #[snafu(display("Invalid config file at '{}': {}", path.display(), source))]
        InvalidToml {
            path: PathBuf,
            source: toml::de::Error,
        },

        #[snafu(display("Invalid lock file at '{}': {}", path.display(), source))]
        InvalidLock {
            path: PathBuf,
            source: serde_yaml::Error,
        },

        #[snafu(display("Missing config: {}", what))]
        MissingConfig { what: String },
    }
}
pub use error::Error;
pub type Result<T> = std::result::Result<T, error::Error>;
