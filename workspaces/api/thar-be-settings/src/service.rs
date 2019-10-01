use snafu::{ensure, OptionExt, ResultExt};
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;
use std::process;

use itertools::join;

use crate::client;
use crate::{error, Result};
use apiserver::model;

/// Wrapper for the multiple functions needed to go from
/// a list of changed settings to a Services map
#[allow(clippy::implicit_hasher)]
pub fn get_affected_services<P>(
    socket_path: P,
    settings_limit: Option<HashSet<String>>,
) -> Result<model::Services>
where
    P: AsRef<Path>,
{
    let service_limit = if let Some(settings_limit) = settings_limit {
        let setting_to_service_map =
            get_affected_service_map(socket_path.as_ref(), settings_limit)?;
        if setting_to_service_map.is_empty() {
            return Ok(HashMap::new());
        }

        let service_names = get_affected_service_names(setting_to_service_map);
        Some(service_names)
    } else {
        // No limit, we want all services.
        None
    };

    let services = get_service_metadata(socket_path.as_ref(), service_limit)?;

    Ok(services)
}

/// Gather the services affected for each setting into a map, or if `settings_limit` is None, all
/// services
#[allow(clippy::implicit_hasher)]
fn get_affected_service_map<P>(
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
        client::get_json(socket_path, uri, Some(query))?;
    trace!("API response: {:?}", &setting_to_services_map);

    Ok(setting_to_services_map)
}

/// Given a map of Setting to affected Service name, return a
/// HashSet of affected service names
fn get_affected_service_names(
    setting_to_service_map: HashMap<String, Vec<String>>,
) -> HashSet<String> {
    // Build a HashSet of names of affected services
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

/// Gather the metadata for each Service affected
fn get_service_metadata<P>(
    socket_path: P,
    services_limit: Option<HashSet<String>>,
) -> Result<model::Services>
where
    P: AsRef<Path>,
{
    // Only want a query parameter if we had specific affected services, otherwise we want all
    let query = services_limit.map(|services| ("names", join(&services, ",")));

    // Query the API for affected service metadata
    debug!("Querying API for affected service metadata");
    let service_map: model::Services = client::get_json(socket_path, "/services", query)?;
    trace!("Service metadata: {:?}", &service_map);

    Ok(service_map)
}

/// Call the `restart()` method on each Service in a Services object
pub fn restart_services(services: model::Services) -> Result<()> {
    for (name, service) in services {
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

impl ServiceRestart for model::Service {
    fn restart(&self) -> Result<()> {
        for restart_command in self.restart_commands.iter() {
            // Split on space, assume the first item is the command
            // and the rest are args.
            debug!("Restart command: {:?}", &restart_command);
            let mut command_strings = restart_command.split(' ');
            let command = command_strings
                .next()
                .context(error::InvalidRestartCommand {
                    command: restart_command.as_str(),
                })?;
            trace!("Command: {}", &command);
            trace!("Args: {:?}", &command_strings);

            // Go execute the restart command
            let result = process::Command::new(command)
                .args(command_strings)
                .output()
                .context(error::CommandExecutionFailure {
                    command: restart_command.as_str(),
                })?;

            // If the restart command exited nonzero, call it a failure
            ensure!(
                result.status.success(),
                error::FailedRestartCommand {
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
}
