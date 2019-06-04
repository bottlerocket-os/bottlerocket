//! This is the primary binary for the Thar API server.

#[macro_use]
extern crate log;

use log::Level::Info;
use snafu::ResultExt;
use std::env;
use std::error::Error;
use std::path::Path;
use std::process;

use apiserver::datastore::FilesystemDataStore;
use apiserver::handle_request;

// FIXME temporary port
const DEFAULT_BIND_ADDR: &str = "localhost:4242";

mod error {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility = "pub(crate)")]
    pub(crate) enum Error {
        #[snafu(display("Logger setup error: {}", source))]
        Logger { source: log::SetLoggerError },
    }
}

/// Stores user-supplied arguments.
struct Args {
    verbosity: usize,
    color: stderrlog::ColorChoice,
    datastore_path: String,
    socket_address: String,
}

/// Informs the user about proper usage of the program and exits.
fn usage() -> ! {
    let program_name = env::args().next().unwrap_or_else(|| "program".to_string());
    eprintln!(
        r"Usage: {}
            --datastore-path PATH
            [ --socket-address ADDR:PORT ]
            [ --no-color ]
            [ --verbose --verbose ... ]

    Socket address defaults to {}",
        program_name, DEFAULT_BIND_ADDR
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
    let mut datastore_path = None;
    let mut socket_address = None;
    let mut verbosity = 0;
    let mut color = stderrlog::ColorChoice::Auto;

    let mut iter = args.skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_ref() {
            "-v" | "--verbosity" => verbosity += 1,

            "--no-color" => color = stderrlog::ColorChoice::Never,

            "--datastore-path" => {
                datastore_path = Some(
                    iter.next()
                        .unwrap_or_else(|| usage_msg("Did not give argument to --datastore-path")),
                )
            }

            "--socket-address" => {
                socket_address = Some(
                    iter.next()
                        .unwrap_or_else(|| usage_msg("Did not give argument to --socket-address")),
                )
            }

            _ => usage(),
        }
    }

    Args {
        verbosity,
        color,
        datastore_path: datastore_path.unwrap_or_else(|| usage()),
        socket_address: socket_address.unwrap_or_else(|| DEFAULT_BIND_ADDR.to_string()),
    }
}

/// Starts a web server to accept user requests, dispatching those requests to the controller.
fn main() -> Result<(), Box<Error>> {
    let args = parse_args(env::args());

    // TODO: starting with simple stderr logging, replace when we have a better idea.
    stderrlog::new()
        .module(module_path!())
        .timestamp(stderrlog::Timestamp::Millisecond)
        .verbosity(args.verbosity)
        .color(args.color)
        .init()
        .context(error::Logger)?;

    // Create default datastore if it doesn't exist
    if !Path::new(&args.datastore_path).exists() {
        info!("Creating datastore at: {}", &args.datastore_path);
        FilesystemDataStore::populate_default(&args.datastore_path)?;
    }

    // Each request makes its own handle to the datastore; there's no locking or
    // synchronization yet.  Therefore, only use 1 thread for safety.
    let threads = Some(1);

    if log_enabled!(Info) {
        let threads_str = match threads {
            Some(n) => format!("{}", n),
            None => "DEFAULT".to_string(),
        };
        let threads_suffix = match threads {
            Some(n) if n > 1 => "s",
            Some(_) => "",
            None => "s",
        };
        info!(
            "Starting server at {} with {} thread{} and datastore at {}",
            &args.socket_address, threads_str, threads_suffix, &args.datastore_path,
        );
    }

    rouille::start_server_with_pool(args.socket_address.clone(), threads, move |request| {
        handle_request(request, &args.datastore_path)
    })
}
