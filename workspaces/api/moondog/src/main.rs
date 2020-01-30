/*!
# Introduction

moondog sends provider-specific platform data to the Thar API.

For most providers this means configuration from user data and platform metadata, taken from
something like an instance metadata service.

Currently, Amazon EC2 is supported through the IMDSv1 HTTP API.  Data will be taken from files in
/etc/moondog instead, if available, for testing purposes.
*/

#![deny(rust_2018_idioms)]

#[macro_use]
extern crate log;

use http::StatusCode;
use serde::Serialize;
use serde_json::json;
use simplelog::{Config as LogConfig, LevelFilter, TermLogger, TerminalMode};
use snafu::{ensure, OptionExt, ResultExt};
use std::path::Path;
use std::str::FromStr;
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

    // Taken from pluto.
    // Extracts the status code from a reqwest::Error and converts it to a string to be displayed
    fn get_bad_status_code(source: &reqwest::Error) -> String {
        source
            .status()
            .as_ref()
            .map(|i| i.as_str())
            .unwrap_or("Unknown")
            .to_string()
    }

    use std::io;
    use std::path::PathBuf;

    #[derive(Debug, Snafu)]
    #[snafu(visibility = "pub(super)")]
    pub(super) enum MoondogError {
        #[snafu(display("Error {}ing '{}': {}", method, uri, source))]
        Request {
            method: String,
            uri: String,
            source: reqwest::Error,
        },

        #[snafu(display("Response '{}' from '{}': {}", get_bad_status_code(&source), uri, source))]
        BadResponse { uri: String, source: reqwest::Error },

        #[snafu(display("Error {}ing '{}': {}", method, uri, source))]
        APIRequest {
            method: String,
            uri: String,
            source: apiclient::Error,
        },

        #[snafu(display("Error {} when {}ing '{}': {}", code, method, uri, response_body))]
        Response {
            method: String,
            uri: String,
            code: StatusCode,
            response_body: String,
        },

        #[snafu(display(
            "Unable to read response body when {}ing '{}' (code {}) - {}",
            method,
            uri,
            code,
            source
        ))]
        ResponseBody {
            method: String,
            uri: String,
            code: StatusCode,
            source: reqwest::Error,
        },

        #[snafu(display("Error parsing TOML user data: {}", source))]
        TOMLUserDataParse { source: toml::de::Error },

        #[snafu(display("Data is not a TOML table"))]
        UserDataNotTomlTable,

        #[snafu(display("TOML data did not contain 'settings' section"))]
        UserDataMissingSettings,

        #[snafu(display("Error serializing TOML to JSON: {}", source))]
        SettingsToJSON { source: serde_json::error::Error },

        #[snafu(display("Error deserializing from JSON: {}", source))]
        DeserializeJson { source: serde_json::error::Error },

        #[snafu(display("Unable to read input file '{}': {}", path.display(), source))]
        InputFileRead { path: PathBuf, source: io::Error },

        #[snafu(display("Instance identity document missing {}", missing))]
        IdentityDocMissingData { missing: String },

        #[snafu(display("Logger setup error: {}", source))]
        Logger { source: simplelog::TermLogError },
    }
}
use error::MoondogError;

/// Support for new platforms can be added by implementing this trait.
trait PlatformDataProvider {
    /// You should return a list of SettingsJson, representing the settings changes you want to
    /// send to the API.
    ///
    /// This is a list so that handling multiple data sources within a platform can feel more
    /// natural; you can also send all changes in one entry if you like.
    fn platform_data(&self) -> Result<Vec<SettingsJson>>;
}

/// Unit struct for AWS so we can implement the PlatformDataProvider trait.
struct AwsDataProvider;

impl AwsDataProvider {
    // Currently only able to get fetch session tokens from `latest`
    // FIXME Pin to a date version that supports IMDSv2 once such a date version is available.
    const IMDS_TOKEN_ENDPOINT: &'static str = "http://169.254.169.254/latest/api/token";

    const USER_DATA_FILE: &'static str = "/etc/moondog/user-data";
    const USER_DATA_ENDPOINT: &'static str = "http://169.254.169.254/2018-09-24/user-data";
    const IDENTITY_DOCUMENT_FILE: &'static str = "/etc/moondog/identity-document";
    const IDENTITY_DOCUMENT_ENDPOINT: &'static str =
        "http://169.254.169.254/2018-09-24/dynamic/instance-identity/document";

