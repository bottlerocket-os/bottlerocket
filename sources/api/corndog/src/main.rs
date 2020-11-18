/*!
corndog is a delicious way to get at the meat inside the kernels.
It sets kernel sysctl values based on key/value pairs in `settings.kernel.sysctl`.
*/

#![deny(rust_2018_idioms)]

use log::{debug, error, trace};
use simplelog::{Config as LogConfig, LevelFilter, TermLogger, TerminalMode};
use snafu::ResultExt;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::string::String;
use std::{env, process};

const DEFAULT_API_SOCKET: &str = "/run/api.sock";
const SYSCTL_PATH_PREFIX: &str = "/proc/sys";

/// Store the args we receive on the command line.
struct Args {
    log_level: LevelFilter,
    socket_path: String,
}

/// Main entry point.
async fn run() -> Result<()> {
    let args = parse_args(env::args());

    // TerminalMode::Mixed will send errors to stderr and anything less to stdout.
    TermLogger::init(args.log_level, LogConfig::default(), TerminalMode::Mixed)
        .context(error::Logger)?;

    // If the user has sysctl settings, apply them.
    let model = get_model(args.socket_path).await?;
    if let Some(settings) = model.settings {
        if let Some(kernel) = settings.kernel {
            if let Some(sysctls) = kernel.sysctl {
                debug!("Applying sysctls: {:#?}", sysctls);
                set_sysctls(sysctls);
            }
        }
    }

    Ok(())
}

/// Retrieve the current model from the API.
async fn get_model<P>(socket_path: P) -> Result<model::Model>
where
    P: AsRef<Path>,
{
    let uri = "/";
    let method = "GET";
    trace!("{}ing from {}", method, uri);
    let (code, response_body) = apiclient::raw_request(socket_path, &uri, method, None)
        .await
        .context(error::APIRequest { method, uri })?;

    if !code.is_success() {
        return error::APIResponse {
            method,
            uri,
            code,
            response_body,
        }
        .fail();
    }
    trace!("JSON response: {}", response_body);

    serde_json::from_str(&response_body).context(error::ResponseJson { method, uri })
}

fn sysctl_path<S>(name: S) -> PathBuf
where
    S: AsRef<str>,
{
    let name = name.as_ref();
    let mut path = PathBuf::from(SYSCTL_PATH_PREFIX);
    path.extend(name.replace('.', "/").split('/'));
    trace!("Path for {}: {}", name, path.display());
    path
}

/// Applies the requested sysctls to the system.  The keys are used to generate the appropriate
/// path, and the value its contents.
fn set_sysctls<K>(sysctls: HashMap<K, String>)
where
    K: AsRef<str>,
{
    for (key, value) in sysctls {
        let key = key.as_ref();
        let path = sysctl_path(key);
        if let Err(e) = fs::write(&path, value) {
            // We don't fail because sysctl keys can vary between kernel versions and depend on
            // loaded modules.  It wouldn't be possible to deploy settings to a mixed-kernel fleet
            // if newer sysctl values failed on your older kernels, for example, and we believe
            // it's too cumbersome to have to specify in settings which keys are allowed to fail.
            error!("Failed to write sysctl value '{}': {}", key, e);
        }
    }
}

/// Print a usage message in the event a bad argument is given.
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

/// Parses the arguments to the program and return a representative `Args`.
fn parse_args(args: env::Args) -> Args {
    let mut log_level = None;
    let mut socket_path = None;

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
        log_level: log_level.unwrap_or_else(|| LevelFilter::Info),
        socket_path: socket_path.unwrap_or_else(|| DEFAULT_API_SOCKET.to_string()),
    }
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
    #[snafu(visibility = "pub(super)")]
    pub(super) enum Error {
        #[snafu(display("Error {}ing to {}: {}", method, uri, source))]
        APIRequest {
            method: String,
            uri: String,
            source: apiclient::Error,
        },

        #[snafu(display("Error {} when {}ing to {}: {}", code, method, uri, response_body))]
        APIResponse {
            method: String,
            uri: String,
            code: StatusCode,
            response_body: String,
        },

        #[snafu(display("Logger setup error: {}", source))]
        Logger { source: simplelog::TermLogError },

        #[snafu(display(
            "Error deserializing response as JSON from {} to '{}': {}",
            method,
            uri,
            source
        ))]
        ResponseJson {
            method: &'static str,
            uri: String,
            source: serde_json::Error,
        },
    }
}
type Result<T> = std::result::Result<T, error::Error>;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn no_traversal() {
        assert_eq!(
            sysctl_path("../../root/file").to_string_lossy(),
            format!("{}/root/file", SYSCTL_PATH_PREFIX)
        );
    }
}
