use crate::service::Services;
use crate::{error, Result};
use itertools::join;
use snafu::ResultExt;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

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
    let config_files: model::ConfigurationFiles = schnauzer::get_json(socket_path, uri, query)
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
pub fn render_config_files(
    registry: &handlebars::Handlebars<'_>,
    config_files: model::ConfigurationFiles,
    settings: model::Model,
    strict: bool,
) -> Result<Vec<RenderedConfigFile>> {
    // Go write all the configuration files from template
    let mut rendered_configs = Vec::new();
    for (name, metadata) in config_files {
        debug!("Rendering {}", &name);

        let try_rendered = registry.render(&name, &settings);
        if strict {
            let rendered = try_rendered.context(error::TemplateRenderSnafu { template: name })?;
            rendered_configs.push(RenderedConfigFile::new(&metadata.path, rendered));
        } else {
            match try_rendered {
                Ok(rendered) => {
                    rendered_configs.push(RenderedConfigFile::new(&metadata.path, rendered))
                }
                Err(err) => warn!("Unable to render template '{}': {}", &name, err),
            }
        }
    }
    trace!("Rendered configs: {:?}", &rendered_configs);
    Ok(rendered_configs)
}

/// Write all the configuration files to disk
pub fn write_config_files(rendered_config: Vec<RenderedConfigFile>) -> Result<()> {
    for cfg in rendered_config {
        debug!("Writing {:?}", &cfg.path);
        cfg.write_to_disk()?;
    }
    Ok(())
}

/// RenderedConfigFile contains both the path to the config file
/// and the rendered data to write.
#[derive(Debug)]
pub struct RenderedConfigFile {
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
        if let Some(dirname) = self.path.parent() {
            fs::create_dir_all(dirname).context(error::TemplateWriteSnafu {
                path: dirname,
                pathtype: "directory",
            })?;
        };

        fs::write(&self.path, self.rendered.as_bytes()).context(error::TemplateWriteSnafu {
            path: &self.path,
            pathtype: "file",
        })
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
