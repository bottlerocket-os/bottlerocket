/*!
# Introduction

moondog is a minimal user data agent.

It accepts TOML-formatted settings from a user data provider such as an instance metadata service.
These are sent to a known Thar API server endpoint, then committed.

Currently, Amazon EC2 user data support is implemented.
User data can also be retrieved from a file for testing.
*/

#[macro_use]
extern crate derive_error;
#[macro_use]
extern crate log;

use std::fs;
use std::path::Path;
use std::process;

use reqwest::StatusCode;
use serde::Serialize;

// TODO
// Tests!

// FIXME Get these from configuration in the future
const API_SETTINGS_URI: &str = "http://localhost:4242/settings";
const API_COMMIT_URI: &str = "http://localhost:4242/settings/commit";

type Result<T> = std::result::Result<T, MoondogError>;

/// Potential errors during user data management.
#[derive(Debug, Error)]
enum MoondogError {
    /// Error making network call with reqwest
    NetworkRequest(reqwest::Error),
    /// Logger setup error
    Logger(log::SetLoggerError),
    /// Error parsing TOML user data
    TOMLUserDataParse(toml::de::Error),
    /// Error serializing TOML to JSON
    TOMLtoJSON(serde_json::error::Error),
    /// Unable to read user data input file
    InputFileRead(std::io::Error),
    #[error(msg_embedded, no_from, non_std)]
    /// No user data found
    UserDataNotFound(String),
    #[error(msg_embedded, no_from, non_std)]
    /// Unknown error requesting data from IMDS
    IMDSRequest(String),
}

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
        let mut response = reqwest::get(Self::USER_DATA_ENDPOINT)?;
        trace!("IMDS response: {:?}", &response);

        match response.status() {
            StatusCode::OK => {
                info!("User data found");
                let raw_data = response.text()?;
                trace!("IMDS response text: {:?}", &raw_data);

                Ok(RawUserData::new(raw_data))
            }
            // IMDS doesn't even include a user data endpoint
            // if no user data is given, so we get a 404
            StatusCode::NOT_FOUND => Err(MoondogError::UserDataNotFound(
                "User data not found in IMDS".to_string(),
            )),
            _ => Err(MoondogError::IMDSRequest(format!(
                "Unknown err: {:?}",
                response.text()
            ))),
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
        let contents = fs::read_to_string(Self::USER_DATA_INPUT_FILE)?;
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
        let user_data: toml::Value = toml::from_str(&self.raw_data)?;
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
        .init()?;

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
            MoondogError::UserDataNotFound(msg) => {
                warn!("No user data found: {}", &msg);
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
    let request_body = serde_json::to_string(&user_data)?;
    trace!("API request body: {:?}", request_body);

    // Create an HTTP client and PATCH the JSON
    info!("POST-ing user data to the API");
    let client = reqwest::Client::new();
    client
        .patch(API_SETTINGS_URI)
        .body(request_body)
        .send()?
        .error_for_status()?;

    // POST to /commit to actually make the changes
    info!("POST-ing to /commit to finalize the changes");
    client
        .post(API_COMMIT_URI)
        .body("")
        .send()?
        .error_for_status()?;

    Ok(())
}
