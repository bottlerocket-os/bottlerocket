//! The config module owns the definition and loading process for our configuration sources.

use chrono::Duration;
use parse_datetime::parse_offset;
use serde::{Deserialize, Deserializer};
use snafu::ResultExt;
use std::collections::{HashMap, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};
use url::Url;

/// Configuration needed to load and create repos
#[derive(Debug, Deserialize)]
pub struct InfraConfig {
    // Repo subcommand config
    pub root_role_path: Option<PathBuf>,
    pub signing_keys: Option<HashMap<String, SigningKeyConfig>>,
    pub repo: Option<HashMap<String, RepoConfig>>,

    // Config for AWS specific subcommands
    pub aws: Option<AwsConfig>,
}

impl InfraConfig {
    /// Deserializes an InfraConfig from a given path
    pub fn from_path<P>(path: P) -> Result<InfraConfig>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let infra_config_str = fs::read_to_string(path).context(error::File { path })?;
        toml::from_str(&infra_config_str).context(error::InvalidToml { path })
    }
}

/// AWS-specific infrastructure configuration
#[derive(Debug, Default, Deserialize)]
pub struct AwsConfig {
    #[serde(default)]
    pub regions: VecDeque<String>,
    pub role: Option<String>,
    pub profile: Option<String>,
    #[serde(default)]
    pub region: HashMap<String, AwsRegionConfig>,
    pub ssm_prefix: Option<String>,
}

/// AWS region-specific configuration
#[derive(Debug, Deserialize)]
pub struct AwsRegionConfig {
    pub role: Option<String>,
    pub endpoint: Option<String>,
}

/// Location of signing keys
// These variant names are lowercase because they have to match the text in Infra.toml, and it's
// more common for TOML config to be lowercase.
#[allow(non_camel_case_types)]
#[derive(Debug, Deserialize)]
pub enum SigningKeyConfig {
    file { path: PathBuf },
    kms { key_id: String },
    ssm { parameter: String },
}

/// Location of existing published repo
#[derive(Debug, Deserialize)]
pub struct RepoConfig {
    pub metadata_base_url: Option<Url>,
    pub targets_url: Option<Url>,
}

/// How long it takes for each metadata type to expire
#[derive(Debug, Deserialize)]
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
    }
}
pub use error::Error;
pub type Result<T> = std::result::Result<T, error::Error>;
