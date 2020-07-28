//! The config module owns the definition and loading process for our configuration sources.

use crate::deserialize_offset;
use chrono::Duration;
use serde::Deserialize;
use snafu::ResultExt;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use url::Url;

/// Configuration needed to load and create repos
#[derive(Debug, Deserialize)]
pub(crate) struct InfraConfig {
    pub(crate) root_role_path: Option<PathBuf>,
    pub(crate) signing_keys: Option<HashMap<String, SigningKeyConfig>>,
    pub(crate) repo: Option<HashMap<String, RepoConfig>>,
}

impl InfraConfig {
    /// Deserializes an InfraConfig from a given path
    pub(crate) fn from_path<P>(path: P) -> Result<InfraConfig>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let infra_config_str = fs::read_to_string(path).context(error::File { path })?;
        toml::from_str(&infra_config_str).context(error::InvalidToml { path })
    }
}

/// Location of signing keys
// These variant names are lowercase because they have to match the text in Infra.toml, and it's
// more common for TOML config to be lowercase.
#[allow(non_camel_case_types)]
#[derive(Debug, Deserialize)]
pub(crate) enum SigningKeyConfig {
    file { path: PathBuf },
    ssm { parameter: String },
}

/// Location of existing published repo
#[derive(Debug, Deserialize)]
pub(crate) struct RepoConfig {
    pub(crate) metadata_base_url: Option<Url>,
    pub(crate) targets_url: Option<Url>,
}

/// How long it takes for each metadata type to expire
#[derive(Debug, Deserialize)]
pub(crate) struct RepoExpirationPolicy {
    #[serde(deserialize_with = "deserialize_offset")]
    pub(crate) snapshot_expiration: Duration,
    #[serde(deserialize_with = "deserialize_offset")]
    pub(crate) targets_expiration: Duration,
    #[serde(deserialize_with = "deserialize_offset")]
    pub(crate) timestamp_expiration: Duration,
}

impl RepoExpirationPolicy {
    /// Deserializes a RepoExpirationPolicy from a given path
    pub(crate) fn from_path<P>(path: P) -> Result<RepoExpirationPolicy>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let expiration_str = fs::read_to_string(path).context(error::File { path })?;
        toml::from_str(&expiration_str).context(error::InvalidToml { path })
    }
}

mod error {
    use snafu::Snafu;
    use std::io;
    use std::path::PathBuf;

    #[derive(Debug, Snafu)]
    #[snafu(visibility = "pub(super)")]
    pub(crate) enum Error {
        #[snafu(display("Failed to read '{}': {}", path.display(), source))]
        File { path: PathBuf, source: io::Error },

        #[snafu(display("Invalid config file at '{}': {}", path.display(), source))]
        InvalidToml {
            path: PathBuf,
            source: toml::de::Error,
        },
    }
}
pub(crate) use error::Error;
type Result<T> = std::result::Result<T, error::Error>;
