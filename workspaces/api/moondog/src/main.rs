/*!
# Introduction

moondog is a minimal user data agent.

It accepts TOML-formatted settings from a user data provider such as an instance metadata service.
These are sent to a known Thar API server endpoint.

Currently, Amazon EC2 user data support is implemented.
User data can also be retrieved from a file for testing.
*/

#![deny(rust_2018_idioms)]

#[macro_use]
extern crate log;

use http::StatusCode;
use serde::Serialize;
use snafu::{ensure, OptionExt, ResultExt};
use std::path::Path;
use std::{env, fs, process};

// TODO
// Tests!

// FIXME Get these from configuration in the future
const DEFAULT_API_SOCKET: &str = "/run/api.sock";
const API_SETTINGS_URI: &str = "/settings";

// We only want to run moondog once, at first boot.  Our systemd unit file has a
// ConditionPathExists that will prevent it from running again if this file exists.
// We create it after running successfully.
const MARKER_FILE: &str = "/var/lib/thar/moondog.ran";

type Result<T> = std::result::Result<T, MoondogError>;

mod error {
    use http::StatusCode;
    use snafu::Snafu;
    use std::io;
    use std::path::PathBuf;

    /// Potential errors during user data management.
    #[derive(Debug, Snafu)]
    #[snafu(visibility = "pub(super)")]
    pub(super) enum MoondogError {
        #[snafu(display("Error requesting '{}': {}", uri, source))]
        UserDataRequest { uri: String, source: reqwest::Error },

        #[snafu(display("Error {} requesting '{}': {}", code, uri, source))]
        UserDataResponse {
            code: StatusCode,
            uri: String,
            source: reqwest::Error,
        },

        #[snafu(display("Error sending {} to {}: {}", method, uri, source))]
        APIRequest {
            method: String,
            uri: String,
            source: apiclient::Error,
        },

        #[snafu(display("Error {} when sending {} to {}: {}", code, method, uri, response_body))]
        APIResponse {
            method: String,
            uri: String,
            code: StatusCode,
            response_body: String,
        },

        #[snafu(display("Logger setup error: {}", source))]
        Logger { source: log::SetLoggerError },

        #[snafu(display("Error parsing TOML user data: {}", source))]
        TOMLUserDataParse { source: toml::de::Error },

        #[snafu(display("User data is not a TOML table"))]
        UserDataNotTomlTable,

        #[snafu(display("TOML user data did not contain 'settings' section"))]
        UserDataMissingSettings,

        #[snafu(display("Error serializing TOML to JSON: {}", source))]
        SettingsToJSON { source: serde_json::error::Error },

        #[snafu(display("Unable to read user data input file '{}': {}", path.display(), source))]
        InputFileRead { path: PathBuf, source: io::Error },

        #[snafu(display("No user data found from provider '{}'", provider))]
        UserDataNotFound {
            provider: &'static str,
            location: String,
        },

        #[snafu(display("Error {} requesting data from IMDS: {}", code, response))]
        IMDSRequest { code: StatusCode, response: String },
    }
}
use error::MoondogError;

/// UserDataProviders must implement this trait. It retrieves the user data (leaving the complexity
/// of this to each different provider) and returns an unparsed and not validated "raw" user data.
trait UserDataProvider {
    /// Retrieve the raw, unparsed user data.
    fn retrieve_user_data(&self) -> Result<RawUserData>;
}

/// Unit struct for AWS so we can implement the UserDataProvider trait.
// This will more than likely not stay a unit struct once we have more things to store about this
// provider.
struct AwsUserDataProvider;

impl AwsUserDataProvider {
    const USER_DATA_ENDPOINT: &'static str = "http://169.254.169.254/latest/user-data";
}

impl UserDataProvider for AwsUserDataProvider {
    fn retrieve_user_data(&self) -> Result<RawUserData> {
        debug!("Requesting user data from IMDS");
        let mut response =
            reqwest::get(Self::USER_DATA_ENDPOINT).context(error::UserDataRequest {
                uri: Self::USER_DATA_ENDPOINT,
            })?;
        trace!("IMDS response: {:?}", &response);

        match response.status() {
            StatusCode::OK => {
                info!("User data found");
                let raw_data = response.text().context(error::UserDataRequest {
                    uri: Self::USER_DATA_ENDPOINT,
                })?;
                trace!("IMDS response text: {:?}", &raw_data);

                Ok(RawUserData::new(raw_data))
            }

            // IMDS doesn't even include a user data endpoint
            // if no user data is given, so we get a 404
            StatusCode::NOT_FOUND => error::UserDataNotFound {
                provider: "IMDS",
                location: Self::USER_DATA_ENDPOINT,
            }
            .fail(),

            code @ _ => error::IMDSRequest {
                code: code,
                response: response.text().context(error::UserDataResponse {
                    code: code,
                    uri: Self::USER_DATA_ENDPOINT,
                })?,
            }
            .fail(),
        }
    }
}

/// Retrieves user data from a known file.  Useful for testing, or simpler providers that store
/// user data on disk.
struct FileUserDataProvider;

