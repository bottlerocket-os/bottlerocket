#[macro_use]
extern crate log;

use snafu::ResultExt;
use std::env;
use std::process;

use thar_be_settings::{config, get_changed_settings, service, settings, template};

// TODO
// Use a client rather than building queries and making HTTP calls

mod error {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility = "pub(super)")]
    pub(super) enum Error {
        #[snafu(display("Logger setup error: {}", source))]
        Logger { source: log::SetLoggerError },
    }
}

/// Store the args we receive on the command line
struct Args {
    verbosity: usize,
}

/// Print a usage message in the event a bad arg is passed
fn usage() -> ! {
    let program_name = env::args().next().unwrap_or_else(|| "program".to_string());
    eprintln!(
        r"Usage: {}
            [ --verbose --verbose ... ]
        ",
        program_name
    );
    process::exit(2);
}

/// Parse the args to the program and return an Args struct
fn parse_args(args: env::Args) -> Args {
    let mut verbosity = 2;

    for arg in args.skip(1) {
        match arg.as_ref() {
            "-v" | "--verbose" => verbosity += 1,
            _ => usage(),
        }
    }

    Args { verbosity }
}

fn main() -> Result<(), Box<std::error::Error>> {
    // Parse and store the args passed to the program
    let args = parse_args(env::args());

    // TODO fix this in the future when we understand our logging strategy;
    // it should also be configurable
    // Start the logger
    stderrlog::new()
        .module(module_path!())
        .timestamp(stderrlog::Timestamp::Millisecond)
        .verbosity(args.verbosity)
        .color(stderrlog::ColorChoice::Never)
        .init()
        .context(error::Logger)?;

    info!("thar-be-settings started");

    // Get the settings that changed via stdin
    info!("Parsing stdin for updated settings");
    let changed_settings = get_changed_settings()?;

    // Create a client for all our API calls
    let client = reqwest::Client::new();

    // Create a HashSet of affected services
    info!(
        "Requesting affected services for settings: {:?}",
        &changed_settings
    );
    let services = service::get_affected_services(&client, changed_settings)?;
    if services.is_empty() {
        info!("No services are affected, exiting...");
        process::exit(0)
    }

    // Create a HashSet of configuration file names
    let config_file_names = config::get_config_file_names(&services);
    if !config_file_names.is_empty() {
        // Create a vec of ConfigFile structs from the list of changed services
        info!("Requesting configuration file data for affected services");
        let config_files = config::get_affected_config_files(&client, config_file_names)?;

        // Build the template registry from config file metadata
        debug!("Building template registry");
        let template_registry = template::build_template_registry(&config_files)?;

        // Get all settings values for config file templates
        debug!("Requesting settings values");
        let settings = settings::get_settings_from_template(&client, &template_registry)?;

        // Ensure all files render properly
        info!("Rendering config files...");
        let rendered = config::render_config_files(&template_registry, config_files, settings)?;

        // If all the config renders properly, write it to disk
        info!("Writing config files to disk...");
        config::write_config_files(rendered)?;
    }

    // Now go bounce all the services
    info!("Restarting affected services...");
    service::restart_all_services(services)?;

    Ok(())
}
