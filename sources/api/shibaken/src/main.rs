/*!
# Introduction

shibaken is called by sundog as a setting generator.

shibaken will fetch and populate the admin container's user-data with authorized ssh keys from the
AWS instance metadata service (IMDS).

(The name "shibaken" comes from the fact that Shiba are small, but agile, hunting dogs.)
*/

#![deny(rust_2018_idioms)]

use log::{debug, info, warn};
use reqwest::blocking::Client;
use serde::Serialize;
use simplelog::{ColorChoice, Config as LogConfig, LevelFilter, TermLogger, TerminalMode};
use snafu::{OptionExt, ResultExt};
use std::str::FromStr;
use std::{env, process};

// Instance Meta Data Service.
//
// Currently only able to get fetch session tokens from `latest`
// FIXME Pin to a date version that supports IMDSv2 once such a date version is available.
const IMDS_PUBLIC_KEY_BASE_URI: &str = "http://169.254.169.254/latest/meta-data/public-keys";
const IMDS_SESSION_TOKEN_URI: &str = "http://169.254.169.254/latest/api/token";

#[derive(Serialize)]
struct UserData {
    ssh: Ssh,
}

#[derive(Serialize)]
struct Ssh {
    authorized_keys: Vec<String>,
}
impl UserData {
    fn new(public_keys: Vec<String>) -> Self {
        UserData {
            ssh: Ssh {
                authorized_keys: public_keys,
            },
        }
    }
}

/// Helper to fetch an IMDSv2 session token that is valid for 60 seconds.
fn fetch_imds_session_token(client: &Client) -> Result<String> {
    let uri = IMDS_SESSION_TOKEN_URI;
    let imds_session_token = client
        .put(uri)
        .header("X-aws-ec2-metadata-token-ttl-seconds", "60")
        .send()
        .context(error::ImdsRequest { method: "PUT", uri })?
        .error_for_status()
        .context(error::ImdsResponse { uri })?
        .text()
        .context(error::ImdsText { uri })?;
    Ok(imds_session_token)
}

/// Helper to fetch data from IMDS. Inspired by pluto.
fn fetch_from_imds(client: &Client, uri: &str, session_token: &str) -> Result<Option<String>> {
    let response = client
        .get(uri)
        .header("X-aws-ec2-metadata-token", session_token)
        .send()
        .context(error::ImdsRequest { method: "GET", uri })?;
    if response.status().as_u16() == 404 {
        return Ok(None);
    }
    Ok(Some(
        response
            .error_for_status()
            .context(error::ImdsResponse { uri })?
            .text()
            .context(error::ImdsText { uri })?,
    ))
}

/// Returns a list of public keys.
fn fetch_public_keys_from_imds() -> Result<Vec<String>> {
    info!("Fetching IMDS session token");
    let client = Client::new();
    let imds_session_token = fetch_imds_session_token(&client)?;

    info!("Fetching list of available public keys from IMDS");
    // Returns a list of available public keys as '0=my-public-key'
    let public_key_list = if let Some(public_key_list) =
        fetch_from_imds(&client, IMDS_PUBLIC_KEY_BASE_URI, &imds_session_token)?
    {
        debug!("available public keys '{}'", &public_key_list);
        public_key_list
    } else {
        debug!("no available public keys");
        return Ok(Vec::new());
    };

    info!("Generating uris to fetch text of available public keys");
    let public_key_uris = build_public_key_uris(&public_key_list);

    info!("Fetching public keys from IMDS");
    let mut public_keys = Vec::new();
    for uri in public_key_uris {
        let public_key_text = fetch_from_imds(&client, &uri, &imds_session_token)?
            .context(error::KeyNotFound { uri })?;
        let public_key = public_key_text.trim_end();
        // Simple check to see if the text is probably an ssh key.
        if public_key.starts_with("ssh") {
            debug!("{}", &public_key);
            public_keys.push(public_key.to_string())
        } else {
            warn!(
                "'{}' does not appear to be a valid key. Skipping...",
                &public_key_text
            );
            continue;
        }
    }
    if public_keys.is_empty() {
        warn!("No valid keys found");
    }
    Ok(public_keys)
}

/// Returns a list of public key uris strings for the public keys in IMDS. Since IMDS returns the
/// list of available public keys as '0=my-public-key', we need to strip the index from the list and
/// insert it into the key uri.
fn build_public_key_uris(public_key_list: &str) -> Vec<String> {
    let mut public_key_uris = Vec::new();
    for available_key in public_key_list.lines() {
        let f: Vec<&str> = available_key.split('=').collect();
        // If f[0] isn't a number, then it isn't a valid index.
        if f[0].parse::<u32>().is_ok() {
            let public_key_uri = format!("{}/{}/openssh-key", IMDS_PUBLIC_KEY_BASE_URI, f[0]);
            public_key_uris.push(public_key_uri);
        } else {
            warn!(
                "'{}' does not appear to be a valid index. Skipping...",
                &f[0]
            );
            continue;
        }
    }
    if public_key_uris.is_empty() {
        warn!("No valid key uris found");
    }
    public_key_uris
}

