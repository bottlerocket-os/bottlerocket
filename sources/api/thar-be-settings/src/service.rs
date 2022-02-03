use crate::{error, Result};
use itertools::join;
use snafu::{ensure, OptionExt, ResultExt};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::process::Command;

// TODO: thar-be-settings isn't used as a library; declare its modules in main rather than lib so
// we don't have to expose helper types like this just so we can call related functions in main.
/// The `Services` type is an augmented version of `model::Services` that also stores the list of
/// settings that were changed and that are relevant to each service.
#[derive(Debug, Default)]
pub struct Services(pub HashMap<String, Service>);
/// The `Service` type stores the original `model::Service` and the list of settings that have
/// changed that are relevant to that service.
#[derive(Debug)]
pub struct Service {
    /// The specific list of settings that changed and are relevant to this service.  Will be None
    /// if the program is running for *all* services, like at startup.
    pub changed_settings: Option<HashSet<String>>,
    /// The model's representation of this service.
    pub model: model::Service,
}

impl Services {
    /// Convert from the model's representation of all services to our own, adding in the lists of
    /// changed settings, if appropriate; pass None if the program is running for *all* services.
    pub fn from_model_services(
        input: model::Services,
        mut all_changed_settings: Option<HashMap<String, HashSet<String>>>,
    ) -> Self {
        let mut output = HashMap::new();
        for (name, model) in input {
            let changed_settings = all_changed_settings.as_mut().and_then(|s| s.remove(&name));
            output.insert(
                name,
                Service {
                    changed_settings,
                    model,
                },
            );
        }
        Self(output)
    }
}

/// Returns a `Services` reflecting the set of services affected by the given changed settings in
/// `settings_limit`.  If `settings_limit` is None, reflects all known services.
pub async fn get_affected_services<P>(
    socket_path: P,
    settings_limit: Option<HashSet<String>>,
) -> Result<Services>
where
    P: AsRef<Path>,
{
    let services: Services;
    if let Some(settings_limit) = settings_limit {
        // Get the list of affected services for each setting
        let affected_services =
            get_affected_service_metadata(socket_path.as_ref(), settings_limit).await?;
        if affected_services.is_empty() {
            return Ok(Services::default());
        }

        // Pull out the names of the services so we can ask the API about them.
        let service_names = affected_services.values().flatten().collect();
        // Ask the API for its metadata about the affected services.
        let service_meta = get_service_metadata(socket_path.as_ref(), Some(service_names)).await?;

        // Reverse the mapping, getting the list of changed settings for each service
        let mut changed_settings = HashMap::new();
        for (setting, services) in affected_services {
            for service in services {
                let settings = changed_settings.entry(service).or_insert_with(HashSet::new);
                settings.insert(setting.clone());
            }
        }

        services = Services::from_model_services(service_meta, Some(changed_settings));
    } else {
        // If there was no settings limit, get data for all services.
        let service_meta = get_service_metadata(socket_path.as_ref(), None).await?;
        services = Services::from_model_services(service_meta, None);
    }

    Ok(services)
}

/// Ask the API which services are affected by the given list of settings.
#[allow(clippy::implicit_hasher)]
async fn get_affected_service_metadata<P>(
    socket_path: P,
    settings: HashSet<String>,
) -> Result<HashMap<String, Vec<String>>>
where
    P: AsRef<Path>,
{
    let query = ("keys", join(&settings, ","));

    // Query the API for affected services
    debug!("Querying API for affected services names");
    let uri = "/metadata/affected-services";

    let setting_to_services_map: HashMap<String, Vec<String>> =
        schnauzer::get_json(socket_path, uri, Some(query))
            .await
            .context(error::GetJsonSnafu { uri })?;
    trace!("API response: {:?}", &setting_to_services_map);

    Ok(setting_to_services_map)
}

/// Ask the API for metadata about the given list of services, or all services if `services_limit`
/// is None.
async fn get_service_metadata<P>(
    socket_path: P,
    services_limit: Option<HashSet<&String>>,
) -> Result<model::Services>
where
    P: AsRef<Path>,
{
    // Only want a query parameter if we had specific affected services, otherwise we want all
    let query = services_limit.map(|services| ("names", join(&services, ",")));

    // Query the API for affected service metadata
    debug!("Querying API for affected service metadata");
    let uri = "/services";
    let service_map: model::Services = schnauzer::get_json(socket_path, uri, query)
        .await
        .context(error::GetJsonSnafu { uri })?;
    trace!("Service metadata: {:?}", &service_map);

    Ok(service_map)
}

/// Call the `restart()` method on each Service in a Services object
pub fn restart_services(services: Services) -> Result<()> {
    for (name, service) in services.0 {
        debug!("Checking for restart-commands for {}", name);
        service.restart()?;
    }
    Ok(())
}

/// This trait is primarily meant to extend the Service model.  It uses the metadata
/// inside the Service struct to restart the service.
trait ServiceRestart {
    /// Restart the service
    fn restart(&self) -> Result<()>;
}

impl ServiceRestart for Service {
    fn restart(&self) -> Result<()> {
        let restart_commands = &self.model.restart_commands;
        info!("restart commands {:?}", restart_commands);
        for restart_command in restart_commands {
            // Split on space, assume the first item is the command
            // and the rest are args.
            debug!("Restart command: {:?}", &restart_command);
            let mut command_strings = restart_command.split(' ');
            let command = command_strings
                .next()
                .context(error::InvalidRestartCommandSnafu {
                    command: restart_command.as_str(),
                })?;
            trace!("Command: {}", &command);
            trace!("Args: {:?}", &command_strings);

            // Go execute the restart command
            let mut process_command = Command::new(command);
            process_command.args(command_strings);
            if let Some(ref changed_settings) = self.changed_settings {
                if !changed_settings.is_empty() {
                    process_command.env("CHANGED_SETTINGS", join(changed_settings, " "));
                }
            }
            let result = process_command
                .output()
                .context(error::CommandExecutionFailureSnafu {
                    command: restart_command.as_str(),
                })?;

            // If the restart command exited nonzero, call it a failure
            ensure!(
                result.status.success(),
                error::FailedRestartCommandSnafu {
                    command: restart_command.as_str(),
                    stderr: String::from_utf8_lossy(&result.stderr),
                }
            );
            trace!(
                "Command stdout: {}",
                String::from_utf8_lossy(&result.stdout)
            );
            trace!(
                "Command stderr: {}",
                String::from_utf8_lossy(&result.stderr)
            );
        }
        Ok(())
    }
}
