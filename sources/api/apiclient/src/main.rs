use std::env;
use std::process;
use unindent::unindent;

const DEFAULT_API_SOCKET: &str = "/run/api.sock";
const DEFAULT_METHOD: &str = "GET";

/// Stores user-supplied global arguments.
#[derive(Debug)]
struct Args {
    verbosity: usize,
    socket_path: String,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            verbosity: 3,
            socket_path: DEFAULT_API_SOCKET.to_string(),
        }
    }
}

/// Stores the usage mode specified by the user as a subcommand.
enum Subcommand {
    Raw(RawArgs),
}

/// Stores user-supplied arguments for the 'raw' subcommand.
struct RawArgs {
    method: String,
    uri: String,
    data: Option<String>,
}

/// Informs the user about proper usage of the program and exits.
fn usage() -> ! {
    let msg = &format!(
        r"Usage: apiclient [SUBCOMMAND] [OPTION]...

        Global options:
            -s, --socket-path PATH     Override the server socket path.  Default: {socket}
            -v, --verbose              Print extra information like HTTP status code.

        Subcommands:
            raw                        Makes an HTTP request to the server.
                                       'raw' is the default subcommand and may be omitted.

        raw options:
            -u, --uri URI              Required; URI to request from the server, e.g. /tx
            -m, -X, --method METHOD    HTTP method to use in request.  Default: {method}
            -d, --data DATA            Data to include in the request body.  Default: empty",
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
            "-v" | "--verbose" => global_args.verbosity += 1,

            "-s" | "--socket-path" => {
                global_args.socket_path = iter
                    .next()
                    .unwrap_or_else(|| usage_msg("Did not give argument to -s | --socket-path"))
            }

            // Subcommands
            "raw" if subcommand.is_none() && !arg.starts_with('-') => subcommand = Some(arg),

            // Other arguments are passed to the subcommand parser
            _ => subcommand_args.push(arg),
        }
    }

    match subcommand.as_deref() {
        // Default subcommand is 'raw'
        None | Some("raw") => return (global_args, parse_raw_args(subcommand_args)),
        _ => usage_msg("Missing or unknown subcommand"),
    }
}

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

async fn run() -> apiclient::Result<()> {
    let (args, subcommand) = parse_args(env::args());

    match subcommand {
        Subcommand::Raw(raw) => {
            let (status, body) =
                apiclient::raw_request(args.socket_path, raw.uri, raw.method, raw.data).await?;

            if args.verbosity > 3 {
                eprintln!("{}", status);
            }
            if !body.is_empty() {
                println!("{}", body);
            }
        }
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
