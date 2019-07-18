/*!
# Background

thar-be-settings is a simple configuration applier.

It is intended to be called from, and work directly with, the API server in Thar, the OS.
After a settings change, this program queries the API to determine which services and configuration files are affected by that change.
Once it has done so, it renders and rewrites the affected configuration files and restarts any affected services.
*/

#[macro_use]
extern crate log;

use snafu::ResultExt;
use std::collections::HashSet;
use std::io::{self, Read};

pub mod client;
pub mod config;
pub mod error;
pub mod helpers;
pub mod service;
pub mod settings;
pub mod template;

pub use error::TBSError;
type Result<T> = std::result::Result<T, TBSError>;

// FIXME Get these from configuration in the future
const API_CONFIGURATION_URI: &str = "http://localhost:4242/configuration-files";
const API_METADATA_URI: &str = "http://localhost:4242/metadata";
const API_SETTINGS_URI: &str = "http://localhost:4242/settings";
const API_SERVICES_URI: &str = "http://localhost:4242/services";

/// Read stdin and parse into JSON
pub fn get_changed_settings() -> Result<HashSet<String>> {
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .context(error::ReadInput { location: "stdin" })?;
    trace!("Raw input from stdin: {}", &input);

    // Settings should be a vec of strings
    debug!("Parsing stdin as JSON");
    let changed_settings: HashSet<String> =
        serde_json::from_str(&input).context(error::InvalidInput {
            reason: "Input must be a JSON array of strings",
            input: input,
        })?;
    trace!("Parsed input: {:?}", &changed_settings);

    Ok(changed_settings)
}
