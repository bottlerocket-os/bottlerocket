use std::env;
use std::process;

const DEFAULT_API_SOCKET: &str = "/run/api.sock";

/// Stores user-supplied arguments.
struct Args {
    verbosity: usize,
    socket_path: String,
    method: String,
    uri: String,
    data: Option<String>,
}

/// Informs the user about proper usage of the program and exits.
fn usage() -> ! {
    let program_name = env::args().next().unwrap_or_else(|| "program".to_string());
    eprintln!(
        r"Usage: {}
            (-u | --uri) URI
            [ (-X | -m | --method) METHOD ]
            [ (-d | --data) DATA ]
            [ (-s | --socket-path) PATH ]
            [ -v | --verbose ... ]

    Method defaults to GET
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

/// Parses user arguments into an Args structure.
fn parse_args(args: env::Args) -> Args {
    let mut socket_path = None;
    let mut verbosity = 3; // default to INFO
    let mut method = None;
    let mut uri = None;
    let mut data = None;

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

            _ => usage(),
        }
    }

    Args {
        verbosity,
        socket_path: socket_path.unwrap_or_else(|| DEFAULT_API_SOCKET.to_string()),
        method: method.unwrap_or_else(|| "GET".to_string()),
        uri: uri.unwrap_or_else(|| usage()),
        data,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args(env::args());

    let (status, body) =
        apiclient::raw_request(args.socket_path, args.uri, args.method, args.data)?;

    if args.verbosity > 3 {
        eprintln!("{}", status);
    }
    if !body.is_empty() {
        println!("{}", body);
    }
    Ok(())
}
