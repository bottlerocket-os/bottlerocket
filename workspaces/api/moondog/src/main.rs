/*!
# Introduction

moondog is a minimal user data agent.

It accepts TOML-formatted settings from a user data provider such as an instance metadata service.
These are sent to a known Thar API server endpoint, then committed.

Currently, Amazon EC2 user data support is implemented.
User data can also be retrieved from a file for testing.
*/

#[macro_use]
extern crate log;

use std::fs;
use std::path::Path;
use std::process;

use reqwest::StatusCode;
use serde::Serialize;
use snafu::ResultExt;

// TODO
// Tests!

// FIXME Get these from configuration in the future
const API_SETTINGS_URI: &str = "http://localhost:4242/settings";
const API_COMMIT_URI: &str = "http://localhost:4242/settings/commit";

type Result<T> = std::result::Result<T, MoondogError>;

mod error {
    use reqwest::StatusCode;
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

        #[snafu(display("Error sending {} to '{}': {}", method, uri, source))]
        APIRequest {
            method: &'static str,
            uri: String,
            source: reqwest::Error,
        },

        #[snafu(display("Error updating settings through '{}': {}", uri, source))]
        UpdatingAPISettings { uri: String, source: reqwest::Error },

        #[snafu(display("Error committing changes to '{}': {}", uri, source))]
        CommittingAPISettings { uri: String, source: reqwest::Error },

        #[snafu(display("Logger setup error: {}", source))]
        Logger { source: log::SetLoggerError },

        #[snafu(display("Error parsing TOML user data: {}", source))]
        TOMLUserDataParse { source: toml::de::Error },

        #[snafu(display("Error serializing TOML to JSON: {}", source))]
        TOMLtoJSON { source: serde_json::error::Error },

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
    /// Parses raw user data as TOML.  This is a syntactic check only - it doesn't check if the
    /// values are valid to send to the API.
    fn decode(&self) -> Result<impl Serialize> {
        debug!("Parsing TOML from raw user data");
        let user_data: toml::Value =
            toml::from_str(&self.raw_data).context(error::TOMLUserDataParse)?;
        trace!("TOML user data: {:?}", &user_data);

        Ok(user_data)
    }
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
    let user_data = raw_user_data.decode()?;

    // Serialize the TOML Value into JSON
    info!("Serializing user data to JSON for API request");
    let request_body = serde_json::to_string(&user_data).context(error::TOMLtoJSON)?;
    trace!("API request body: {:?}", request_body);

    // Create an HTTP client and PATCH the JSON
    info!("Sending user data to the API");
    let client = reqwest::Client::new();
    client
        .patch(API_SETTINGS_URI)
        .body(request_body)
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
    info!("POST-ing to /commit to finalize the changes");
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
