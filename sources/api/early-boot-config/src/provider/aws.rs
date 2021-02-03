//! The aws module implements the `PlatformDataProvider` trait for gathering userdata on AWS.

use super::{PlatformDataProvider, SettingsJson};
use http::StatusCode;
use reqwest::blocking::Client;
use serde_json::json;
use snafu::{OptionExt, ResultExt};
use std::fs;
use std::path::Path;

/// Unit struct for AWS so we can implement the PlatformDataProvider trait.
pub(crate) struct AwsDataProvider;

impl AwsDataProvider {
    // Currently only able to get fetch session tokens from `latest`
    // FIXME Pin to a date version that supports IMDSv2 once such a date version is available.
    const IMDS_TOKEN_ENDPOINT: &'static str = "http://169.254.169.254/latest/api/token";

    const USER_DATA_ENDPOINT: &'static str = "http://169.254.169.254/2018-09-24/user-data";
    const IDENTITY_DOCUMENT_FILE: &'static str = "/etc/early-boot-config/identity-document";
    const IDENTITY_DOCUMENT_ENDPOINT: &'static str =
        "http://169.254.169.254/2018-09-24/dynamic/instance-identity/document";

    /// Helper to fetch an IMDSv2 session token that is valid for 60 seconds.
    fn fetch_imds_session_token(client: &Client) -> Result<String> {
        let uri = Self::IMDS_TOKEN_ENDPOINT;
        let response = client
            .put(uri)
            .header("X-aws-ec2-metadata-token-ttl-seconds", "60")
            .send()
            .context(error::Request { method: "PUT", uri })?
            .error_for_status()
            .context(error::BadResponse { uri })?;
        let code = response.status();
        response.text().context(error::ResponseBody {
            method: "PUT",
            uri,
            code,
        })
    }

    /// Helper to fetch data from IMDS, preferring an override file if present.
    ///
    /// IMDS returns a 404 if no user data was given, for example; we return Ok(None) to represent
    /// this, otherwise Ok(Some(body)) with the response body.
    fn fetch_imds(
        client: &Client,
        session_token: &str,
        uri: &str,
        description: &str,
    ) -> Result<Option<String>> {
        debug!("Requesting {} from {}", description, uri);
        let response = client
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
    fn user_data(client: &Client, session_token: &str) -> Result<Option<SettingsJson>> {
        let desc = "user data";
        let uri = Self::USER_DATA_ENDPOINT;

        let user_data_str = match Self::fetch_imds(client, session_token, uri, desc) {
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

        let json = SettingsJson::from_val(&inner, desc).context(error::SettingsToJSON)?;
        Ok(Some(json))
    }

    /// Fetches the instance identity, returning a SettingsJson representing the values from the
    /// document which we'd like to send to the API - currently just region.
    fn identity_document(client: &Client, session_token: &str) -> Result<Option<SettingsJson>> {
        let desc = "instance identity document";
        let uri = Self::IDENTITY_DOCUMENT_ENDPOINT;
        let file = Self::IDENTITY_DOCUMENT_FILE;

        let iid_str = if Path::new(file).exists() {
            info!("{} found at {}, using it", desc, file);
            fs::read_to_string(file).context(error::InputFileRead { path: file })?
        } else {
            match Self::fetch_imds(client, session_token, uri, desc) {
                Err(e) => return Err(e),
                Ok(None) => return Ok(None),
                Ok(Some(s)) => s,
            }
        };
        trace!("Received instance identity document: {}", iid_str);

        // Grab region from instance identity document.
        let iid: serde_json::Value =
            serde_json::from_str(&iid_str).context(error::DeserializeJson)?;
        let region = iid
            .get("region")
            .context(error::IdentityDocMissingData { missing: "region" })?;
        let val = json!({ "aws": {"region": region} });

        let json = SettingsJson::from_val(&val, desc).context(error::SettingsToJSON)?;
        Ok(Some(json))
    }
}

impl PlatformDataProvider for AwsDataProvider {
    /// Return settings changes from the instance identity document and user data.
    fn platform_data(&self) -> std::result::Result<Vec<SettingsJson>, Box<dyn std::error::Error>> {
        let mut output = Vec::new();
        let client = Client::new();

        let session_token = Self::fetch_imds_session_token(&client)?;

        // Instance identity doc first, so the user has a chance to override
        match Self::identity_document(&client, &session_token) {
            Err(e) => return Err(e).map_err(Into::into),
            Ok(None) => warn!("No instance identity document found."),
            Ok(Some(s)) => output.push(s),
        }

        // Optional user-specified configuration / overrides
        match Self::user_data(&client, &session_token) {
            Err(e) => return Err(e).map_err(Into::into),
            Ok(None) => warn!("No user data found."),
            Ok(Some(s)) => output.push(s),
        }

        Ok(output)
    }
}

mod error {
    use http::StatusCode;
    use snafu::Snafu;
    use std::io;
    use std::path::PathBuf;

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

    #[derive(Debug, Snafu)]
    #[snafu(visibility = "pub(super)")]
    pub(crate) enum Error {
        #[snafu(display("Response '{}' from '{}': {}", get_bad_status_code(&source), uri, source))]
        BadResponse { uri: String, source: reqwest::Error },

        #[snafu(display("Error deserializing from JSON: {}", source))]
        DeserializeJson { source: serde_json::error::Error },

        #[snafu(display("Instance identity document missing {}", missing))]
        IdentityDocMissingData { missing: String },

        #[snafu(display("Unable to read input file '{}': {}", path.display(), source))]
        InputFileRead { path: PathBuf, source: io::Error },

        #[snafu(display("Error {}ing '{}': {}", method, uri, source))]
        Request {
            method: String,
            uri: String,
            source: reqwest::Error,
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

        #[snafu(display("Error serializing TOML to JSON: {}", source))]
        SettingsToJSON { source: serde_json::error::Error },

        #[snafu(display("Error parsing TOML user data: {}", source))]
        TOMLUserDataParse { source: toml::de::Error },

        #[snafu(display("TOML data did not contain 'settings' section"))]
        UserDataMissingSettings,

        #[snafu(display("Data is not a TOML table"))]
        UserDataNotTomlTable,
    }
}

type Result<T> = std::result::Result<T, error::Error>;
