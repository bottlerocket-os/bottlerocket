/*!
# Introduction

settings-committer can be called to commit a pending transaction in the API.
It logs any pending settings, then commits them to live.

By default, it commits the 'bottlerocket-launch' transaction, which is used to organize boot-time services - this program is typically run as a pre-exec command by any services that depend on settings changes from previous services.

The `--transaction` argument can be used to specify another transaction.
*/
#[macro_use]
extern crate log;

use simplelog::{Config as LogConfig, LevelFilter, SimpleLogger};
use snafu::ResultExt;
use std::str::FromStr;
use std::{collections::HashMap, env, process};

const API_PENDING_URI_BASE: &str = "/tx";
const API_COMMIT_URI_BASE: &str = "/tx/commit";

type Result<T> = std::result::Result<T, error::SettingsCommitterError>;

mod error {
    use http::StatusCode;
    use snafu::Snafu;

    /// Potential errors during user data management.
    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub(super) enum SettingsCommitterError {
        #[snafu(display("Error sending {} to {}: {}", method, uri, source))]
        APIRequest {
            method: String,
            uri: String,
            #[snafu(source(from(apiclient::Error, Box::new)))]
            source: Box<apiclient::Error>,
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
    }
}

/// Checks pending settings and logs them. We don't want to prevent a
/// commit if there's a blip in retrieval or parsing of the pending
/// settings.  We know the system won't be functional without a commit,
/// but we can live without logging what was committed.
async fn check_pending_settings<S: AsRef<str>>(socket_path: S, transaction: &str) {
    let uri = format!("{}?tx={}", API_PENDING_URI_BASE, transaction);

    debug!("GET-ing {} to determine if there are pending settings", uri);
    let get_result = apiclient::raw_request(socket_path.as_ref(), &uri, "GET", None).await;
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
            debug!("Pending settings for tx {}: {:?}", transaction, &pending);
        }
        Err(err) => {
            warn!("Failed to parse response from {}: {}", uri, err);
        }
    }
}

/// Commits pending settings to live.
async fn commit_pending_settings<S: AsRef<str>>(socket_path: S, transaction: &str) -> Result<()> {
    let uri = format!("{}?tx={}", API_COMMIT_URI_BASE, transaction);
    debug!("POST-ing to {} to move pending settings to live", uri);

    if let Err(e) = apiclient::raw_request(socket_path.as_ref(), &uri, "POST", None).await {
        match e {
            // Some types of response errors are OK for this use.
            apiclient::Error::ResponseStatus { code, body, .. } => {
                if code.as_u16() == 422 {
                    info!("settings-committer found no settings changes to commit");
                    return Ok(());
                } else {
                    return error::APIResponseSnafu {
                        method: "POST",
                        uri,
                        code,
                        response_body: body,
                    }
                    .fail();
                }
            }
            // Any other type of error means we couldn't even make the request.
            _ => {
                return Err(e).context(error::APIRequestSnafu {
                    method: "POST",
                    uri,
                });
            }
        }
    }
    Ok(())
}

/// Store the args we receive on the command line
struct Args {
    transaction: String,
    log_level: LevelFilter,
    socket_path: String,
}

/// Print a usage message in the event a bad arg is passed
fn usage() -> ! {
    let program_name = env::args().next().unwrap_or_else(|| "program".to_string());
    eprintln!(
        r"Usage: {}
            [ --transaction TX ]
            [ --socket-path PATH ]
            [ --log-level trace|debug|info|warn|error ]

    Transaction defaults to {}
    Socket path defaults to {}",
        program_name,
        constants::LAUNCH_TRANSACTION,
        constants::API_SOCKET
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
    let mut transaction = None;
    let mut log_level = None;
    let mut socket_path = None;

    let mut iter = args.skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_ref() {
            "--transaction" => {
                transaction = Some(
                    iter.next()
                        .unwrap_or_else(|| usage_msg("Did not give argument to --transaction")),
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
        transaction: transaction.unwrap_or_else(|| constants::LAUNCH_TRANSACTION.to_string()),
        log_level: log_level.unwrap_or(LevelFilter::Trace),
        socket_path: socket_path.unwrap_or_else(|| constants::API_SOCKET.to_string()),
    }
}

async fn run() -> Result<()> {
    // Parse and store the args passed to the program
    let args = parse_args(env::args());

    // SimpleLogger will send errors to stderr and anything less to stdout.
    SimpleLogger::init(args.log_level, LogConfig::default()).context(error::LoggerSnafu)?;

    info!("Checking pending settings.");
    check_pending_settings(&args.socket_path, &args.transaction).await;

    info!("Committing settings.");
    commit_pending_settings(&args.socket_path, &args.transaction).await?;

    Ok(())
}

// Returning a Result from main makes it print a Debug representation of the error, but with Snafu
// we have nice Display representations of the error, so we wrap "main" (run) and print any error.
// https://github.com/shepmaster/snafu/issues/110
#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("{}", e);
        process::exit(1);
    }
}
