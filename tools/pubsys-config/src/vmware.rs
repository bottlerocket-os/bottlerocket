//! The vmware module owns the definition and loading process for our VMware configuration sources.
use lazy_static::lazy_static;
use log::debug;
use serde::{Deserialize, Serialize};
use snafu::{OptionExt, ResultExt};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::{env, fs};

lazy_static! {
    /// Determine the full path to the Vsphere credentials at runtime.  This is an Option because it is
    /// possible (however unlikely) that `home_dir()` is unable to find the home directory of the
    /// current user
    pub static ref VMWARE_CREDS_PATH: Option<PathBuf> = home::home_dir().map(|home| home
        .join(".config")
        .join("pubsys")
        .join("vsphere-credentials.toml"));
}

const GOVC_USERNAME: &str = "GOVC_USERNAME";
const GOVC_PASSWORD: &str = "GOVC_PASSWORD";
const GOVC_URL: &str = "GOVC_URL";
const GOVC_DATACENTER: &str = "GOVC_DATACENTER";
const GOVC_DATASTORE: &str = "GOVC_DATASTORE";
const GOVC_NETWORK: &str = "GOVC_NETWORK";
const GOVC_RESOURCE_POOL: &str = "GOVC_RESOURCE_POOL";
const GOVC_FOLDER: &str = "GOVC_FOLDER";

/// VMware-specific infrastructure configuration
#[derive(Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct VmwareConfig {
    #[serde(default)]
    pub datacenters: Vec<String>,
    #[serde(default)]
    pub datacenter: HashMap<String, DatacenterBuilder>,
    pub common: Option<DatacenterBuilder>,
}

/// VMware datacenter-specific configuration.
///
/// Fields are optional here because this struct is used to gather environment variables, common
/// config, and datacenter-specific configuration, each of which may not have the complete set of
/// fields.  It is used to build a complete datacenter configuration (hence the "Builder" name).
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DatacenterBuilder {
    pub vsphere_url: Option<String>,
    pub datacenter: Option<String>,
    pub datastore: Option<String>,
    pub network: Option<String>,
    pub folder: Option<String>,
    pub resource_pool: Option<String>,
}

/// Helper macro for retrieving a field from another struct if the field in `self` is `None`
macro_rules! field_or {
    ($self:expr, $field:ident, $other:expr) => {
        $self
            .$field
            .as_ref()
            .or($other.and_then(|o| o.$field.as_ref()))
            .cloned()
    };
}

impl DatacenterBuilder {
    /// Create a DatacenterBuilder from environment variables
    pub fn from_env() -> Self {
        Self {
            vsphere_url: get_env(GOVC_URL),
            datacenter: get_env(GOVC_DATACENTER),
            datastore: get_env(GOVC_DATASTORE),
            network: get_env(GOVC_NETWORK),
            folder: get_env(GOVC_FOLDER),
            resource_pool: get_env(GOVC_RESOURCE_POOL),
        }
    }

    /// Creates a new DatacenterBuilder, merging fields from another (Optional)
    /// DatacenterBuilder if the field in `self` is None
    pub fn take_missing_from(&self, other: Option<&Self>) -> Self {
        Self {
            vsphere_url: field_or!(self, vsphere_url, other),
            datacenter: field_or!(self, datacenter, other),
            datastore: field_or!(self, datastore, other),
            network: field_or!(self, network, other),
            folder: field_or!(self, folder, other),
            resource_pool: field_or!(self, resource_pool, other),
        }
    }

    /// Attempts to create a `Datacenter`, consuming `self` and ensuring that each field contains a
    /// value.
    pub fn build(self) -> Result<Datacenter> {
        let get_or_err =
            |opt: Option<String>, what: &str| opt.context(error::MissingConfigSnafu { what });

        Ok(Datacenter {
            vsphere_url: get_or_err(self.vsphere_url, "vSphere URL")?,
            datacenter: get_or_err(self.datacenter, "vSphere datacenter")?,
            datastore: get_or_err(self.datastore, "vSphere datastore")?,
            network: get_or_err(self.network, "vSphere network")?,
            folder: get_or_err(self.folder, "vSphere folder")?,
            resource_pool: get_or_err(self.resource_pool, "vSphere resource pool")?,
        })
    }
}

/// A fully configured VMware datacenter, i.e. no optional fields
#[derive(Debug)]
pub struct Datacenter {
    pub vsphere_url: String,
    pub datacenter: String,
    pub datastore: String,
    pub network: String,
    pub folder: String,
    pub resource_pool: String,
}

/// VMware infrastructure credentials for all datacenters
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatacenterCredsConfig {
    #[serde(default)]
    pub datacenter: HashMap<String, DatacenterCredsBuilder>,
}

impl DatacenterCredsConfig {
    /// Deserializes a DatacenterCredsConfig from a given path
    pub fn from_path<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let creds_config_str = fs::read_to_string(path).context(error::FileSnafu { path })?;
        toml::from_str(&creds_config_str).context(error::InvalidTomlSnafu { path })
    }
}

/// VMware datacenter-specific credentials.  Fields are optional here since this struct is used to
/// gather environment variables as well as fields from file, either of which may or may not exist.
/// It is used to build a complete credentials configuration (hence the "Builder" name).
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DatacenterCredsBuilder {
    pub username: Option<String>,
    pub password: Option<String>,
}

impl DatacenterCredsBuilder {
    /// Create a DatacenterCredsBuilder from environment variables
    pub fn from_env() -> Self {
        Self {
            username: get_env(GOVC_USERNAME),
            password: get_env(GOVC_PASSWORD),
        }
    }

    /// Creates a new DatacenterCredsBuilder, merging fields from another (Optional)
    /// DatacenterCredsBuilder if the field in `self` is None
    pub fn take_missing_from(&self, other: Option<&Self>) -> Self {
        Self {
            username: field_or!(self, username, other),
            password: field_or!(self, password, other),
        }
    }
    /// Attempts to create a `DatacenterCreds`, consuming `self` and ensuring that each field
    /// contains a value
    pub fn build(self) -> Result<DatacenterCreds> {
        let get_or_err =
            |opt: Option<String>, what: &str| opt.context(error::MissingConfigSnafu { what });

        Ok(DatacenterCreds {
            username: get_or_err(self.username, "vSphere username")?,
            password: get_or_err(self.password, "vSphere password")?,
        })
    }
}

/// Fully configured datacenter credentials, i.e. no optional fields
#[derive(Debug)]
pub struct DatacenterCreds {
    pub username: String,
    pub password: String,
}

/// Attempt to retrieve an environment variable, returning None if it doesn't exist
fn get_env(var: &str) -> Option<String> {
    match env::var(var) {
        Ok(v) => Some(v),
        Err(e) => {
            debug!("Unable to read environment variable '{}': {}", var, e);
            None
        }
    }
}

mod error {
    use snafu::Snafu;
    use std::io;
    use std::path::PathBuf;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub enum Error {
        #[snafu(display("Failed to read '{}': {}", path.display(), source))]
        File { path: PathBuf, source: io::Error },

        #[snafu(display("Invalid config file at '{}': {}", path.display(), source))]
        InvalidToml {
            path: PathBuf,
            source: toml::de::Error,
        },

        #[snafu(display("Missing config: {}", what))]
        MissingConfig { what: String },
    }
}
pub use error::Error;
pub type Result<T> = std::result::Result<T, error::Error>;
