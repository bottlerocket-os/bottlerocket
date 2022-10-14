//! The settings module owns the `SettingsJson` struct which contains the JSON settings data being
//! sent to the API.

use serde::Serialize;
use snafu::{OptionExt, ResultExt};

/// SettingsJson represents a change that a provider would like to make in the API.
#[derive(Debug)]
pub(crate) struct SettingsJson {
    pub(crate) json: String,
    pub(crate) desc: String,
}

impl SettingsJson {
    /// Construct a SettingsJson from a serializable object and a description of that object,
    /// which is used for logging.
    ///
    /// The serializable object is typically something like a toml::Value or serde_json::Value,
    /// since they can be easily deserialized from text input in the platform, and manipulated as
    /// desired.
    pub(crate) fn from_val<S>(data: &impl Serialize, desc: S) -> Result<Self>
    where
        S: Into<String>,
    {
        Ok(Self {
            json: serde_json::to_string(&data).context(error::SettingsToJSONSnafu)?,
            desc: desc.into(),
        })
    }

    /// Construct a SettingsJson from a string containing TOML-formatted data and a description of
    /// the object, which is used for logging.
    ///
    /// This method takes care of the easy-to-miss task of removing the outer `settings` layer from
    /// the TOML data before it gets submitted to the API.
    pub(crate) fn from_toml_str<S1, S2>(data: S1, desc: S2) -> Result<Self>
    where
        S1: AsRef<str>,
        S2: Into<String>,
    {
        let mut val: toml::Value =
            toml::from_str(data.as_ref()).context(error::TOMLUserDataParseSnafu)?;
        let table = val
            .as_table_mut()
            .context(error::UserDataNotTomlTableSnafu)?;
        let inner = table
            .remove("settings")
            .context(error::UserDataMissingSettingsSnafu)?;

        SettingsJson::from_val(&inner, desc)
    }
}

mod error {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(crate)))]
    pub(crate) enum Error {
        #[snafu(display("Error serializing settings to JSON: {}", source))]
        SettingsToJSON { source: serde_json::error::Error },

        #[snafu(display("Error parsing TOML user data: {}", source))]
        TOMLUserDataParse { source: toml::de::Error },

        #[snafu(display("TOML data did not contain 'settings' section"))]
        UserDataMissingSettings,

        #[snafu(display("Data is not a TOML table"))]
        UserDataNotTomlTable,
    }
}

pub(crate) use error::Error;
type Result<T> = std::result::Result<T, error::Error>;
