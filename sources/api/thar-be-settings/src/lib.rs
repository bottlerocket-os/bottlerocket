/*!
# Background

thar-be-settings is a simple configuration applier.
Its job is to update configuration files and restart services, as necessary, to make the system reflect any changes to settings.

In the normal ("specific keys") mode, it's intended to be called by the Bottlerocket API server after a settings commit.
It's told the keys that changed, and then queries metadata APIs to determine which services and configuration files are affected by changes to those keys.
Detailed data is then fetched for the relevant services and configuration files.
Configuration file data from the API includes paths to template files for each configuration file, along with the final path to write.
It then renders the templates and rewrites the affected configuration files.
Service data from the API includes any commands needed to restart services affected by configuration file changes, which are run here.

In the standalone ("all keys") mode, it queries the API for all services and configuration files, then renders and rewrites all configuration files and restarts all services.
*/

#[macro_use]
extern crate log;

use snafu::ResultExt;
use std::collections::HashSet;
use std::io::{self, Read};

pub mod config;
pub mod error;
pub mod service;

pub use error::Error;
type Result<T> = std::result::Result<T, Error>;

/// Read stdin and parse into JSON
pub fn get_changed_settings() -> Result<HashSet<String>> {
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .context(error::ReadInputSnafu { from: "stdin" })?;
    trace!("Raw input from stdin: {}", &input);

    // Settings should be a vec of strings
    debug!("Parsing stdin as JSON");
    let changed_settings: HashSet<String> =
        serde_json::from_str(&input).context(error::InvalidInputSnafu {
            reason: "Input must be a JSON array of strings",
            input,
        })?;
    trace!("Parsed input: {:?}", &changed_settings);

    Ok(changed_settings)
}
