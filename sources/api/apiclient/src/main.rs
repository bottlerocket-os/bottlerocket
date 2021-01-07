//! The apiclient binary provides some high-level, synchronous methods of interacting with the
//! API, for example an `update` subcommand that wraps the individual API calls needed to update
//! the host.  There's also a low-level `raw` subcommand for direct interaction.

use apiclient::update;
use log::{info, log_enabled, trace, warn};
use simplelog::{ConfigBuilder as LogConfigBuilder, LevelFilter, TermLogger, TerminalMode};
use snafu::ResultExt;
use std::env;
use std::process;
use std::str::FromStr;
use unindent::unindent;

const DEFAULT_API_SOCKET: &str = "/run/api.sock";
const DEFAULT_METHOD: &str = "GET";

/// Stores user-supplied global arguments.
#[derive(Debug)]
struct Args {
    log_level: LevelFilter,
    socket_path: String,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            log_level: LevelFilter::Info,
            socket_path: DEFAULT_API_SOCKET.to_string(),
        }
    }
}

/// Stores the usage mode specified by the user as a subcommand.
#[derive(Debug)]
enum Subcommand {
    Raw(RawArgs),
    Reboot(RebootArgs),
    Update(UpdateSubcommand),
}

/// Stores user-supplied arguments for the 'raw' subcommand.
#[derive(Debug)]
struct RawArgs {
    method: String,
    uri: String,
    data: Option<String>,
}

/// Stores user-supplied arguments for the 'reboot' subcommand.
#[derive(Debug)]
struct RebootArgs {}

/// Stores the 'update' subcommand specified by the user.
#[derive(Debug)]
enum UpdateSubcommand {
    Check(CheckArgs),
    Apply(ApplyArgs),
    Cancel(CancelArgs),
}

/// Stores user-supplied arguments for the 'update check' subcommand.
#[derive(Debug)]
struct CheckArgs {}

/// Stores user-supplied arguments for the 'update apply' subcommand.
#[derive(Debug)]
struct ApplyArgs {
    check: bool,
    reboot: bool,
}

/// Stores user-supplied arguments for the 'update cancel' subcommand.
#[derive(Debug)]
struct CancelArgs {}

/// Informs the user about proper usage of the program and exits.
fn usage() -> ! {
    let msg = &format!(
        r"Usage: apiclient [SUBCOMMAND] [OPTION]...

        Global options:
            -s, --socket-path PATH     Override the server socket path.  Default: {socket}
            --log-level                Desired amount of output; trace|debug|info|warn|error
            -v, --verbose              Sets log level to 'debug'.  This prints extra info,
                                       like HTTP status code to stderr in 'raw' mode.

        Subcommands:
            raw                        Makes an HTTP request and prints the response on stdout.
                                       'raw' is the default subcommand and may be omitted.
            update check               Prints information about available updates.
            update apply               Applies available updates.
            update cancel              Deactivates an applied update.
            reboot                     Reboots the host.

        raw options:
            -u, --uri URI              Required; URI to request from the server, e.g. /tx
            -m, -X, --method METHOD    HTTP method to use in request.  Default: {method}
            -d, --data DATA            Data to include in the request body.  Default: empty

        reboot options:
            None.

        update check options:
            None.

        update apply options:
            -c, --check                Automatically `update check` and apply whatever is found.
            -r, --reboot               Automatically reboot if an update was found and applied.

        update cancel options:
            None.",
        socket = DEFAULT_API_SOCKET,
        method = DEFAULT_METHOD,
    );
    eprintln!("{}", unindent(msg));
    process::exit(2);
}

