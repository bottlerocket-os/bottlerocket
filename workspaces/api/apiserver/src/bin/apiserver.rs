//! This is the primary binary for the Thar API server.

#![deny(rust_2018_idioms)]

#[macro_use]
extern crate tracing;

use snafu::{ensure, ResultExt};
use std::env;
use std::path::Path;
use std::process;
use tracing_subscriber::{
    FmtSubscriber,
    filter::{LevelFilter, EnvFilter},
};

use apiserver::serve;

const DEFAULT_BIND_PATH: &str = "/run/api.sock";

type Result<T> = std::result::Result<T, error::Error>;

mod error {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility = "pub(crate)")]
    pub(crate) enum Error {
        #[snafu(display("Datastore does not exist, did storewolf run?"))]
        NonexistentDatastore,

        #[snafu(display("{}", source))]
        Server { source: apiserver::server::Error },

        #[snafu(display("Failed to parse provided directive: {}", source))]
        TracingDirectiveParse {
            source: tracing_subscriber::filter::LevelParseError,
        },
    }
}

/// Stores user-supplied arguments.
struct Args {
    datastore_path: String,
    socket_path: String,
    verbosity: usize,
}

/// Informs the user about proper usage of the program and exits.
fn usage() -> ! {
    let program_name = env::args().next().unwrap_or_else(|| "program".to_string());
    eprintln!(
        r"Usage: {}
            --datastore-path PATH
            [ --socket-path PATH ]
            [ --no-color ]
            [ --verbose --verbose ... ]

    Socket path defaults to {}",
        program_name, DEFAULT_BIND_PATH
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
    let mut socket_path = None;
    let mut verbosity = 1;

    let mut iter = args.skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_ref() {
            "-v" | "--verbose" => verbosity += 1,

            "--datastore-path" => {
                datastore_path = Some(
                    iter.next()
                        .unwrap_or_else(|| usage_msg("Did not give argument to --datastore-path")),
                )
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
        verbosity,
        datastore_path: datastore_path.unwrap_or_else(|| usage()),
        socket_path: socket_path.unwrap_or_else(|| DEFAULT_BIND_PATH.to_string()),
    }
}

/// Starts a web server to accept user requests, dispatching those requests to the controller.
fn main() -> Result<()> {
    let args = parse_args(env::args());

    let level: LevelFilter = args.verbosity.to_string().parse().context(error::TracingDirectiveParse)?;
    let filter = EnvFilter::from_default_env().add_directive(level.into());
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .finish();
    // Start the logger
    tracing::subscriber::set_global_default(subscriber).expect("setting tracing default failed");

    // Make sure the datastore exists
    ensure!(
        Path::new(&args.datastore_path).exists(),
        error::NonexistentDatastore
    );

    // Each request makes its own handle to the datastore; there's no locking or
    // synchronization yet.  Therefore, only use 1 thread for safety.
    let threads = 1;

    let threads_suffix = match threads {
        n if n > 1 => "s",
        _ => "",
    };
    info!(
        "Starting server at {} with {} thread{} and datastore at {}",
        &args.socket_path, threads, threads_suffix, &args.datastore_path,
    );

    serve(&args.socket_path, &args.datastore_path, threads).context(error::Server)
}
