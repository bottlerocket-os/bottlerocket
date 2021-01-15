//! The settings module owns the `SettingsJson` struct which contains the JSON settings data being
//! sent to the API.

use serde::Serialize;

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
    pub(crate) fn from_val<S>(
        data: &impl Serialize,
        desc: S,
    ) -> Result<Self, serde_json::error::Error>
    where
        S: Into<String>,
    {
        Ok(Self {
            json: serde_json::to_string(&data)?,
            desc: desc.into(),
        })
    }
}
