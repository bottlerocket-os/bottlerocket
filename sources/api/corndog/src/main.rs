/*!
corndog is a delicious way to get at the meat inside the kernels.
It sets kernel-related settings, for example:
* sysctl values, based on key/value pairs in `settings.kernel.sysctl`
* lockdown mode, based on the value of `settings.kernel.lockdown`
*/

use log::{debug, error, info, trace, warn};
use simplelog::{Config as LogConfig, LevelFilter, SimpleLogger};
use snafu::ResultExt;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::string::String;
use std::{env, process};

const SYSCTL_PATH_PREFIX: &str = "/proc/sys";
const LOCKDOWN_PATH: &str = "/sys/kernel/security/lockdown";

/// Store the args we receive on the command line.
struct Args {
    subcommand: String,
    log_level: LevelFilter,
    socket_path: String,
}

/// Main entry point.
async fn run() -> Result<()> {
    let args = parse_args(env::args());

    // SimpleLogger will send errors to stderr and anything less to stdout.
    SimpleLogger::init(args.log_level, LogConfig::default()).context(error::LoggerSnafu)?;

    // If the user has kernel settings, apply them.
    let model = get_model(args.socket_path).await?;
    if let Some(settings) = model.settings {
        if let Some(kernel) = settings.kernel {
            match args.subcommand.as_ref() {
                "sysctl" => {
                    if let Some(sysctls) = kernel.sysctl {
                        debug!("Applying sysctls: {:#?}", sysctls);
                        set_sysctls(sysctls);
                    }
                }
                "lockdown" => {
                    if let Some(lockdown) = kernel.lockdown {
                        debug!("Setting lockdown: {:#?}", lockdown);
                        set_lockdown(&lockdown)?;
                    }
                }
                _ => usage_msg(format!("Unknown subcommand '{}'", args.subcommand)), // should be unreachable
            }
        }
    }

    Ok(())
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

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
        .context(error::APIRequestSnafu { method, uri })?;

    if !code.is_success() {
        return error::APIResponseSnafu {
            method,
            uri,
            code,
            response_body,
        }
        .fail();
    }
    trace!("JSON response: {}", response_body);

    serde_json::from_str(&response_body).context(error::ResponseJsonSnafu { method, uri })
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
        if let Err(e) = fs::write(path, value) {
            // We don't fail because sysctl keys can vary between kernel versions and depend on
            // loaded modules.  It wouldn't be possible to deploy settings to a mixed-kernel fleet
            // if newer sysctl values failed on your older kernels, for example, and we believe
            // it's too cumbersome to have to specify in settings which keys are allowed to fail.
            error!("Failed to write sysctl value '{}': {}", key, e);
        }
    }
}

/// Sets the requested lockdown mode in the kernel.
///
/// The Linux kernel won't allow lowering the lockdown setting, but we want to allow users to
/// change the Bottlerocket setting and reboot for it to take effect.  Changing the Bottlerocket
/// setting means this code will run to write it out, but it wouldn't be able to convince the
/// kernel.  So, we just warn the user rather than trying to write and causing a failure that could
/// prevent the rest of a settings-changing transaction from going through.  We'll run again after
/// reboot to set lockdown as it was requested.
fn set_lockdown(lockdown: &str) -> Result<()> {
    let current_raw = fs::read_to_string(LOCKDOWN_PATH).unwrap_or_else(|_| "unknown".to_string());
    let current = parse_kernel_setting(&current_raw);
    trace!("Parsed lockdown setting '{}' to '{}'", current_raw, current);

    // The kernel doesn't allow rewriting the current value.
    if current == lockdown {
        info!("Requested lockdown setting is already in effect.");
        return Ok(());
    // As described above, the kernel doesn't allow lowering the value.
    } else if current == "confidentiality" || (current == "integrity" && lockdown == "none") {
        warn!("Can't lower lockdown setting at runtime; please reboot for it to take effect.",);
        return Ok(());
    }

    fs::write(LOCKDOWN_PATH, lockdown).context(error::LockdownSnafu { current, lockdown })
}

/// The Linux kernel provides human-readable output like `[none] integrity confidentiality` when
/// you read settings from virtual files like /sys/kernel/security/lockdown.  This parses out the
/// current value of the setting from that human-readable output.
///
/// There are also some files that only output the current value without the other options, so we
/// return the output as-is (except for trimming whitespace) if there are no brackets.
fn parse_kernel_setting(setting: &str) -> &str {
    let mut setting = setting.trim();
    // Take after the '['
    if let Some(idx) = setting.find('[') {
        if setting.len() > idx + 1 {
            setting = &setting[idx + 1..];
        }
    }
    // Take before the ']'
    if let Some(idx) = setting.find(']') {
        setting = &setting[..idx];
    }
    setting
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// Print a usage message in the event a bad argument is given.
fn usage() -> ! {
    let program_name = env::args().next().unwrap_or_else(|| "program".to_string());
    eprintln!(
        r"Usage: {} SUBCOMMAND [ ARGUMENTS... ]

    Subcommands:
        sysctl
        lockdown

    Global arguments:
        --socket-path PATH
        --log-level trace|debug|info|warn|error

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

/// Parses the arguments to the program and return a representative `Args`.
fn parse_args(args: env::Args) -> Args {
    let mut log_level = None;
    let mut socket_path = None;
    let mut subcommand = None;

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

            "sysctl" | "lockdown" => subcommand = Some(arg),

            _ => usage(),
        }
    }

    Args {
        subcommand: subcommand.unwrap_or_else(|| usage_msg("Must specify a subcommand.")),
        log_level: log_level.unwrap_or(LevelFilter::Info),
        socket_path: socket_path.unwrap_or_else(|| constants::API_SOCKET.to_string()),
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

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

mod error {
    use http::StatusCode;
    use snafu::Snafu;
    use std::io;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub(super) enum Error {
        #[snafu(display("Error {}ing to {}: {}", method, uri, source))]
        APIRequest {
            method: String,
            uri: String,
            #[snafu(source(from(apiclient::Error, Box::new)))]
            source: Box<apiclient::Error>,
        },

        #[snafu(display("Error {} when {}ing to {}: {}", code, method, uri, response_body))]
        APIResponse {
            method: String,
            uri: String,
            code: StatusCode,
            response_body: String,
        },

        #[snafu(display(
            "Failed to change lockdown from '{}' to '{}': {}",
            current,
            lockdown,
            source
        ))]
        Lockdown {
            current: String,
            lockdown: String,
            source: io::Error,
        },

        #[snafu(display("Logger setup error: {}", source))]
        Logger { source: log::SetLoggerError },

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

    #[test]
    fn brackets() {
        assert_eq!(
            "none",
            parse_kernel_setting("[none] integrity confidentiality")
        );
        assert_eq!(
            "integrity",
            parse_kernel_setting("none [integrity] confidentiality\n")
        );
        assert_eq!(
            "confidentiality",
            parse_kernel_setting("none integrity [confidentiality]")
        );
    }

    #[test]
    fn no_brackets() {
        assert_eq!("none", parse_kernel_setting("none"));
        assert_eq!(
            "none integrity confidentiality",
            parse_kernel_setting("none integrity confidentiality\n")
        );
    }
}
