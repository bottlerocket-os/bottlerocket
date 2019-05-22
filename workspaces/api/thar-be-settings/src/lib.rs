/*!
# Background

thar-be-settings is a simple configuration applier.

It is intended to be called from, and work directly with, the API server in Thar, the OS.
After a settings change, this program queries the API to determine which services and configuration files are affected by that change.
Once it has done so, it renders and rewrites the affected configuration files and restarts any affected services.
*/

#[macro_use]
extern crate log;
#[macro_use]
extern crate derive_error;

use std::collections::HashSet;
use std::io;
use std::io::prelude::*;

use apiserver::datastore::deserialization;

pub mod client;
pub mod config;
pub mod service;
pub mod settings;
pub mod template;

type Result<T> = std::result::Result<T, TBSError>;

// FIXME Get these from configuration in the future
const API_CONFIGURATION_URI: &str = "http://localhost:4242/configuration-files";
const API_METADATA_URI: &str = "http://localhost:4242/metadata";
const API_SETTINGS_URI: &str = "http://localhost:4242/settings";
const API_SERVICES_URI: &str = "http://localhost:4242/services";

/// Read stdin and parse into JSON
pub fn get_changed_settings() -> Result<HashSet<String>> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    trace!("Raw input from stdin: {}", &input);

    // Settings should be a vec of strings
    debug!("Parsing stdin as JSON");
    let changed_settings: HashSet<String> = serde_json::from_str(&input).map_err(|_| {
        TBSError::InvalidInput(format!(
            "Input must be a JSON array of strings; received '{}'",
            &input[0..50]
        ))
    })?;
    trace!("Parsed input: {:?}", &changed_settings);

    Ok(changed_settings)
}

/// Potential errors during configuration application
#[derive(Debug, Error)]
pub enum TBSError {
    /// Restart command failure
    RestartCommand(std::io::Error),
    #[error(msg_embedded, no_from, non_std)]
    /// Restart command is invalid or malformed (empty, starts with spaces, etc.)
    InvalidRestartCommand(String),
    /// Configuration file template fails to render
    TemplateRender(handlebars::RenderError),
    /// Failure to read template file from path
    TemplateRegister(handlebars::TemplateFileError),
    /// Error making request to API
    APIRequest(reqwest::Error),
    /// JSON general error
    JSON(serde_json::error::Error),
    /// Deserialization error coming from API code
    DeserializationError(deserialization::DeserializationError),
    /// Logger setup error
    Logger(log::SetLoggerError),
    #[error(msg_embedded, no_from, non_std)]
    /// Program input is invalid JSON
    InvalidInput(String),
}
