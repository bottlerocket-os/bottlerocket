/*!
# Background

thar-be-settings is a simple configuration applier.

It is intended to be called from, and work directly with, the API server in Thar, the OS.
After a settings change, this program queries the API to determine which services and configuration files are affected by that change.
Once it has done so, it renders and rewrites the affected configuration files and restarts any affected services.
*/

#[macro_use]
extern crate derive_error;
#[macro_use]
extern crate log;

use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::io;
use std::io::prelude::*;
use std::path::PathBuf;
use std::process;

use handlebars::template;
use handlebars::Handlebars;

use apiserver::datastore::deserialization;
use apiserver::model;

// TODO
// Use a client rather than building queries and making HTTP calls

// FIXME Get these from configuration in the future
const API_CONFIGURATION_URI: &str = "http://localhost:4242/configuration-files";
const API_METADATA_URI: &str = "http://localhost:4242/metadata";
const API_SETTINGS_URI: &str = "http://localhost:4242/settings";
const API_SERVICES_URI: &str = "http://localhost:4242/services";

type Result<T> = std::result::Result<T, TBSError>;

/// Potential errors during configuration application
#[derive(Debug, Error)]
enum TBSError {
    /// Restart command failure
    RestartCommand(std::io::Error),
    /// Configuration file template fails to render
    TemplateRender(handlebars::RenderError),
    /// Failure to read template file from path
    TemplateRegister(handlebars::TemplateFileError),
    /// Error making request to API
    APIRequest(reqwest::Error),
    /// JSON ser/deserialize error
    JSON(serde_json::error::Error),
    #[error(msg_embedded, no_from, non_std)]
    /// Value from the datastore is invalid or unable to be operated
    /// on; i.e. a path that cannot exist
    DatastoreValue(String),
    /// Deserialization error coming from API code
    DeserializationError(deserialization::DeserializationError),
    /// Logger setup error
    Logger(log::SetLoggerError),
}

/// This trait is primarily meant to extend the Service model.  It uses the metadata
/// inside the Service struct to restart the service.
trait ServiceRestart {
    /// Restart the service
    fn restart(&self) -> Result<()>;
}

impl ServiceRestart for model::Service {
    fn restart(&self) -> Result<()> {
        for restart_command in self.restart_commands.iter() {
            // Split on space, assume the first item is the command
            // and the rest are args.
            debug!("Restart command: {:?}", &restart_command);
            let command_strings: Vec<&str> = restart_command.split(' ').collect();
            let command = command_strings[0];
            let args = command_strings[1..].to_vec();
            trace!("Command: {}", &command);
            trace!("Args: {:?}", &args);

            // Go execute the restart command
            let _fixme = process::Command::new(command)
                .args(args)
                .output()
                .map_err(TBSError::RestartCommand)?;
        }
        Ok(())
    }
}

/// RenderedConfigFile contains both the path to the config file
/// and the rendered data to write.
#[derive(Debug)]
struct RenderedConfigFile {
    path: PathBuf,
    rendered: String,
}

impl RenderedConfigFile {
    fn new(path: &str, rendered: String) -> RenderedConfigFile {
        RenderedConfigFile {
            path: PathBuf::from(&path),
            rendered,
        }
    }

    /// Writes the rendered template at the proper location
    fn write_to_disk(&self) -> Result<()> {
        let dirname = self.path.parent().ok_or_else(|| {
            TBSError::DatastoreValue("Config file path does not have proper prefix".to_string())
        })?;

        fs::create_dir_all(dirname).map_err(TBSError::from)?;

        fs::write(&self.path, self.rendered.as_bytes()).map_err(TBSError::from)
    }
}

/// This trait allows us to get a list of template keys (Expressions in handlebars
/// parlance) out of a template
trait TemplateKeys {
    /// Return a HashSet of template keys from a template
    fn get_all_template_keys(&self) -> Result<HashSet<String>>;
}