/// Store the args we receive on the command line.
struct Args {
    log_level: LevelFilter,
}

/// Print a usage message in the event a bad arg is passed
fn usage() {
    let program_name = env::args().next().unwrap_or_else(|| "program".to_string());
    eprintln!(
        r"Usage: {}
            [ --log-level trace|debug|info|warn|error ]",
        program_name
    );
}

/// Parse the args to the program and return an Args struct
fn parse_args(args: env::Args) -> Result<Args> {
    let mut log_level = None;

    let mut iter = args.skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_ref() {
            "--log-level" => {
                let log_level_str = iter.next().context(error::Usage {
                    message: "Did not give argument to --log-level",
                })?;
                log_level = Some(
                    LevelFilter::from_str(&log_level_str)
                        .context(error::LogLevel { log_level_str })?,
                );
            }

            x => {
                return error::Usage {
                    message: format!("unexpected argument '{}'", x),
                }
                .fail()
            }
        }
    }

    Ok(Args {
        log_level: log_level.unwrap_or_else(|| LevelFilter::Info),
    })
}

fn run() -> Result<()> {
    let args = parse_args(env::args())?;

    // TerminalMode::Stderr will send all logs to stderr, as sundog only expects the json output of
    // the setting on stdout.
    TermLogger::init(
        args.log_level,
        LogConfig::default(),
        TerminalMode::Stderr,
        ColorChoice::Auto,
    )
    .context(error::Logger)?;

    info!("shibaken started");

    let public_keys = fetch_public_keys_from_imds()?;

    let user_data = UserData::new(public_keys);

    info!("Generating user-data");
    // Serialize user_data to a JSON string that can be read by the admin container.
    let user_data_json = serde_json::to_string(&user_data).context(error::SerializeJson)?;
    debug!("{}", &user_data_json);

    info!("Encoding user-data");
    // admin container user-data must be base64-encoded to be passed through to the admin container
    // using a setting, rather than another arbitrary storage mechanism. This approach allows the
    // user to bypass shibaken and use their own user-data if desired.
    let user_data_base64 = base64::encode(&user_data_json);

    info!("Outputting user-data");
    // sundog expects JSON-serialized output so that many types can be represented, allowing the
    // API model to use more accurate types.
    let output = serde_json::to_string(&user_data_base64).context(error::SerializeJson)?;

    println!("{}", output);

    Ok(())
}

// Returning a Result from main makes it print a Debug representation of the error, but with Snafu
// we have nice Display representations of the error, so we wrap "main" (run) and print any error.
// https://github.com/shepmaster/snafu/issues/110
fn main() {
    if let Err(e) = run() {
        match e {
            error::Error::Usage { .. } => {
                eprintln!("{}", e);
                usage();
                // sundog matches on the exit codes of the setting generators, so we should return 1
                // to make sure that this is treated as a failure.
                process::exit(1);
            }
            _ => {
                eprintln!("{}", e);
                process::exit(1);
            }
        }
    }
}

mod error {
    use snafu::Snafu;
    fn code(source: &reqwest::Error) -> String {
        source
            .status()
            .as_ref()
            .map(|i| i.as_str())
            .unwrap_or("Unknown")
            .to_string()
    }

    #[derive(Debug, Snafu)]
    #[snafu(visibility = "pub(super)")]
    pub(super) enum Error {
        #[snafu(display("Error {}ing '{}': {}", method, uri, source))]
        ImdsRequest {
            method: String,
            uri: String,
            source: reqwest::Error,
        },

        #[snafu(display("Error '{}' from '{}': {}", code(&source), uri, source))]
        ImdsResponse { uri: String, source: reqwest::Error },

        #[snafu(display("Error getting text response from {}: {}", uri, source))]
        ImdsText { uri: String, source: reqwest::Error },

        #[snafu(display("Error retrieving key from {}", uri))]
        KeyNotFound { uri: String },

        #[snafu(display("Logger setup error: {}", source))]
        Logger { source: log::SetLoggerError },

        #[snafu(display("Invalid log level '{}'", log_level_str))]
        LogLevel {
            log_level_str: String,
            source: log::ParseLevelError,
        },

        #[snafu(display("Error serializing to JSON: {}", source))]
        SerializeJson { source: serde_json::error::Error },

        #[snafu(display("{}", message))]
        Usage { message: String },
    }
}
use error::Error;
type Result<T> = std::result::Result<T, Error>;
