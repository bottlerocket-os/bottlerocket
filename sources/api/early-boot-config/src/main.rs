/*!
# Introduction

early-boot-config sends provider-specific platform data to the Bottlerocket API.

For most providers this means configuration from user data and platform metadata, taken from
something like an instance metadata service.

Currently, Amazon EC2 is supported through the IMDSv1 HTTP API.  Data will be taken from files in
/etc/early-boot-config instead, if available, for testing purposes.
*/

#[macro_use]
extern crate log;

use simplelog::{Config as LogConfig, LevelFilter, SimpleLogger};
use snafu::{ensure, ResultExt};
use std::fs;
use std::str::FromStr;
use std::{env, process};

mod compression;
mod provider;
mod settings;
use crate::provider::{Platform, PlatformDataProvider};

// TODO
// Tests!

// We only want to run early-boot-config once, at first boot.  Our systemd unit file has a
// ConditionPathExists that will prevent it from running again if this file exists.
// We create it after running successfully.
const MARKER_FILE: &str = "/var/lib/bottlerocket/early-boot-config.ran";

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
        program_name,
        constants::API_SOCKET,
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
        log_level: log_level.unwrap_or(LevelFilter::Info),
        socket_path: socket_path.unwrap_or_else(|| constants::API_SOCKET.to_string()),
    }
}

async fn run() -> Result<()> {
    // Parse and store the args passed to the program
    let args = parse_args(env::args());

    // SimpleLogger will send errors to stderr and anything less to stdout.
    SimpleLogger::init(args.log_level, LogConfig::default()).context(error::LoggerSnafu)?;

    info!("early-boot-config started");

    info!("Retrieving platform-specific data");
    let uri = &format!(
        "{}?tx={}",
        constants::API_SETTINGS_URI,
        constants::LAUNCH_TRANSACTION
    );
    let method = "PATCH";
    for settings_json in Platform
        .platform_data()
        .await
        .context(error::ProviderSnafu)?
    {
        // Don't send an empty request to the API
        if settings_json.json.is_empty() {
            warn!("{} was empty", settings_json.desc);
            continue;
        }

        info!("Sending {} to API", settings_json.desc);
        trace!("Request body: {}", settings_json.json);
        let (code, response_body) =
            apiclient::raw_request(&args.socket_path, uri, method, Some(settings_json.json))
                .await
                .context(error::APIRequestSnafu { method, uri })?;
        ensure!(
            code.is_success(),
            error::ResponseSnafu {
                method,
                uri,
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
#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("{}", e);
        process::exit(1);
    }
}

mod error {
    use http::StatusCode;
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub(super) enum Error {
        #[snafu(display("Error {}ing '{}': {}", method, uri, source))]
        APIRequest {
            method: String,
            uri: String,
            #[snafu(source(from(apiclient::Error, Box::new)))]
            source: Box<apiclient::Error>,
        },

        #[snafu(display("Provider error: {}", source))]
        Provider { source: Box<dyn std::error::Error> },

        #[snafu(display("Error {} when {}ing '{}': {}", code, method, uri, response_body))]
        Response {
            method: String,
            uri: String,
            code: StatusCode,
            response_body: String,
        },

        #[snafu(display("Logger setup error: {}", source))]
        Logger { source: log::SetLoggerError },
    }
}

type Result<T> = std::result::Result<T, error::Error>;