/// Extends the template::Template type from the Handlebars library to extract
/// all keys from a single template
impl TemplateKeys for template::Template {
    /// Retrieve all keys from a single template
    fn get_all_template_keys(&self) -> Result<HashSet<String>> {
        let mut keys: HashSet<String> = HashSet::new();

        for element in &self.elements {
            // Currently we only match on Expressions and HelperBlocks (conditionals)
            // and ignore everything else. Our templates are simple so far and this
            // match should capture all the template keys.
            match element {
                handlebars::template::TemplateElement::Expression(name) => {
                    if let handlebars::template::Parameter::Name(key) = name {
                        trace!("Found key: {}", &key);
                        keys.insert(key.to_string());
                    }
                }

                handlebars::template::TemplateElement::HelperBlock(block) => {
                    if let Some(ref tmpl) = block.template {
                        for key in tmpl.get_all_template_keys()?.into_iter() {
                            trace!("Found key: {}", &key);
                            keys.insert(key);
                        }
                    }
                }

                // Not an expression
                _ => {}
            }
        }
        Ok(keys)
    }
}

/// Extends the Handlebars type (the template Registry) from the Handlebars library
/// to extract all keys from all templates currently registered
impl TemplateKeys for Handlebars {
    /// Retrieve all keys from all templates in the registry
    fn get_all_template_keys(&self) -> Result<HashSet<String>> {
        debug!("Querying registry for templates");
        let templates = self.get_templates();
        trace!("Templates in registry: {:?}", &templates);

        // For each of the templates in the repository, get all the
        // keys and add them to the HashSet to be returned
        let mut keys = HashSet::new();
        for (template_name, template) in templates {
            debug!("Parsing template: {}", &template_name);
            for key in template.get_all_template_keys()?.into_iter() {
                keys.insert(key);
            }
        }
        Ok(keys)
    }
}

/// Given a map of Setting to affected Service name, return a
/// HashSet of affected service names
fn get_affected_service_names(
    setting_to_service_map: HashMap<String, Vec<String>>,
) -> HashSet<String> {
    // Gather the affected services in a HashSet.
    debug!("Building set of affected services");
    let mut service_set: HashSet<String> = HashSet::new();
    for (_, service_list) in setting_to_service_map {
        for service_name in service_list {
            debug!("Found {}", &service_name);
            service_set.insert(service_name);
        }
    }

    trace!("Affected service names: {:?}", service_set);
    service_set
}

/// Given a map of Service objects, return a HashSet of
/// affected configuration file names
fn get_affected_config_file_names(services: &HashMap<String, model::Service>) -> HashSet<String> {
    debug!("Building set of affected configuration file names");
    let mut config_file_set = HashSet::new();
    for (service_name, service) in services {
        for file in service.configuration_files.iter() {
            debug!("Found {} for service {}", &file, &service_name);
            config_file_set.insert(file.to_string());
        }
    }

    trace!("Affected configuration files {:?}", config_file_set);
    config_file_set
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
            "-v" | "--verbosity" => verbosity += 1,
            _ => usage(),
        }
    }

    Args { verbosity }
}

