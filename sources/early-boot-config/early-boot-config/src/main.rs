/*!
# Introduction

early-boot-config sends user data to the Bottlerocket API.

Variants include their required user data provider binaries via packages.  early-boot-config discovers these binaries at runtime in /usr/libexec/early-boot-config/data-providers.d and runs them in order, sending any user data found to the API.

User data provider binaries each implement the ability to obtain user data from a single source.  Sources include local files, AWS Instance Metadata Service (IMDS), among others.
*/

#[macro_use]
extern crate log;

use early_boot_config_provider::settings::SettingsJson;
use early_boot_config_provider::LOG_LEVEL_ENV_VAR;
use env_logger::{Target, WriteStyle};
use log::LevelFilter;
use snafu::{ensure, ResultExt};
use std::fs;
use std::path::{Path, PathBuf};
use std::str::{self, FromStr};
use std::{env, io, process};
use tokio::process::Command as AsyncCommand;
use walkdir::WalkDir;

// TODO
// Tests!

// We only want to run early-boot-config once, at first boot.  Our systemd unit file has a
// ConditionPathExists that will prevent it from running again if this file exists.
// We create it after running successfully.
const MARKER_FILE: &str = "/var/lib/bottlerocket/early-boot-config.ran";
/// The directory containing user data provider binaries
const PROVIDERS_DIR: &str = "/usr/libexec/early-boot-config/data-providers.d";

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

/// Gather user data providers to run in order
fn gather_providers() -> Result<Vec<PathBuf>> {
    Ok(WalkDir::new(PROVIDERS_DIR)
        .max_depth(1)
        .min_depth(1)
        .sort_by_file_name()
        .into_iter()
        .collect::<std::result::Result<Vec<_>, _>>()?
        .into_iter()
        .filter(|f| f.path().is_symlink())
        .map(|f| f.into_path())
        .collect())
}

/// Run a user data provider binary
async fn run_provider<P>(log_level: &LevelFilter, provider: P) -> io::Result<process::Output>
where
    P: AsRef<Path>,
{
    let provider = provider.as_ref();
    AsyncCommand::new(provider)
        .env(LOG_LEVEL_ENV_VAR, log_level.as_str())
        .output()
        .await
}

/// Check that a user data provider succeeded and forward its logs
fn check_provider_status<P>(provider: P, output: &process::Output) -> Result<()>
where
    P: AsRef<Path>,
{
    let provider = provider.as_ref();
    // Regardless of provider status, log its output
    let provider_name = provider
        .file_name()
        .unwrap_or(provider.as_os_str())
        .to_string_lossy();
    let provider_logs = String::from_utf8_lossy(&output.stderr);
    for line in provider_logs.lines() {
        info!("Provider '{}': {}", provider_name, line);
    }

    ensure!(
        output.status.success(),
        error::ProviderFailureSnafu {
            provider: &provider,
            message: String::from_utf8_lossy(&output.stdout),
        }
    );

    Ok(())
}

/// Submit user data to the API
async fn submit_user_data<S>(socket_path: S, user_data: serde_json::Value) -> Result<()>
where
    S: AsRef<str>,
{
    let socket_path = socket_path.as_ref();
    let uri = &format!(
        "{}?tx={}",
        constants::API_SETTINGS_URI,
        constants::LAUNCH_TRANSACTION
    );
    let method = "PATCH";
    trace!("Request body: {}", user_data);

    let (code, response_body) =
        apiclient::raw_request(socket_path, uri, method, Some(user_data.to_string()))
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
    Ok(())
}

async fn run() -> Result<()> {
    // Parse and store the args passed to the program
    let args = parse_args(env::args());

    env_logger::Builder::new()
        .filter_level(args.log_level)
        .format_module_path(false)
        .target(Target::Stdout)
        .write_style(WriteStyle::Never)
        .init();

    info!("early-boot-config started");

    info!("Gathering user data providers");
    let mut threads = Vec::new();
    let providers = gather_providers()?;
    for provider in providers {
        threads.push((
            provider.clone(),
            tokio::spawn(async move { run_provider(&args.log_level, &provider).await }),
        ));
    }

    for (provider, handle) in threads {
        let result =
            handle
                .await
                .context(error::ThreadSnafu)?
                .context(error::CommandFailureSnafu {
                    provider: provider.clone(),
                })?;
        check_provider_status(&provider, &result)?;

        // User data providers output a serialized `SettingsJson` if they are successful in finding
        // user data at their respective source.  Output will be empty otherwise.
        //
        // Read into a string first to ensure UTF8 and strip any whitespace/newlines
        let output_raw = str::from_utf8(&result.stdout)
            .context(error::ProviderOutputSnafu {
                provider: &provider,
            })?
            .trim()
            .to_string();
        trace!("Provider '{}' output: {}", &provider.display(), &output_raw);

        if output_raw.is_empty() {
            info!("No user data found via {}", &provider.display());
            continue;
        }

        let output: SettingsJson =
            serde_json::from_str(&output_raw).context(error::ProviderJsonSnafu { provider })?;

        info!("Found user data via {}, sending to API", output.desc);
        submit_user_data(&args.socket_path, output.json).await?;
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
    use crate::PROVIDERS_DIR;
    use http::StatusCode;
    use snafu::Snafu;
    use std::path::PathBuf;

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

        #[snafu(display("Failed to start provider '{}': {}", provider.display(), source))]
        CommandFailure {
            provider: PathBuf,
            source: std::io::Error,
        },

        #[snafu(display("Provider error: {}", source))]
        Provider { source: Box<dyn std::error::Error> },

        #[snafu(display("Provider '{}' failed: {}", provider.display(), message))]
        ProviderFailure {
            provider: PathBuf,
            message: String,
        },

        #[snafu(display(
            "Error deserializing provider output as JSON from {}: '{}'",
            provider.display(),
            source,
        ))]
        ProviderJson {
            provider: PathBuf,
            source: serde_json::Error,
        },

        #[snafu(display("Invalid (non-utf8) output from provider '{}': {}", provider.display(), source))]
        ProviderOutput {
            provider: PathBuf,
            source: std::str::Utf8Error,
        },

        #[snafu(display("Error {} when {}ing '{}': {}", code, method, uri, response_body))]
        Response {
            method: String,
            uri: String,
            code: StatusCode,
            response_body: String,
        },

        #[snafu(display("Thread execution error: {}", source))]
        Thread { source: tokio::task::JoinError },

        #[snafu(display("Logger setup error: {}", source))]
        Logger { source: log::SetLoggerError },

        #[snafu(
            display("Unable to walk providers directory '{}': {}", PROVIDERS_DIR, source),
            context(false)
        )]
        WalkDir { source: walkdir::Error },
    }
}

type Result<T> = std::result::Result<T, error::Error>;