/// Prints a more specific message before exiting through usage().
fn usage_msg<S: AsRef<str>>(msg: S) -> ! {
    eprintln!("{}\n", msg.as_ref());
    usage();
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// Parses user arguments into an Args structure.
fn parse_args(args: env::Args) -> (Args, Subcommand) {
    let mut global_args = Args::default();
    let mut subcommand = None;
    let mut subcommand_args = Vec::new();

    let mut iter = args.into_iter().skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_ref() {
            "-h" | "--help" => usage(),

            // Global args
            "--log-level" => {
                let log_level_str = iter
                    .next()
                    .unwrap_or_else(|| usage_msg("Did not give argument to --log-level"));
                global_args.log_level =
                    LevelFilter::from_str(&log_level_str).unwrap_or_else(|_| {
                        usage_msg(format!("Invalid log level '{}'", log_level_str))
                    });
            }

            "-v" | "--verbose" => global_args.log_level = LevelFilter::Debug,

            "-s" | "--socket-path" => {
                global_args.socket_path = iter
                    .next()
                    .unwrap_or_else(|| usage_msg("Did not give argument to -s | --socket-path"))
            }

            // Subcommands
            "raw" | "reboot" | "update" if subcommand.is_none() && !arg.starts_with('-') => {
                subcommand = Some(arg)
            }

            // Other arguments are passed to the subcommand parser
            _ => subcommand_args.push(arg),
        }
    }

    match subcommand.as_deref() {
        // Default subcommand is 'raw'
        None | Some("raw") => return (global_args, parse_raw_args(subcommand_args)),
        Some("reboot") => return (global_args, parse_reboot_args(subcommand_args)),
        Some("update") => return (global_args, parse_update_args(subcommand_args)),
        _ => usage_msg("Missing or unknown subcommand"),
    }
}

/// Parses arguments for the 'raw' subcommand, which is also the default if no subcommand is
/// provided.
fn parse_raw_args(args: Vec<String>) -> Subcommand {
    let mut method = None;
    let mut uri = None;
    let mut data = None;

    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_ref() {
            "-X" | "-m" | "--method" => {
                method = Some(
                    iter.next()
                        .unwrap_or_else(|| usage_msg("Did not give argument to -m | --method")),
                )
            }

            "-u" | "--uri" => {
                uri = Some(
                    iter.next()
                        .unwrap_or_else(|| usage_msg("Did not give argument to -u | --uri")),
                )
            }

            "-d" | "--data" => {
                data = Some(
                    iter.next()
                        .unwrap_or_else(|| usage_msg("Did not give argument to -d | --data")),
                )
            }

            x => usage_msg(&format!("Unknown argument '{}'", x)),
        }
    }

    Subcommand::Raw(RawArgs {
        method: method.unwrap_or_else(|| DEFAULT_METHOD.to_string()),
        uri: uri.unwrap_or_else(|| usage_msg("Missing required argument '--uri'")),
        data,
    })
}

/// Parses arguments for the 'reboot' subcommand.
fn parse_reboot_args(args: Vec<String>) -> Subcommand {
    if !args.is_empty() {
        usage_msg(&format!("Unknown arguments: {}", args.join(", ")));
    }
    Subcommand::Reboot(RebootArgs {})
}

/// Parses the desired subcommand of 'update'.
fn parse_update_args(args: Vec<String>) -> Subcommand {
    let mut subcommand = None;
    let mut subcommand_args = Vec::new();

    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_ref() {
            // Subcommands
            "check" | "apply" | "cancel" if subcommand.is_none() && !arg.starts_with('-') => {
                subcommand = Some(arg)
            }

            // Other arguments are passed to the subcommand parser
            _ => subcommand_args.push(arg),
        }
    }

    let update = match subcommand.as_deref() {
        Some("check") => parse_check_args(subcommand_args),
        Some("apply") => parse_apply_args(subcommand_args),
        Some("cancel") => parse_cancel_args(subcommand_args),
        _ => usage_msg("Missing or unknown subcommand for 'update'"),
    };

    Subcommand::Update(update)
}

/// Parses arguments for the 'update check' subcommand.
fn parse_check_args(args: Vec<String>) -> UpdateSubcommand {
    if !args.is_empty() {
        usage_msg(&format!("Unknown arguments: {}", args.join(", ")));
    }
    UpdateSubcommand::Check(CheckArgs {})
}

/// Parses arguments for the 'update apply' subcommand.
fn parse_apply_args(args: Vec<String>) -> UpdateSubcommand {
    let mut check = false;
    let mut reboot = false;

    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_ref() {
            "-c" | "--check" => check = true,
            "-r" | "--reboot" => reboot = true,

            x => usage_msg(&format!("Unknown argument '{}'", x)),
        }
    }

    UpdateSubcommand::Apply(ApplyArgs { check, reboot })
}

