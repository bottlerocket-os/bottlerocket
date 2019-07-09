/*!
# Introduction

sundog is a small program to handle settings that must be generated at OS runtime.

It requests settings generators from the API and runs them.
The output is collected and sent to a known Thar API server endpoint and committed.
*/

use snafu::ResultExt;
use std::collections::HashMap;
use std::env;
use std::process;
use std::str;

use apiserver::datastore::{self, deserialization};
use apiserver::model;

#[macro_use]
extern crate log;

// FIXME Get these from configuration in the future
const API_METADATA_URI: &str = "http://localhost:4242/metadata";
const API_SETTINGS_URI: &str = "http://localhost:4242/settings";
const API_COMMIT_URI: &str = "http://localhost:4242/settings/commit";

/// Potential errors during Sundog execution
mod error {
    use snafu::Snafu;

    use apiserver::datastore;
    use apiserver::datastore::deserialization;

    // Get the HTTP status code out of a reqwest::Error
    fn code(source: &reqwest::Error) -> String {
        source
            .status()
            .as_ref()
            .map(|i| i.as_str())
            .unwrap_or("Unknown")
            .to_string()
    }

    /// Potential errors during dynamic settings retrieval
    #[derive(Debug, Snafu)]
    #[snafu(visibility = "pub(super)")]
    pub(super) enum SundogError {
        #[snafu(display("Logger setup error: {}", source))]
        Logger { source: log::SetLoggerError },

        #[snafu(display("Failed to start generator '{}': {}", program, source))]
        CommandFailure {
            program: String,
            source: std::io::Error,
        },

        #[snafu(display(
            "Setting generator '{}' failed with exit code {} - stderr: {}",
            program,
            code,
            stderr
        ))]
        FailedSettingGenerator {
            program: String,
            code: String,
            stderr: String,
        },

        #[snafu(display(
            "Setting generator '{}' returned unexpected exit code '{}' - stderr: {}",
            program,
            code,
            stderr
        ))]
        UnexpectedReturnCode {
            program: String,
            code: String,
            stderr: String,
        },

        #[snafu(display("Invalid (non-utf8) output from generator '{}': {}", program, source))]
        GeneratorOutput {
            program: String,
            source: std::str::Utf8Error,
        },

        #[snafu(display("Error '{}' sending {} to '{}': {}", code(&source), method, uri, source))]
        APIRequest {
            method: &'static str,
            uri: String,
            source: reqwest::Error,
        },

        #[snafu(display(
            "Error '{}' from {} to '{}': {}",
            code(&source),
            method,
            uri,
            source
        ))]
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

        #[snafu(display("Error deserializing HashMap to Settings: {}", source))]
        Deserialize { source: deserialization::Error },

        #[snafu(display("Error serializing Settings to JSON: {}", source))]
        Serialize { source: serde_json::error::Error },

        #[snafu(display("Error serializing command output '{}': {}", output, source))]
        SerializeScalar {
            output: String,
            source: datastore::ScalarError,
        },

        #[snafu(display("Error updating settings through '{}': {}", uri, source))]
        UpdatingAPISettings { uri: String, source: reqwest::Error },

        #[snafu(display("Error committing changes to '{}': {}", uri, source))]
        CommittingAPISettings { uri: String, source: reqwest::Error },
    }
}

use error::SundogError;

type Result<T> = std::result::Result<T, SundogError>;

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
fn get_dynamic_settings(generators: HashMap<String, String>) -> Result<model::Settings> {
    let mut settings = HashMap::new();

    // For each generator, run it and capture the output
    for (setting, generator) in generators {
        debug!("Running generator {}", &generator);
        let result = process::Command::new(&generator)
            .output()
            .context(error::CommandFailure {
                program: generator.as_str(),
            })?;

        // Match on the generator's exit code. This code lays the foundation
        // for handling alternative exit codes from generators. For now,
        // handle 0 and 1
        match result.status.code() {
            Some(0) => {}
            Some(1) => {
                return error::FailedSettingGenerator {
                    program: generator.as_str(),
                    code: 1.to_string(),
                    stderr: String::from_utf8_lossy(&result.stderr),
                }
                .fail()
            }
            Some(x) => {
                return error::UnexpectedReturnCode {
                    program: generator.as_str(),
                    code: x.to_string(),
                    stderr: String::from_utf8_lossy(&result.stderr),
                }
                .fail()
            }
            // A process will return None if terminated by a signal, regard this as
            // a failure since we could have incomplete data
            None => {
                return error::FailedSettingGenerator {
                    program: generator.as_str(),
                    code: "signal",
                    stderr: String::from_utf8_lossy(&result.stderr),
                }
                .fail()
            }
        }

        // Build a valid utf8 string from the stdout and trim any whitespace
        let output = str::from_utf8(&result.stdout)
            .context(error::GeneratorOutput {
                program: generator.as_str(),
            })?
            .trim()
            .to_string();
        trace!("Generator '{}' output: {}", &generator, &output);

        // The command output must be serialized since we intend to call the
        // datastore-level construct `from_map` on it. `from_map` treats
        // strings as a serialized structure.
        let serialized_output = datastore::serialize_scalar(&output)
            .context(error::SerializeScalar { output: output })?;
        trace!("Serialized output: {}", &serialized_output);

        settings.insert(setting, serialized_output);
    }

    // The API takes a properly nested Settings struct, so deserialize our map to a Settings
    // and ensure it is correct
    let settings_struct: model::Settings =
        deserialization::from_map(&settings).context(error::Deserialize)?;

    Ok(settings_struct)
}

/// Send and commit the settings to the datastore through the API
fn set_settings(client: &reqwest::Client, settings: model::Settings) -> Result<()> {
    // Serialize our Settings struct to the JSON wire format
    let settings_json = serde_json::to_string(&settings).context(error::Serialize)?;
    trace!("Settings to PATCH: {}", &settings_json);

    client
        .patch(API_SETTINGS_URI)
        .body(settings_json)
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

/// Store the args we receive on the command line
struct Args {
    verbosity: usize,
}

/// Print a usage message in the event a bad arg is passed
fn usage() -> ! {
    let program_name = env::args().next().unwrap_or_else(|| "program".to_string());
    eprintln!(
        r"Usage: {}
            [ --verbose --verbose ... ]
        ",
        program_name
    );
    process::exit(2);
}

/// Parse the args to the program and return an Args struct
fn parse_args(args: env::Args) -> Args {
    let mut verbosity = 2;

    for arg in args.skip(1) {
        match arg.as_ref() {
            "-v" | "--verbosity" => verbosity += 1,
            _ => usage(),
        }
    }

    Args { verbosity }
}

fn main() -> Result<()> {
    // Parse and store the args passed to the program
    let args = parse_args(env::args());

    // TODO Fix this later when we decide our logging story
    // Start the logger
    stderrlog::new()
        .module(module_path!())
        .timestamp(stderrlog::Timestamp::Millisecond)
        .verbosity(args.verbosity)
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

    info!("Sending settings values to the API");
    set_settings(&client, settings)?;

    Ok(())
}