// main is long now, but we plan to reorganize when we have a better API client.
#[allow(clippy::cyclomatic_complexity)]
fn main() -> Result<()> {
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
        .init()?;

    info!("thar-be-settings started");

    // Get the settings that changed via stdin
    info!("Parsing stdin for updated settings");
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    trace!("Raw input from stdin: {}", &input);

    // Settings should be a vec of strings
    // FIXME log helpful error output here if input isn't
    // in a format we expect
    info!("Parsing stdin into JSON");
    let changed_settings: HashSet<String> = serde_json::from_str(&input)?;
    trace!("Parsed input: {:?}", &changed_settings);

    // Create a client for all our API calls
    let client = reqwest::Client::new();

    // Create a list of Services that are affected by settings changed
    info!(
        "Requesting affected services for settings: {:?}",
        &changed_settings
    );
    // Build the affected services query string
    debug!("Building API query for affected services");
    let affected_services_query = changed_settings
        .into_iter()
        .map(|p| p.to_string())
        .collect::<Vec<String>>()
        .join(",")
        .to_string();
    trace!("Query string: {}", &affected_services_query);

    // Query the API for affected services
    debug!("Querying API for affected services names");
    let uri = API_METADATA_URI.to_string() + "/affected-services";
    let setting_to_services_map: HashMap<String, Vec<String>> = client
        .get(&uri)
        .query(&[("keys", affected_services_query)])
        .send()?
        .error_for_status()?
        .json()?;
    trace!("API response: {:?}", &setting_to_services_map);

    // Create a HashSet of affected services
    let affected_services = get_affected_service_names(setting_to_services_map);
    if affected_services.is_empty() {
        info!("No services are affected, exiting...");
        process::exit(0)
    }

    // Build the service metadata query string
    debug!("Building API query for service metadata");
    let service_metadata_query = affected_services
        .into_iter()
        .map(|p| p.to_string())
        .collect::<Vec<String>>()
        .join(",")
        .to_string();
    trace!("Query string: {}", &service_metadata_query);

    // Query the API for affected service metadata
    debug!("Querying API for affected service metadata");
    let service_map: HashMap<String, model::Service> = client
        .get(API_SERVICES_URI)
        .query(&[("names", service_metadata_query)])
        .send()?
        .error_for_status()?
        .json()?;
    trace!("Service metadata: {:?}", &service_map);

    // Create a HashSet of configuration file names
    let affected_config_files = get_affected_config_file_names(&service_map);
    if !affected_config_files.is_empty() {
        // Create a vec of ConfigFile structs from the list of changed services
        info!("Requesting configuration file data for affected services");
        debug!("Building API query for configuration file metadata");
        let config_query = affected_config_files
            .into_iter()
            .map(|p| p.to_string())
            .collect::<Vec<String>>()
            .join(",")
            .to_string();
        trace!("Query string: {}", &config_query);

        debug!("Querying API for configuration file metadata");
        let config_files_map: HashMap<String, model::ConfigurationFile> = client
            .get(API_CONFIGURATION_URI)
            .query(&[("names", &config_query)])
            .send()?
            .error_for_status()?
            .json()?;

        // Build the template registry using the ConfigFile structs
        // and let handlebars parse the templates
        // Strict mode will panic if a key exists in the template
        // but isn't provided in the data given to the renderer
        let mut template_registry = Handlebars::new();
        template_registry.set_strict_mode(true);

        info!("Building template registry of configuration files");
        for (name, metadata) in &config_files_map {
            debug!(
                "Registering {} at path '{}'",
                &name, &metadata.template_path
            );
            template_registry.register_template_file(&name, &metadata.template_path)?;
        }

        // Using the template registry, pull the keys out of the templates
        // and query the API to get a structure of Settings which we can
        // use to render the templates
        info!("Gathering keys from configuration file templates");
        let settings_to_query = template_registry.get_all_template_keys()?;

        info!("Requesting settings values for template keys");
        // Build the settings query
        debug!("Building API query for affected services");
        let settings_query = &settings_to_query
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<String>>()
            .join(",")
            .to_string();
        trace!("Settings query: {:?}", &settings_query);

        // Query the settings
        debug!("Querying API for settings data");
        let settings: model::Settings = client
            .get(API_SETTINGS_URI)
            .query(&[("keys", &settings_query)])
            .send()?
            .error_for_status()?
            .json()?;

        // The following is simply to satisfy the Handlebars templating library.
        // The variables in the templates are prefixed with "settings"
        // {{ settings.foo.bar }} so we need to wrap the Settings struct in a map
        // with the key "settings".
        let mut wrapped_template_keys: HashMap<String, model::Settings> = HashMap::new();
        wrapped_template_keys.insert("settings".to_string(), settings);
        trace!("Final template keys map: {:?}", &wrapped_template_keys);

        // Go write all the configuration files from template
        info!("Rendering config files...");
        let mut rendered_configs = Vec::new();
        for (name, metadata) in config_files_map {
            info!("Rendering {}", &name);
            let rendered = template_registry.render(&name, &wrapped_template_keys)?;
            rendered_configs.push(RenderedConfigFile::new(&metadata.path, rendered));
        }
        trace!("Rendered configs: {:?}", &rendered_configs);

        // If all the config renders properly, write it to disk
        info!("Writing config files to disk...");
        for cfg in rendered_configs {
            info!("Writing {:?}", &cfg.path);
            cfg.write_to_disk()?;
        }
    }

    // Now go bounce all the services
    info!("Restarting affected services...");
    for (_, service) in service_map {
        service.restart()?;
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use maplit::{hashmap, hashset};

    #[test]
    fn test_get_affected_service_names() {
        let input_map = hashmap!(
            "settings.hostname".to_string() => vec![
                "hostname".to_string(),
                "timezone".to_string(),
            ],
            "settings.foobar".to_string() => vec![
                "timezone".to_string(),
                "barbaz".to_string()
            ]
        );

        let expected_output =
            hashset! {"hostname".to_string(), "timezone".to_string(), "barbaz".to_string()};

        assert_eq!(get_affected_service_names(input_map), expected_output)
    }

    #[test]
    fn test_get_affected_config_file_names() {
        let input_map = hashmap!(
            "foo".to_string() => model::Service {
                configuration_files: vec!["file1".to_string()],
                restart_commands: vec!["echo hi".to_string()]
            },
            "bar".to_string() => model::Service {
                configuration_files: vec!["file1".to_string(), "file2".to_string()],
                restart_commands: vec!["echo hi".to_string()]
            },
        );

        let expected_output = hashset! {"file1".to_string(), "file2".to_string() };

        assert_eq!(get_affected_config_file_names(&input_map), expected_output)
    }

    #[test]
    // Ensure that we get all the keys out of a single template
    fn get_template_keys_from_single_template() {
        let template_name = "test_tmpl";
        let template_string = "This is a cool {{template}}. Here is an conditional: {{#if bridge-ip }}{{bridge-ip}}{{/if}}";
        let expected_keys = hashset! {"template".to_string(), "bridge-ip".to_string() };

        // Register the template so the registry creates a Template object
        let mut registry = Handlebars::new();
        registry
            .register_template_string(template_name, template_string)
            .unwrap();

        // Get the template from the registry
        let template = registry.get_template(template_name).unwrap();

        assert!(template.get_all_template_keys().is_ok());
        assert_eq!(template.get_all_template_keys().unwrap(), expected_keys)
    }

    #[test]
    // Ensure that we get all the keys out of a the entire registry
    fn get_template_keys_from_registry() {
        let name1 = "test_tmpl1";
        let tmpl1 = "This is a cool {{template}}. Here is an conditional: {{#if bridge-ip }}{{bridge-ip}}{{/if}}";

        let name2 = "test_tmpl2";
        let tmpl2 = "This is a cool {{frob}}. Here is an conditional: {{#if frobnicate }}{{frobnicate}}{{/if}}";

        let expected_keys = hashset! {"template".to_string(), "bridge-ip".to_string(), "frob".to_string(), "frobnicate".to_string() };

        // Register the templates so the registry creates Template objects
        let mut registry = Handlebars::new();
        registry.register_template_string(name1, tmpl1).unwrap();
        registry.register_template_string(name2, tmpl2).unwrap();

        assert!(registry.get_all_template_keys().is_ok());
        assert_eq!(registry.get_all_template_keys().unwrap(), expected_keys)
    }
}