impl FileUserDataProvider {
    const USER_DATA_INPUT_FILE: &'static str = "/etc/moondog/input";
}

impl UserDataProvider for FileUserDataProvider {
    fn retrieve_user_data(&self) -> Result<RawUserData> {
        debug!("Reading user data input file");
        let contents =
            fs::read_to_string(Self::USER_DATA_INPUT_FILE).context(error::InputFileRead {
                path: Self::USER_DATA_INPUT_FILE,
            })?;
        trace!("Raw file contents: {:?}", &contents);

        Ok(RawUserData::new(contents))
    }
}

/// This function determines which provider we're currently running on.
fn find_provider() -> Result<Box<dyn UserDataProvider>> {
    // FIXME We need to decide what we're going to do with this
    // in the future. If the user data file exists at a location on disk,
    // use it by default as the UserDataProvider.
    if Path::new(FileUserDataProvider::USER_DATA_INPUT_FILE).exists() {
        info!(
            "User data file found at {}, using it",
            &FileUserDataProvider::USER_DATA_INPUT_FILE
        );
        Ok(Box::new(FileUserDataProvider))
    } else {
        info!("Running on AWS: Using IMDS for user data");
        Ok(Box::new(AwsUserDataProvider))
    }
}

/// This struct contains the raw and unparsed user data retrieved from the UserDataProvider.
struct RawUserData {
    raw_data: String,
}

impl RawUserData {
    fn new(raw_data: String) -> RawUserData {
        RawUserData { raw_data }
    }

    // This function should account for multipart data in the future.  The question is what it will
    // return if we plan on supporting more than just TOML.  A Vec of members of an Enum?
    /// Returns the "settings" table from the input TOML, if any.
    fn settings(&self) -> Result<impl Serialize> {
        let mut val: toml::Value =
            toml::from_str(&self.raw_data).context(error::TOMLUserDataParse)?;
        let table = val.as_table_mut().context(error::UserDataNotTomlTable)?;
        table
            .remove("settings")
            .context(error::UserDataMissingSettings)
    }
}

/// Store the args we receive on the command line
struct Args {
    verbosity: usize,
    socket_path: String,
}

/// Print a usage message in the event a bad arg is passed
fn usage() -> ! {
    let program_name = env::args().next().unwrap_or_else(|| "program".to_string());
    eprintln!(
        r"Usage: {}
            [ --socket-path PATH ]
            [ --verbose --verbose ... ]

    Socket path defaults to {}",
        program_name, DEFAULT_API_SOCKET,
    );
    process::exit(2);
}

/// Prints a more specific message before exiting through usage().
fn usage_msg<S: AsRef<str>>(msg: S) -> ! {
    eprintln!("{}\n", msg.as_ref());
    usage();
}

/// Parse the args to the program and return an Args struct
fn parse_args(args: env::Args) -> Args {
    let mut socket_path = None;
    let mut verbosity = 2;

    let mut iter = args.skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_ref() {
            "--socket-path" => {
                socket_path = Some(
                    iter.next()
                        .unwrap_or_else(|| usage_msg("Did not give argument to --socket-path")),
                )
            }

            "-v" | "--verbose" => verbosity += 1,
            _ => usage(),
        }
    }

    Args {
        socket_path: socket_path.unwrap_or_else(|| DEFAULT_API_SOCKET.to_string()),
        verbosity,
    }
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

    info!("Moondog started");

    // Figure out the current provider
    info!("Detecting user data provider");
    let user_data_provider = find_provider()?;

    // Query the raw data using the method provided by the
    // UserDataProvider trait
    info!("Retrieving user data");
    let raw_user_data = match user_data_provider.retrieve_user_data() {
        Ok(raw_ud) => raw_ud,
        Err(err) => match err {
            error::MoondogError::UserDataNotFound { .. } => {
                warn!("{}", err);
                process::exit(0)
            }
            _ => {
                error!("Error retrieving user data, exiting: {:?}", err);
                process::exit(1)
            }
        },
    };

    // Decode the user data into a generic toml Value
    info!("Parsing TOML user data");
    let user_settings = raw_user_data.settings()?;

    // Serialize the TOML Value into JSON
    info!("Serializing settings to JSON for API request");
    let request_body = serde_json::to_string(&user_settings).context(error::SettingsToJSON)?;
    trace!("API request body: {:?}", request_body);

    // Create an HTTP client and PATCH the JSON
    info!("Sending user data to the API");
    let (code, response_body) = apiclient::raw_request(
        &args.socket_path,
        API_SETTINGS_URI,
        "PATCH",
        Some(request_body),
    )
    .context(error::APIRequest {
        method: "PATCH",
        uri: API_SETTINGS_URI,
    })?;
    ensure!(
        code.is_success(),
        error::APIResponse {
            method: "PATCH",
            uri: API_SETTINGS_URI,
            code,
            response_body,
        }
    );

    fs::write(MARKER_FILE, "").unwrap_or_else(|e| {
        warn!(
            "Failed to create marker file {}, may unexpectedly run again: {}",
            MARKER_FILE, e
        )
    });

    Ok(())
}
