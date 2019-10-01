/*!
# Introduction

settings-committer runs on boot after any services that can update
settings. It logs any pending settings, then commits them to live.

*/
#![deny(rust_2018_idioms)]

#[macro_use]
extern crate tracing;

use std::{collections::HashMap, env, process};

use tracing_subscriber::{
    FmtSubscriber,
    filter::{EnvFilter, LevelFilter},
};

use snafu::{ensure, ResultExt};

const DEFAULT_API_SOCKET: &str = "/run/api.sock";
const API_PENDING_URI: &str = "/settings/pending";
const API_COMMIT_URI: &str = "/settings/commit";

type Result<T> = std::result::Result<T, error::SettingsCommitterError>;

mod error {
    use http::StatusCode;
    use snafu::Snafu;

    /// Potential errors during user data management.
    #[derive(Debug, Snafu)]
    #[snafu(visibility = "pub(super)")]
    pub(super) enum SettingsCommitterError {
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

        #[snafu(display("Failed to parse provided directive: {}", source))]
        TracingDirectiveParse {
            source: tracing_subscriber::filter::LevelParseError,
        },
    }
}

/// Checks pending settings and logs them. We don't want to prevent a
/// commit if there's a blip in retrieval or parsing of the pending
/// settings.  We know the system won't be functional without a commit,
/// but we can live without logging what was committed.
fn check_pending_settings<S: AsRef<str>>(socket_path: S) {
    let uri = API_PENDING_URI;

    debug!("GET-ing {} to determine if there are pending settings", uri);
    let get_result = apiclient::raw_request(socket_path.as_ref(), uri, "GET", None);
    let response_body = match get_result {
        Ok((code, response_body)) => {
            if !code.is_success() {
                warn!(
                    "Got {} when sending GET to {}: {}",
                    code, uri, response_body
                );
                return;
            }
            response_body
        }
        Err(err) => {
            warn!("Failed to GET pending settings from {}: {}", uri, err);
            return;
        }
    };

    let pending_result: serde_json::Result<HashMap<String, serde_json::Value>> =
        serde_json::from_str(&response_body);
    match pending_result {
        Ok(pending) => {
            debug!("Pending settings: {:?}", &pending);
        }
        Err(err) => {
            warn!("Failed to parse response from {}: {}", uri, err);
        }
    }
}

/// Commits pending settings to live.
fn commit_pending_settings<S: AsRef<str>>(socket_path: S) -> Result<()> {
    let uri = API_COMMIT_URI;

    debug!("POST-ing to {} to move pending settings to live", uri);
    let (code, response_body) = apiclient::raw_request(socket_path.as_ref(), uri, "POST", None)
        .context(error::APIRequest {
            method: "POST",
            uri,
        })?;
    ensure!(
        // 422 is returned when there are no pending settings.
        code.is_success() || code == 422,
        error::APIResponse {
            method: "POST",
            uri,
            code,
            response_body,
        }
    );
    Ok(())
}

/// Store the args we receive on the command line
struct Args {
    socket_path: String,
    verbosity: usize,
}

/// Print a usage message in the event a bad arg is passed
fn usage() -> ! {
    let program_name = env::args().next().unwrap_or_else(|| "program".to_string());
    eprintln!(
        r"Usage: {}
            [ --socket-path PATH ]
            [ --verbose --verbose ... ]
    Socket path defaults to {}",
        program_name, DEFAULT_API_SOCKET
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
    let mut verbosity = 3;

    let mut iter = args.skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_ref() {
            "-v" | "--verbose" => verbosity += 1,
            "--socket-path" => {
                socket_path = Some(
                    iter.next()
                        .unwrap_or_else(|| usage_msg("Did not give argument to --socket-path")),
                )
            }
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
    let level: LevelFilter = args.verbosity.to_string().parse().context(error::TracingDirectiveParse)?;
    let filter = EnvFilter::from_default_env().add_directive(level.into());
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .finish();
    // Start the logger
    tracing::subscriber::set_global_default(subscriber).expect("setting tracing default failed");

    info!("Checking pending settings.");
    check_pending_settings(&args.socket_path);

    info!("Committing settings.");
    commit_pending_settings(&args.socket_path)?;

    Ok(())
}
