/*!
# Introduction

moondog is a minimal userdata agent.

It accepts TOML-formatted settings from a userdata provider such as an instance metadata service.
These are sent to a known Thar API server endpoint, then committed.

Currently, AWS userdata support is implemented.
Userdata can also be retrieved from a file for testing.
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

/// Potential errors during userdata management.
#[derive(Debug, Error)]
enum MoondogError {
    // Error making network call with reqwest
    NetworkRequest(reqwest::Error),
    // Logger setup error
    Logger(log::SetLoggerError),
    // Error parsing TOML userdata
    TOMLUserdataParse(toml::de::Error),
    // Error serializing TOML to JSON
    TOMLtoJSON(serde_json::error::Error),
    // Unable to read userdata input file
    InputFileRead(std::io::Error),
    #[error(msg_embedded, no_from, non_std)]
    // No userdata found
    UserdataNotFound(String),
    #[error(msg_embedded, no_from, non_std)]
    // Unknown error requesting data from IMDS
    IMDSRequest(String),
}

/// UserDataProviders must implement this trait. It retrieves the userdata (leaving the complexity
/// of this to each different provider) and returns an unparsed and not validated "raw" userdata.
trait UserDataProvider {
    /// Retrieve the raw, unparsed userdata.
    fn retrieve_userdata(&self) -> Result<RawUserData>;
}

/// Unit struct for AWS so we can implement the UserDataProvider trait.
// This will more than likely not stay a unit struct once we have more things to store about this
// provider.
struct AwsUserDataProvider;

impl AwsUserDataProvider {
    const USERDATA_ENDPOINT: &'static str = "http://169.254.169.254/latest/user-data";
}

impl UserDataProvider for AwsUserDataProvider {
    fn retrieve_userdata(&self) -> Result<RawUserData> {
        debug!("Requesting userdata from IMDS");
        let mut response = reqwest::get(Self::USERDATA_ENDPOINT)?;
        trace!("IMDS response: {:?}", &response);

        match response.status() {
            StatusCode::OK => {
                info!("Userdata found");
                let raw_data = response.text()?;
                trace!("IMDS response text: {:?}", &raw_data);

                Ok(RawUserData::new(raw_data))
            }
            // IMDS doesn't even include a user-data endpoint
            // if no userdata is given, so we get a 404
            StatusCode::NOT_FOUND => {
                Err(MoondogError::UserdataNotFound(
                    "Userdata not found in IMDS".to_string(),
                ))
            }
            _ => {
                Err(MoondogError::IMDSRequest(format!(
                    "Unknown err: {:?}",
                    response.text()
                )))
            }
        }
    }
}

/// Retrieves userdata from a known file.  Useful for testing, or simpler providers that store
/// userdata on disk.
struct FileUserDataProvider;

impl FileUserDataProvider {
    const USERDATA_INPUT_FILE: &'static str = "/etc/moondog/input";
}

impl UserDataProvider for FileUserDataProvider {
    fn retrieve_userdata(&self) -> Result<RawUserData> {
        debug!("Reading userdata input file");
        let contents = fs::read_to_string(Self::USERDATA_INPUT_FILE)?;
        trace!("Raw file contents: {:?}", &contents);

        Ok(RawUserData::new(contents))
    }
}

/// This function determines which provider we're currently running on.
fn find_provider() -> Result<Box<dyn UserDataProvider>> {
    // FIXME We need to decide what we're going to do with this
    // in the future. If the userdata file exists at a location on disk,
    // use it by default as the UserDataProvider.
    if Path::new(FileUserDataProvider::USERDATA_INPUT_FILE).exists() {
        info!(
            "Userdata file found at {}, using it",
            &FileUserDataProvider::USERDATA_INPUT_FILE
        );
        Ok(Box::new(FileUserDataProvider))
    } else {
        info!("Running on AWS: Using IMDS for userdata");
        Ok(Box::new(AwsUserDataProvider))
    }
}

/// This struct contains the raw and unparsed userdata retrieved from the UserDataProvider.
struct RawUserData {
    raw_data: String,
}

impl RawUserData {
    fn new(raw_data: String) -> RawUserData {
        RawUserData { raw_data }
    }

    // This function should account for multipart data in the future.  The question is what it will
    // return if we plan on supporting more than just TOML.  A Vec of members of an Enum?
    /// Parses raw userdata as TOML.  This is a syntactic check only - it doesn't check if the
    /// values are valid to send to the API.
    fn decode(&self) -> Result<impl Serialize> {
        debug!("Parsing TOML from raw userdata");
        let userdata: toml::Value = toml::from_str(&self.raw_data)?;
        trace!("TOML userdata: {:?}", &userdata);

        Ok(userdata)
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
    info!("Detecting userdata provider");
    let userdata_provider = find_provider()?;

    // Query the raw data using the method provided by the
    // UserDataProvider trait
    info!("Retrieving userdata");
    let raw_userdata = match userdata_provider.retrieve_userdata() {
        Ok(raw_ud) => raw_ud,
        Err(err) => match err {
            MoondogError::UserdataNotFound(msg) => {
                warn!("No userdata found: {}", &msg);
                process::exit(0)
            }
            _ => {
                error!("Error retrieving userdata, exiting: {:?}", err);
                process::exit(1)
            }
        },
    };

    // Decode the userdata into a generic toml Value
    info!("Parsing TOML userdata");
    let userdata = raw_userdata.decode()?;

    // Serialize the TOML Value into JSON
    info!("Serializing userdata to JSON for API request");
    let request_body = serde_json::to_string(&userdata)?;
    trace!("API request body: {:?}", request_body);

    // Create an HTTP client and PATCH the JSON
    info!("POST-ing userdata to the API");
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