/// Parses arguments for the 'update cancel' subcommand.
fn parse_cancel_args(args: Vec<String>) -> UpdateSubcommand {
    if !args.is_empty() {
        usage_msg(&format!("Unknown arguments: {}", args.join(", ")));
    }
    UpdateSubcommand::Cancel(CancelArgs {})
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// Requests a reboot through the API.
async fn reboot(args: &Args) -> Result<()> {
    let uri = "/actions/reboot";
    let method = "POST";
    let (_status, _body) = apiclient::raw_request(&args.socket_path, uri, method, None)
        .await
        .context(error::Request { uri, method })?;

    info!("Rebooting, goodbye...");
    Ok(())
}

/// Requests an update status check through the API, printing the updated status, in a pretty
/// format if possible.
async fn check(args: &Args) -> Result<String> {
    let output = update::check(&args.socket_path)
        .await
        .context(error::Check)?;

    match serde_json::from_str::<serde_json::Value>(&output) {
        Ok(value) => println!("{:#}", value),
        Err(e) => {
            warn!("Unable to deserialize response (invalid JSON?): {}", e);
            println!("{}", output);
        }
    }

    Ok(output)
}

/// Main entry point, dispatches subcommands.
async fn run() -> Result<()> {
    let (args, subcommand) = parse_args(env::args());
    trace!("Parsed args for subcommand {:?}: {:?}", subcommand, args);

    // We use TerminalMode::Stderr because apiclient users expect server response data on stdout.
    TermLogger::init(
        args.log_level,
        LogConfigBuilder::new()
            .add_filter_allow_str("apiclient")
            .build(),
        TerminalMode::Stderr,
    )
    .context(error::Logger)?;

    match subcommand {
        Subcommand::Raw(raw) => {
            let (status, body) =
                apiclient::raw_request(&args.socket_path, &raw.uri, &raw.method, raw.data)
                    .await
                    .context(error::Request {
                        uri: &raw.uri,
                        method: &raw.method,
                    })?;

            // In raw mode, the user is expecting only the server response on stdout, so we more
            // carefully control other output and only write it to stderr.
            if log_enabled!(log::Level::Debug) {
                eprintln!("{}", status);
            }
            if !body.is_empty() {
                println!("{}", body);
            }
        }

        Subcommand::Reboot(_reboot) => {
            reboot(&args).await?;
        }

        Subcommand::Update(subcommand) => match subcommand {
            UpdateSubcommand::Check(_check) => {
                check(&args).await?;
            }

            UpdateSubcommand::Apply(apply) => {
                if apply.check {
                    let output = check(&args).await?;
                    // Exit early if no update is required, either because none is available or one
                    // is already applied and ready.
                    if !update::required(&output) {
                        return Ok(());
                    }
                }

                update::apply(&args.socket_path)
                    .await
                    .context(error::Apply)?;

                // If the user requested it, and if we applied an update, reboot.  (update::apply
                // will fail if no update was available or it couldn't apply the update.)
                if apply.reboot {
                    reboot(&args).await?;
                } else {
                    info!("Update has been applied and will take effect on next reboot.");
                }
            }

            UpdateSubcommand::Cancel(_cancel) => {
                update::cancel(&args.socket_path)
                    .await
                    .context(error::Cancel)?;
            }
        },
    }

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
    use apiclient::update;
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility = "pub(super)")]
    pub enum Error {
        #[snafu(display("Failed to apply update: {}", source))]
        Apply { source: update::Error },

        #[snafu(display("Failed to cancel update: {}", source))]
        Cancel { source: update::Error },

        #[snafu(display("Failed to check for updates: {}", source))]
        Check { source: update::Error },

        #[snafu(display("Logger setup error: {}", source))]
        Logger { source: log::SetLoggerError },

        #[snafu(display("Failed {} request to '{}': {}", method, uri, source))]
        Request {
            method: String,
            uri: String,
            source: apiclient::Error,
        },
    }
}
type Result<T> = std::result::Result<T, error::Error>;