    /// Helper to fetch an IMDSv2 session token that is valid for 60 seconds.
    fn fetch_imds_session_token(client: &reqwest::Client) -> Result<String> {
        let uri = Self::IMDS_TOKEN_ENDPOINT;
        let mut response = client
            .put(uri)
            .header("X-aws-ec2-metadata-token-ttl-seconds", "60")
            .send()
            .context(error::Request { method: "PUT", uri })?
            .error_for_status()
            .context(error::BadResponse { uri })?;
        response.text().context(error::ResponseBody {
            method: "PUT",
            uri,
            code: response.status(),
        })
    }

    /// Helper to fetch data from IMDS, preferring an override file if present.
    ///
    /// IMDS returns a 404 if no user data was given, for example; we return Ok(None) to represent
    /// this, otherwise Ok(Some(body)) with the response body.
    fn fetch_imds(
        file: &str,
        client: &reqwest::Client,
        session_token: &str,
        uri: &str,
        description: &str,
    ) -> Result<Option<String>> {
        if Path::new(file).exists() {
            info!("{} file found at {}, using it", description, file);
            return Ok(Some(
                fs::read_to_string(file).context(error::InputFileRead { path: file })?,
            ));
        }
        debug!("Requesting {} from {}", description, uri);
        let mut response = client
            .get(uri)
            .header("X-aws-ec2-metadata-token", session_token)
            .send()
            .context(error::Request { method: "GET", uri })?;
        trace!("IMDS response: {:?}", &response);

        match response.status() {
            code @ StatusCode::OK => {
                info!("Received {}", description);
                let response_body = response.text().context(error::ResponseBody {
                    method: "GET",
                    uri,
                    code,
                })?;
                trace!("Response text: {:?}", &response_body);

                Ok(Some(response_body))
            }

            // IMDS returns 404 if no user data is given, or if IMDS is disabled, for example
            StatusCode::NOT_FOUND => Ok(None),

            code @ _ => {
                let response_body = response.text().context(error::ResponseBody {
                    method: "GET",
                    uri,
                    code,
                })?;
                trace!("Response text: {:?}", &response_body);

                error::Response {
                    method: "GET",
                    uri,
                    code,
                    response_body,
                }
                .fail()
            }
        }
    }

    /// Fetches user data, which is expected to be in TOML form and contain a `[settings]` section,
    /// returning a SettingsJson representing the inside of that section.
    fn user_data(client: &reqwest::Client, session_token: &str) -> Result<Option<SettingsJson>> {
        let desc = "user data";
        let uri = Self::USER_DATA_ENDPOINT;
        let file = Self::USER_DATA_FILE;

        let user_data_str = match Self::fetch_imds(file, client, session_token, uri, desc) {
            Err(e) => return Err(e),
            Ok(None) => return Ok(None),
            Ok(Some(s)) => s,
        };
        trace!("Received user data: {}", user_data_str);

        // Remove outer "settings" layer before sending to API
        let mut val: toml::Value =
            toml::from_str(&user_data_str).context(error::TOMLUserDataParse)?;
        let table = val.as_table_mut().context(error::UserDataNotTomlTable)?;
        let inner = table
            .remove("settings")
            .context(error::UserDataMissingSettings)?;

        SettingsJson::from_val(&inner, desc).map(|s| Some(s))
    }

    /// Fetches the instance identity, returning a SettingsJson representing the values from the
    /// document which we'd like to send to the API - currently just region.
    fn identity_document(
        client: &reqwest::Client,
        session_token: &str,
    ) -> Result<Option<SettingsJson>> {
        let desc = "instance identity document";
        let uri = Self::IDENTITY_DOCUMENT_ENDPOINT;
        let file = Self::IDENTITY_DOCUMENT_FILE;

        let iid_str = match Self::fetch_imds(file, client, session_token, uri, desc) {
            Err(e) => return Err(e),
            Ok(None) => return Ok(None),
            Ok(Some(s)) => s,
        };
        trace!("Received instance identity document: {}", iid_str);

        // Grab region from instance identity document.
        let iid: serde_json::Value =
            serde_json::from_str(&iid_str).context(error::DeserializeJson)?;
        let region = iid
            .get("region")
            .context(error::IdentityDocMissingData { missing: "region" })?;
        let val = json!({ "aws": {"region": region} });

        SettingsJson::from_val(&val, desc).map(|s| Some(s))
    }
}

