#![deny(rust_2018_idioms)]
#![warn(clippy::pedantic)]

#[path = "../error.rs"]
mod error;
#[path = "../transport.rs"]
mod transport;

use crate::error::Result;
use crate::transport::HttpQueryTransport;
use semver::Version as SemVer;
use simplelog::{Config as LogConfig, LevelFilter, TermLogger, TerminalMode};
use snafu::{ErrorCompat, ResultExt};
use std::str::FromStr;
use tough::Transport;
use url::Url;

/// Prints a more specific message before exiting through usage().
fn usage_msg<S: AsRef<str>>(msg: S) -> ! {
    eprintln!("{}\n", msg.as_ref());
    usage();
}

fn usage() -> ! {
    #[rustfmt::skip]
    eprintln!("\
USAGE:
    smoky <QUERY OPTIONS> --target <target url> [--log-level <level>]

QUERY OPTIONS:
    [ --boot-success | --boot-failure ] Signal whether the boot succeeed or not
    [ -v | --current-version ]          Version of currently running image
    [ -p | --previous-version ]         Version of last booted image

    [ --target ]                        URL of the telemetry target file
    [ --log-level trace|debug|info|warn|error ]  Set logging verbosity");
    std::process::exit(1)
}

/// Struct to hold the specified command line argument values
struct Arguments {
    boot_success: Option<bool>,
    current_version: Option<SemVer>,
    previous_version: Option<SemVer>,

    target_url: String,

    log_level: LevelFilter,
}

/// Parse the command line arguments to get the user-specified values
fn parse_args(args: std::env::Args) -> Arguments {
    let mut mark_success = false;
    let mut mark_failure = false;
    let mut target_url = None;
    let mut current_version = None;
    let mut previous_version = None;
    let mut log_level = None;

    let mut iter = args.skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_ref() {
            "--log-level" => {
                let log_level_str = iter
                    .next()
                    .unwrap_or_else(|| usage_msg("Did not give argument to --log-level"));
                log_level = Some(LevelFilter::from_str(&log_level_str).unwrap_or_else(|_| {
                    usage_msg(format!("Invalid log level '{}'", log_level_str))
                }));
            }
            "--boot-success" => {
                mark_success = true;
            }
            "--boot-failure" => {
                mark_failure = true;
            }
            "-v" | "--current-version" => match iter.next() {
                Some(v) => {
                    if let Ok(version) = SemVer::parse(&v) {
                        current_version = Some(version);
                    } else {
                        usage();
                    }
                }
                _ => usage(),
            },
            "-p" | "--previous-version" => match iter.next() {
                Some(v) => {
                    if let Ok(version) = SemVer::parse(&v) {
                        previous_version = Some(version);
                    } else {
                        usage();
                    }
                }
                _ => usage(),
            },
            "--target" => match iter.next() {
                Some(u) => target_url = Some(u),
                _ => usage(),
            },
            _ => usage(),
        }
    }

    if mark_success && mark_failure {
        // Must specify success xor failure
        usage();
    }

    // Exit if no queries were specified
    if !(mark_success || mark_failure) && current_version.is_none() && previous_version.is_none() {
        usage_msg("No queries to send");
    }

    Arguments {
        boot_success: if mark_success || mark_failure {
            Some(mark_success)
        } else {
            None
        },
        current_version,
        previous_version,
        target_url: target_url.unwrap_or_else(|| usage()),
        log_level: log_level.unwrap_or_else(|| LevelFilter::Info),
    }
}

fn main_inner() -> Result<()> {
    let arguments = parse_args(std::env::args());

    // TerminalMode::Mixed will send errors to stderr and anything less to stdout.
    TermLogger::init(
        arguments.log_level,
        LogConfig::default(),
        TerminalMode::Mixed,
    )
    .context(error::Logger)?;

    let transport = HttpQueryTransport::new();
    let query_target = Url::parse(&arguments.target_url).context(error::UrlParse)?;

    if let Some(boot_success) = arguments.boot_success {
        let status = if boot_success {
            String::from("success")
        } else {
            String::from("failure")
        };
        transport
            .queries_get_mut()
            .context(error::TransportBorrow)?
            .push((String::from("boot-status"), status));
    }
    if let Some(current_version) = arguments.current_version {
        transport
            .queries_get_mut()
            .context(error::TransportBorrow)?
            .push((String::from("version"), current_version.to_string()));
    }
    if let Some(previous_version) = arguments.previous_version {
        transport
            .queries_get_mut()
            .context(error::TransportBorrow)?
            .push((String::from("fallback"), previous_version.to_string()));
    }

    transport.fetch(query_target).context(error::FetchFailure)?;
    Ok(())
}

fn main() -> ! {
    std::process::exit(match main_inner() {
        Ok(()) => 0,
        Err(err) => {
            eprintln!("{}", err);
            if let Some(var) = std::env::var_os("RUST_BACKTRACE") {
                if var != "0" {
                    if let Some(backtrace) = err.backtrace() {
                        eprintln!("\n{:?}", backtrace);
                    }
                }
            }
            1
        }
    })
}
