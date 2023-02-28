//! This is the primary binary for the Bottlerocket API server.

#[macro_use]
extern crate log;

use libc::gid_t;
use nix::unistd::Gid;
use simplelog::{Config as LogConfig, LevelFilter, SimpleLogger};
use snafu::{ensure, ResultExt};
use std::env;
use std::path::Path;
use std::process;
use std::str::FromStr;

use apiserver::serve;

/// By default, this is where we create the Unix-domain socket that exposes our API.
const DEFAULT_BIND_PATH: &str = "/run/api.sock";
/// By default, when the user requests that we run a process via /exec, we run the process through
/// this containerd socket.
const DEFAULT_EXEC_SOCKET: &str = "/run/host-containerd/containerd.sock";

type Result<T> = std::result::Result<T, error::Error>;

mod error {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(crate)))]
    pub(crate) enum Error {
        #[snafu(display("Datastore does not exist, did storewolf run?"))]
        NonexistentDatastore,

        #[snafu(display("{}", source))]
        Server { source: apiserver::server::Error },

        #[snafu(display("Logger setup error: {}", source))]
        Logger { source: log::SetLoggerError },
    }
}

/// Stores user-supplied arguments.
struct Args {
    datastore_path: String,
    log_level: LevelFilter,
    socket_gid: Option<Gid>,
    socket_path: String,
    exec_socket_path: String,
}

/// Informs the user about proper usage of the program and exits.
fn usage() -> ! {
    let program_name = env::args().next().unwrap_or_else(|| "program".to_string());
    eprintln!(
        r"Usage: {}
            --datastore-path PATH
            [ --socket-path PATH ]
            [ --socket-gid GROUP_ID ]
            [ --exec-socket-path PATH ]
            [ --no-color ]
            [ --log-level trace|debug|info|warn|error ]

    --socket-path defaults to {}
    --exec-socket-path (for apiclient exec) defaults to {}",
        program_name, DEFAULT_BIND_PATH, DEFAULT_EXEC_SOCKET
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
    let mut log_level = None;
    let mut socket_gid = None;
    let mut socket_path = None;
    let mut exec_socket_path = None;

    let mut iter = args.skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_ref() {
            "--datastore-path" => {
                datastore_path = Some(
                    iter.next()
                        .unwrap_or_else(|| usage_msg("Did not give argument to --datastore-path")),
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

            "--socket-path" => {
                socket_path = Some(
                    iter.next()
                        .unwrap_or_else(|| usage_msg("Did not give argument to --socket-path")),
                )
            }

            "--socket-gid" => {
                let gid_str = iter
                    .next()
                    .unwrap_or_else(|| usage_msg("Did not give argument to --socket-gid"));
                let gid = gid_str.parse::<gid_t>().unwrap_or_else(|e| {
                    usage_msg(format!(
                        "Invalid group ID '{}' given to --socket-gid: {}",
                        gid_str, e
                    ))
                });
                socket_gid = Some(Gid::from_raw(gid));
            }

            "--exec-socket-path" => {
                exec_socket_path =
                    Some(iter.next().unwrap_or_else(|| {
                        usage_msg("Did not give argument to --exec-socket-path")
                    }))
            }

            _ => usage(),
        }
    }

    Args {
        socket_gid,
        datastore_path: datastore_path.unwrap_or_else(|| usage()),
        log_level: log_level.unwrap_or(LevelFilter::Info),
        socket_path: socket_path.unwrap_or_else(|| DEFAULT_BIND_PATH.to_string()),
        exec_socket_path: exec_socket_path.unwrap_or_else(|| DEFAULT_EXEC_SOCKET.to_string()),
    }
}

/// Starts a web server to accept user requests, dispatching those requests to the controller.
async fn run() -> Result<()> {
    let args = parse_args(env::args());

    // SimpleLogger will send errors to stderr and anything less to stdout.
    SimpleLogger::init(args.log_level, LogConfig::default()).context(error::LoggerSnafu)?;

    // Make sure the datastore exists
    ensure!(
        Path::new(&args.datastore_path).exists(),
        error::NonexistentDatastoreSnafu
    );

    // Access to the data store is controlled through a RwLock, allowing many readers, but a
    // writer will block all other access.  We don't expect any real load, though, as the API
    // is only used by the owner of the host.  Note that WebSocket requests like /exec are
    // automatically handled in new threads and won't block others.
    let threads = 1;

    let threads_suffix = match threads {
        n if n > 1 => "s",
        _ => "",
    };
    info!(
        "Starting server at {} with {} thread{} and datastore at {}",
        &args.socket_path, threads, threads_suffix, &args.datastore_path,
    );

    serve(
        &args.socket_path,
        &args.datastore_path,
        threads,
        args.socket_gid,
        args.exec_socket_path,
    )
    .await
    .context(error::ServerSnafu)
}

// Returning a Result from main makes it print a Debug representation of the error, but with Snafu
// we have nice Display representations of the error, so we wrap "main" (run) and print any error.
// https://github.com/shepmaster/snafu/issues/110
#[actix_rt::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("{}", e);
        process::exit(1);
    }
}
