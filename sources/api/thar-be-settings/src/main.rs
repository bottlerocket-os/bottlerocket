#[macro_use]
extern crate log;

use simplelog::{Config as LogConfig, LevelFilter, TermLogger, TerminalMode};
use snafu::ResultExt;
use std::collections::HashSet;
use std::env;
use std::process;
use std::str::FromStr;

use thar_be_settings::{config, get_changed_settings, service};

// FIXME Get from configuration in the future
const DEFAULT_API_SOCKET: &str = "/run/api.sock";

mod error {
    use snafu::Snafu;
    use std::path::PathBuf;

    #[derive(Debug, Snafu)]
    #[snafu(visibility = "pub(super)")]
    pub(super) enum Error {
        #[snafu(display("Logger setup error: {}", source))]
        Logger { source: log::SetLoggerError },

        #[snafu(display("Failure to read template '{}' from '{}': {}", name, path.display(), source))]
        TemplateRegister {
            name: String,
            path: PathBuf,
            source: handlebars::TemplateFileError,
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
            [ --socket-path PATH ]
            [ --log-level trace|debug|info|warn|error ]

    If --all is given, all configuration files will be written and all
    services will have their restart-commands run.  Otherwise, settings keys
    will be read from stdin; only files related to those keys will be written,
    and only services related to those keys will be restarted.

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

/// Parse the args to the program and return an Args struct
fn parse_args(args: env::Args) -> Args {
    let mut log_level = None;
    let mut mode = RunMode::SpecificKeys;
    let mut socket_path = None;

    let mut iter = args.skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_ref() {
            "--all" => mode = RunMode::All,

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
        mode,
        log_level: log_level.unwrap_or_else(|| LevelFilter::Info),
        socket_path: socket_path.unwrap_or_else(|| DEFAULT_API_SOCKET.to_string()),
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

    // Build the template registry from config file metadata
    debug!("Building template registry");
    let mut template_registry = schnauzer::build_template_registry()?;
    for (name, metadata) in &config_files {
        debug!(
            "Registering {} at path '{}'",
            &name, &metadata.template_path
        );
        template_registry
            .register_template_file(&name, metadata.template_path.as_ref())
            .context(error::TemplateRegister {
                name: name.as_str(),
                path: metadata.template_path.as_ref(),
            })?;
    }

    // Get all settings values for config file templates
    debug!("Requesting settings values");
    let settings = schnauzer::get_settings(&args.socket_path).await?;

    // Ensure all files render properly
    info!("Rendering config files...");
    let strict = match &args.mode {
        RunMode::SpecificKeys => true,
        RunMode::All => false,
    };
    let rendered = config::render_config_files(&template_registry, config_files, settings, strict)?;

    // If all the config renders properly, write it to disk
    info!("Writing config files to disk...");
    config::write_config_files(rendered)?;

    Ok(())
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    // Parse and store the args passed to the program
    let args = parse_args(env::args());

    // TerminalMode::Mixed will send errors to stderr and anything less to stdout.
    TermLogger::init(args.log_level, LogConfig::default(), TerminalMode::Mixed)
        .context(error::Logger)?;

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
            if services.is_empty() {
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
#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("{}", e);
        process::exit(1);
    }
}
