use crate::service::Services;
use crate::{error, Result};
use itertools::join;
use schnauzer::BottlerocketTemplateImporter;
use snafu::{ensure, ResultExt};
use std::collections::HashSet;
use std::fs::{self, OpenOptions};
use std::io::prelude::*;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

const SYSTEMCTL_DAEMON_RELOAD: &str = "systemctl daemon-reload";
const DEFAULT_FILE_MODE: u32 = 0o644;

/// Query the API for ConfigurationFile data
#[allow(clippy::implicit_hasher)]
pub async fn get_affected_config_files<P>(
    socket_path: P,
    files_limit: Option<HashSet<String>>,
) -> Result<model::ConfigurationFiles>
where
    P: AsRef<Path>,
{
    // Only want a query parameter if we had specific affected files, otherwise we want all
    let query = files_limit.map(|files| ("names", join(&files, ",")));

    debug!("Querying API for configuration file metadata");
    let uri = "/configuration-files";
    let config_files: model::ConfigurationFiles = schnauzer::v1::get_json(socket_path, uri, query)
        .await
        .context(error::GetJsonSnafu { uri })?;

    Ok(config_files)
}

/// Given a map of Service objects, return a HashSet of
/// affected configuration file names
pub fn get_config_file_names(services: &Services) -> HashSet<String> {
    debug!("Building set of affected configuration file names");
    let mut config_file_set = HashSet::new();
    for service in services.0.values() {
        for file in &service.model.configuration_files {
            config_file_set.insert(file.to_string());
        }
    }

    trace!("Config file names: {:?}", config_file_set);
    config_file_set
}

/// Render the configuration files
// If strict is True, return an error if we fail to render any template.
// If strict is False, ignore failures, always returning an Ok value
// containing any successfully rendered templates.
pub async fn render_config_files(
    template_importer: &BottlerocketTemplateImporter,
    config_files: model::ConfigurationFiles,
    strict: bool,
) -> Result<Vec<RenderedConfigFile>> {
    // Go write all the configuration files from template
    let mut rendered_configs = Vec::new();
    for (name, metadata) in config_files {
        debug!("Rendering {}", &name);

        let try_rendered =
            schnauzer::render_template_file(template_importer, &metadata.template_path.as_ref())
                .await;

        if strict {
            let rendered = try_rendered.context(error::TemplateRenderSnafu { template: name })?;
            rendered_configs.push(RenderedConfigFile::new(
                &metadata.path,
                rendered,
                &metadata.mode,
            ));
        } else {
            match try_rendered {
                Ok(rendered) => rendered_configs.push(RenderedConfigFile::new(
                    &metadata.path,
                    rendered,
                    &metadata.mode,
                )),
                Err(err) => warn!("Unable to render template '{}': {}", &name, err),
            }
        }
    }
    trace!("Rendered configs: {:?}", &rendered_configs);
    Ok(rendered_configs)
}

/// Write all the configuration files to disk
pub fn write_config_files(rendered_configs: &[RenderedConfigFile]) -> Result<()> {
    for cfg in rendered_configs {
        debug!("Writing {:?}", &cfg.path);
        cfg.write_to_disk()?;
    }
    Ok(())
}

/// Run `systemd daemon-reload` if any modified config file requires it.
pub fn reload_config_files(rendered_configs: &[RenderedConfigFile]) -> Result<()> {
    if rendered_configs
        .iter()
        .any(RenderedConfigFile::needs_reload)
    {
        let mut args = SYSTEMCTL_DAEMON_RELOAD.split(' ');
        let program = args.next().expect("failed to split on space");
        trace!("Command: {}", &program);
        trace!("Args: {:?}", &args);

        let result = Command::new(program).args(args).output().context(
            error::CommandExecutionFailureSnafu {
                command: SYSTEMCTL_DAEMON_RELOAD,
            },
        )?;

        // If the reload command exited nonzero, call it a failure
        ensure!(
            result.status.success(),
            error::FailedReloadCommandSnafu {
                command: SYSTEMCTL_DAEMON_RELOAD,
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

/// RenderedConfigFile contains both the path to the config file
/// and the rendered data to write.
#[derive(Debug)]
pub struct RenderedConfigFile {
    path: PathBuf,
    rendered: String,
    mode: Option<String>,
}

impl RenderedConfigFile {
    fn new(path: &str, rendered: String, mode: &Option<String>) -> RenderedConfigFile {
        RenderedConfigFile {
            path: PathBuf::from(&path),
            rendered,
            mode: mode.to_owned(),
        }
    }

    /// Writes the rendered template at the proper location
    fn write_to_disk(&self) -> Result<()> {
        if let Some(dirname) = self.path.parent() {
            fs::create_dir_all(dirname).context(error::TemplateWriteSnafu {
                path: dirname,
                pathtype: "directory",
            })?;
        };

        let mut binding = OpenOptions::new();
        let options = binding
            .write(true)
            .create(true)
            .truncate(true)
            .mode(DEFAULT_FILE_MODE);

        // See if this file has a config setting for a specific mode
        if let Some(mode) = &self.mode {
            let mode_int =
                u32::from_str_radix(mode.as_str(), 8).context(error::TemplateModeSnafu {
                    path: &self.path,
                    mode,
                })?;
            options.mode(mode_int);
        }

        let mut file = options
            .open(&self.path)
            .context(error::TemplateWriteSnafu {
                path: &self.path,
                pathtype: "file",
            })?;

        file.write_all(self.rendered.as_bytes())
            .context(error::TemplateWriteSnafu {
                path: &self.path,
                pathtype: "file",
            })
    }

    /// Checks whether the config file needs `systemd` to reload.
    fn needs_reload(&self) -> bool {
        self.path.to_string_lossy().starts_with("/etc/systemd/")
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::service::Services;
    use maplit::{hashmap, hashset};
    use std::convert::TryInto;

    #[test]
    fn test_get_config_file_names() {
        let input_map = hashmap!(
            "foo".to_string() => model::Service {
                configuration_files: vec!["file1".try_into().unwrap()],
                restart_commands: vec!["echo hi".to_string()]
            },
            "bar".to_string() => model::Service {
                configuration_files: vec!["file1".try_into().unwrap(), "file2".try_into().unwrap()],
                restart_commands: vec!["echo hi".to_string()]
            },
        );
        let services = Services::from_model_services(input_map, None);

        let expected_output = hashset! {"file1".to_string(), "file2".to_string() };

        assert_eq!(get_config_file_names(&services), expected_output)
    }
}