impl PlatformDataProvider for AwsDataProvider {
    /// Return settings changes from the instance identity document and user data.
    fn platform_data(&self) -> Result<Vec<SettingsJson>> {
        let mut output = Vec::new();
        let client = reqwest::Client::new();

        let session_token = Self::fetch_imds_session_token(&client)?;

        // Instance identity doc first, so the user has a chance to override
        match Self::identity_document(&client, &session_token) {
            Err(e) => return Err(e),
            Ok(None) => warn!("No instance identity document found."),
            Ok(Some(s)) => output.push(s),
        }

        // Optional user-specified configuration / overrides
        match Self::user_data(&client, &session_token) {
            Err(e) => return Err(e),
            Ok(None) => warn!("No user data found."),
            Ok(Some(s)) => output.push(s),
        }

        Ok(output)
    }
}

/// This function determines which provider we're currently running on.
fn find_provider() -> Result<Box<dyn PlatformDataProvider>> {
    // FIXME: We need to decide what we're going to do with this in the future; ask each
    // provider if they should be used?  In what order?
    Ok(Box::new(AwsDataProvider))
}

/// SettingsJson represents a change that a provider would like to make in the API.
#[derive(Debug)]
struct SettingsJson {
    json: String,
    desc: String,
}

impl SettingsJson {
    /// Construct a SettingsJson from a serializable object and a description of that object,
    /// which is used for logging.
    ///
    /// The serializable object is typically something like a toml::Value or serde_json::Value,
    /// since they can be easily deserialized from text input in the platform, and manipulated as
    /// desired.
    fn from_val<S>(data: &impl Serialize, desc: S) -> Result<Self>
    where
        S: Into<String>,
    {
        Ok(Self {
            json: serde_json::to_string(&data).context(error::SettingsToJSON)?,
            desc: desc.into(),
        })
    }
}

/// Store the args we receive on the command line
#[derive(Debug)]
struct Args {
    log_level: LevelFilter,
    socket_path: String,
}

/// Print a usage message in the event a bad arg is passed
fn usage() -> ! {
    let program_name = env::args().next().unwrap_or_else(|| "program".to_string());
    eprintln!(
        r"Usage: {}
            [ --socket-path PATH ]
            [ --log-level trace|debug|info|warn|error ]

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
    let mut log_level = None;
    let mut socket_path = None;

    let mut iter = args.skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_ref() {
            "--socket-path" => {
                socket_path = Some(
                    iter.next()
                        .unwrap_or_else(|| usage_msg("Did not give argument to --socket-path")),
                )
            }

            "--log-level" => {
                let log_level_str = iter
                    .next()
                    .unwrap_or_else(|| usage_msg("Did not give argument to --log-level"));
                log_level = Some(LevelFilter::from_str(&log_level_str).unwrap_or_else(|_| {
                    usage_msg(format!("Invalid log level '{}'", log_level_str))
                }));
            }

            _ => usage(),
        }
    }

    Args {
        log_level: log_level.unwrap_or_else(|| LevelFilter::Info),
        socket_path: socket_path.unwrap_or_else(|| DEFAULT_API_SOCKET.to_string()),
    }
}

fn run() -> Result<()> {
    // Parse and store the args passed to the program
    let args = parse_args(env::args());

    // TerminalMode::Mixed will send errors to stderr and anything less to stdout.
    TermLogger::init(args.log_level, LogConfig::default(), TerminalMode::Mixed)
        .context(error::Logger)?;

    info!("Moondog started");

    // Figure out the current provider
    info!("Detecting platform data provider");
    let data_provider = find_provider()?;

    info!("Retrieving platform-specific data");
    for settings_json in data_provider.platform_data()? {
        info!("Sending {} to API", settings_json.desc);
        trace!("Request body: {}", settings_json.json);
        let (code, response_body) = apiclient::raw_request(
            &args.socket_path,
            API_SETTINGS_URI,
            "PATCH",
            Some(settings_json.json),
        )
        .context(error::APIRequest {
            method: "PATCH",
            uri: API_SETTINGS_URI,
        })?;
        ensure!(
            code.is_success(),
            error::Response {
                method: "PATCH",
                uri: API_SETTINGS_URI,
                code,
                response_body,
            }
        );
    }

    fs::write(MARKER_FILE, "").unwrap_or_else(|e| {
        warn!(
            "Failed to create marker file {}, may unexpectedly run again: {}",
            MARKER_FILE, e
        )
    });

    Ok(())
}

// Returning a Result from main makes it print a Debug representation of the error, but with Snafu
// we have nice Display representations of the error, so we wrap "main" (run) and print any error.
// https://github.com/shepmaster/snafu/issues/110
fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        process::exit(1);
    }
}
