#[macro_use]
extern crate log;

use nix::unistd::{fork, ForkResult};
use schnauzer::BottlerocketTemplateImporter;
use simplelog::{Config as LogConfig, LevelFilter, SimpleLogger};
use snafu::ResultExt;
use std::collections::HashSet;
use std::env;
use std::process;
use std::str::FromStr;
use tokio::runtime::Runtime;

use thar_be_settings::{config, get_changed_settings, service};

mod error {
    use snafu::Snafu;
    use std::path::PathBuf;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub(super) enum Error {
        #[snafu(display("Logger setup error: {}", source))]
        Logger { source: log::SetLoggerError },

        #[snafu(display("Failure to read template '{}' from '{}': {}", name, path.display(), source))]
        TemplateRegister {
            name: String,
            path: PathBuf,
            source: handlebars::TemplateError,
        },
    }
}

/// RunMode represents how thar-be-settings was requested to be run, either handling all
/// configuration files and services, or handling configuration files and services based on
/// specific keys given by the user.
#[derive(Debug)]
enum RunMode {
    All,
    SpecificKeys,
}

/// Store the args we receive on the command line
struct Args {
    daemon: bool,
    log_level: LevelFilter,
    mode: RunMode,
    socket_path: String,
}

/// Print a usage message in the event a bad arg is passed
fn usage() -> ! {
    let program_name = env::args().next().unwrap_or_else(|| "program".to_string());
    eprintln!(
        r"Usage: {}
            [ --all ]
            [ --daemon ]
            [ --socket-path PATH ]
            [ --log-level trace|debug|info|warn|error ]

    If --all is given, all configuration files will be written and all
    services will have their restart-commands run.  Otherwise, settings keys
    will be read from stdin; only files related to those keys will be written,
    and only services related to those keys will be restarted.

    If --daemon is given, thar-be-settings will fork and do its work in a new
    process; this is useful to prevent blocking an API call.

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
    let mut daemon = false;
    let mut log_level = None;
    let mut mode = RunMode::SpecificKeys;
    let mut socket_path = None;

    let mut iter = args.skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_ref() {
            "--all" => mode = RunMode::All,

            "--daemon" => daemon = true,

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
        daemon,
        mode,
        log_level: log_level.unwrap_or(LevelFilter::Info),
        socket_path: socket_path.unwrap_or_else(|| constants::API_SOCKET.to_string()),
    }
}

/// Render and write config files to disk.  If `files_limit` is Some, only
/// write those files, otherwise write all known files.
async fn write_config_files(
    args: &Args,
    files_limit: Option<HashSet<String>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create a vec of ConfigFile structs from the list of changed services
    info!("Requesting configuration file data for affected services");
    let config_files = config::get_affected_config_files(&args.socket_path, files_limit).await?;
    trace!("Found config files: {:?}", config_files);

    let template_importer = BottlerocketTemplateImporter::new((&args.socket_path).into());

    // Ensure all files render properly
    info!("Rendering config files...");
    let strict = match &args.mode {
        RunMode::SpecificKeys => true,
        RunMode::All => false,
    };
    let rendered = config::render_config_files(&template_importer, config_files, strict).await?;

    // If all the config renders properly, write it to disk
    info!("Writing config files to disk...");
    config::write_config_files(&rendered)?;

    // If we're done with early boot and only working with specific services,
    // then trigger a reload if necessary.
    if let RunMode::SpecificKeys = &args.mode {
        config::reload_config_files(&rendered)?;
    }

    Ok(())
}

async fn run(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    // SimpleLogger will send errors to stderr and anything less to stdout.
    SimpleLogger::init(args.log_level, LogConfig::default()).context(error::LoggerSnafu)?;

    info!("thar-be-settings started");

    match args.mode {
        RunMode::SpecificKeys => {
            // Get the settings that changed via stdin
            info!("Parsing stdin for updated settings");
            let changed_settings = get_changed_settings()?;

            // Create a HashSet of affected services
            info!(
                "Requesting affected services for settings: {:?}",
                &changed_settings
            );
            let services =
                service::get_affected_services(&args.socket_path, Some(changed_settings)).await?;
            trace!("Found services: {:?}", services);
            if services.0.is_empty() {
                info!("No services are affected, exiting...");
                process::exit(0)
            }

            // Create a HashSet of configuration file names
            let config_file_names = config::get_config_file_names(&services);

            if !config_file_names.is_empty() {
                write_config_files(&args, Some(config_file_names)).await?;
            }

            // Now go bounce the affected services
            info!("Restarting affected services...");
            service::restart_services(services)?;
        }
        RunMode::All => {
            write_config_files(&args, None).await?;

            info!("Restarting all services...");
            let services = service::get_affected_services(&args.socket_path, None).await?;
            trace!("Found services: {:?}", services);
            service::restart_services(services)?;
        }
    }

    Ok(())
}

// Returning a Result from main makes it print a Debug representation of the error, but with Snafu
// we have nice Display representations of the error, so we wrap "main" (run) and print any error.
// https://github.com/shepmaster/snafu/issues/110
//
// In this binary, we also have to do a bit more processing before we get to the "business logic."
// This program is used to apply settings given to the API, but we don't want to block the API, so
// there's a --daemon argument that makes us fork before doing the work.  This also prevents zombie
// processes, since it's simpler to let init wait for our corpse than to make apiserver wait.  To
// determine whether that's wanted, we have to parse args, and then do the fork if requested.
//
// Also, it's not safe to fork within a tokio runtime, so we can't use tokio::main, and have to
// create the runtime manually before we start the business logic in run().
fn main() {
    // Parse and store the args passed to the program
    let args = parse_args(env::args());

    if args.daemon {
        match unsafe { fork() } {
            Ok(ForkResult::Child) => {} // continue
            Ok(ForkResult::Parent { .. }) => process::exit(0),
            Err(e) => {
                eprintln!("Failed to fork child: {}", e);
                process::exit(1);
            }
        }
    }

    let rt = Runtime::new().expect("Failed to create tokio runtime");
    if let Err(e) = rt.block_on(async { run(args).await }) {
        eprintln!("{}", e);
        process::exit(1);
    }
}
