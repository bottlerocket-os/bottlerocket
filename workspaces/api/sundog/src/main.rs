/*!
# Introduction

sundog is a small program to handle settings that must be generated at OS runtime.

It requests settings generators from the API and runs them.
The output is collected and sent to a known Thar API server endpoint and committed.
*/

use snafu::{ensure, ResultExt};
use std::collections::HashMap;
use std::process;
use std::str;

#[macro_use]
extern crate log;

// FIXME Get these from configuration in the future
const API_METADATA_URI: &str = "http://localhost:4242/metadata";
const API_SETTINGS_URI: &str = "http://localhost:4242/settings";
const API_COMMIT_URI: &str = "http://localhost:4242/settings/commit";

type Result<T> = std::result::Result<T, SundogError>;

/// Potential errors during Sundog execution
mod error {
    use snafu::Snafu;

    /// Potential errors during dynamic settings retrieval
    #[derive(Debug, Snafu)]
    #[snafu(visibility = "pub(super)")]
    pub(super) enum SundogError {
        #[snafu(display("Logger setup error: {}", source))]
        Logger { source: log::SetLoggerError },

        #[snafu(display("Command failure - '{}': {}", program, source))]
        CommandFailure {
            program: String,
            source: std::io::Error,
        },

        #[snafu(display(
            "Setting generator '{}' failed with exit code {}: {}",
            program,
            code,
            stderr
        ))]
        FailedSettingGenerator {
            program: String,
            code: String,
            stderr: String,
        },

        #[snafu(display("Invalid (non-utf8) generator output - '{}': {}", program, source))]
        GeneratorOutput {
            program: String,
            source: std::str::Utf8Error,
        },

        #[snafu(display("Error sending {} to '{}': {}", method, uri, source))]
        APIRequest {
            method: &'static str,
            uri: String,
            source: reqwest::Error,
        },

        #[snafu(display("Error response from {} to '{}': {}", method, uri, source))]
        APIResponse {
            method: &'static str,
            uri: String,
            source: reqwest::Error,
        },

        #[snafu(display(
            "Error deserializing response as JSON from {} to '{}': {}",
            method,
            uri,
            source
        ))]
        ResponseJson {
            method: &'static str,
            uri: String,
            source: reqwest::Error,
        },

        #[snafu(display("Error deserializing HashMap to struct: {}", source))]
        MaptoJSON { source: serde_json::error::Error },

        #[snafu(display("Error updating settings through '{}': {}", uri, source))]
        UpdatingAPISettings { uri: String, source: reqwest::Error },

        #[snafu(display("Error committing changes to '{}': {}", uri, source))]
        CommittingAPISettings { uri: String, source: reqwest::Error },
    }
}

use error::SundogError;

/// Request the setting generators from the API.
fn get_setting_generators(client: &reqwest::Client) -> Result<HashMap<String, String>> {
    let uri = API_METADATA_URI.to_string() + "/setting-generators";

    debug!("Requesting setting generators from API");
    let generators: HashMap<String, String> = client
        .get(&uri)
        .send()
        .context(error::APIRequest {
            method: "GET",
            uri: uri.as_str(),
        })?
        .error_for_status()
        .context(error::APIResponse {
            method: "GET",
            uri: uri.as_str(),
        })?
        .json()
        .context(error::ResponseJson {
            method: "GET",
            uri: uri.as_str(),
        })?;
    trace!("Generators: {:?}", &generators);

    Ok(generators)
}

/// Run the setting generators and collect the output
fn get_dynamic_settings(generators: HashMap<String, String>) -> Result<HashMap<String, String>> {
    let mut settings = HashMap::new();

    // For each generator, run it and capture the output
    for (setting, generator) in generators {
        debug!("Running generator {}", &generator);
        let result = process::Command::new(&generator)
            .output()
            .context(error::CommandFailure {
                program: generator.as_str(),
            })?;

        // If the generator exits nonzero, bomb out here
        ensure!(
            result.status.success(),
            error::FailedSettingGenerator {
                code: result
                    .status
                    .code()
                    .map(|i| i.to_string())
                    .unwrap_or("signal".to_string()),
                program: generator.as_str(),
                stderr: String::from_utf8_lossy(&result.stderr)
            }
        );

        // Build a valid utf8 string from the stdout and trim any whitespace
        let output = str::from_utf8(&result.stdout)
            .context(error::GeneratorOutput {
                program: generator.as_str(),
            })?
            .trim()
            .to_string();
        trace!("Generator '{}' output: {}", &generator, &output);

        settings.insert(setting, output);
    }

    Ok(settings)
}

/// Send and commit the settings to the datastore through the API
fn set_settings(client: &reqwest::Client, setting_map: HashMap<String, String>) -> Result<()> {
    // Serialize our map of { setting: value } into JSON
    let settings = serde_json::to_string(&setting_map).context(error::MaptoJSON)?;
    trace!("Settings to PATCH: {}", &settings);

    client
        .patch(API_SETTINGS_URI)
        .body(settings)
        .send()
        .context(error::APIRequest {
            method: "PATCH",
            uri: API_SETTINGS_URI,
        })?
        .error_for_status()
        .context(error::UpdatingAPISettings {
            uri: API_SETTINGS_URI,
        })?;

    // POST to /commit to actually make the changes
    debug!("POST-ing to /commit to finalize the changes");
    client
        .post(API_COMMIT_URI)
        .body("")
        .send()
        .context(error::APIRequest {
            method: "POST",
            uri: API_COMMIT_URI,
        })?
        .error_for_status()
        .context(error::CommittingAPISettings {
            uri: API_COMMIT_URI,
        })?;

    Ok(())
}

fn main() -> Result<()> {
    // TODO Fix this later when we decide our logging story
    // Start the logger
    stderrlog::new()
        .module(module_path!())
        .timestamp(stderrlog::Timestamp::Millisecond)
        .verbosity(2)
        .color(stderrlog::ColorChoice::Never)
        .init()
        .context(error::Logger)?;

    info!("Sundog started");

    // Create a client for all our API calls
    let client = reqwest::Client::new();

    info!("Retrieving setting generators");
    let generators = get_setting_generators(&client)?;
    if generators.is_empty() {
        info!("No settings to generate, exiting");
        process::exit(0)
    }

    info!("Retrieving settings values");
    let settings = get_dynamic_settings(generators)?;
    if settings.is_empty() {
        error!("No settings values were retrieved!");
        process::exit(1)
    }

    info!("Sending settings values to the API");
    set_settings(&client, settings)?;

    Ok(())
}
