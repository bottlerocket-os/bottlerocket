//! The template module owns the finding and rendering of parameter templates that used to generate
//! SSM parameter names and values.

use super::{BuildContext, SsmKey, SsmParameters};
use crate::aws::ami::Image;
use log::{info, trace};
use rusoto_core::Region;
use serde::{Deserialize, Serialize};
use snafu::{ensure, ResultExt};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tinytemplate::TinyTemplate;

/// Represents a single SSM parameter
#[derive(Debug, Deserialize)]
pub(crate) struct TemplateParameter {
    pub(crate) name: String,
    pub(crate) value: String,
}

/// Represents a set of SSM parameters, in a format that allows for clear definition of
/// parameters in TOML files
#[derive(Debug, Deserialize)]
pub(crate) struct TemplateParameters {
    // In a TOML table, it's clearer to define a single entry as a "parameter".
    #[serde(default, rename = "parameter")]
    pub(crate) parameters: Vec<TemplateParameter>,
}

impl TemplateParameters {
    fn extend(&mut self, other: Self) {
        self.parameters.extend(other.parameters)
    }
}

/// Finds and deserializes template parameters from the template directory, taking into account
/// overrides requested by the user
pub(crate) fn get_parameters(
    template_dir: &Path,
    build_context: &BuildContext<'_>,
) -> Result<TemplateParameters> {
    let defaults_path = template_dir.join("defaults.toml");
    let defaults_str = fs::read_to_string(&defaults_path).context(error::File {
        op: "read",
        path: &defaults_path,
    })?;
    let mut template_parameters: TemplateParameters =
        toml::from_str(&defaults_str).context(error::InvalidToml {
            path: &defaults_path,
        })?;
    trace!("Parsed default templates: {:#?}", template_parameters);

    // Allow the user to add/override parameters specific to variant or arch.  Because these are
    // added after the defaults, they will take precedence. (It doesn't make sense to override
    // based on the version argument.)
    let mut context = HashMap::new();
    context.insert("variant", build_context.variant);
    context.insert("arch", build_context.arch);
    for (key, value) in context {
        let override_path = template_dir.join(key).join(format!("{}.toml", value));
        if override_path.exists() {
            info!(
                "Parsing SSM parameter overrides from {}",
                override_path.display()
            );
            let template_str = fs::read_to_string(&override_path).context(error::File {
                op: "read",
                path: &override_path,
            })?;
            let override_parameters: TemplateParameters =
                toml::from_str(&template_str).context(error::InvalidToml {
                    path: &override_path,
                })?;
            trace!("Parsed override templates: {:#?}", override_parameters);
            template_parameters.extend(override_parameters);
        }
    }

    ensure!(
        !template_parameters.parameters.is_empty(),
        error::NoTemplates { path: template_dir }
    );

    Ok(template_parameters)
}

/// Render the given template parameters using the data from the given AMIs
pub(crate) fn render_parameters(
    template_parameters: TemplateParameters,
    amis: HashMap<Region, Image>,
    ssm_prefix: &str,
    build_context: &BuildContext<'_>,
) -> Result<SsmParameters> {
    /// Values that we allow as template variables
    #[derive(Debug, Serialize)]
    struct TemplateContext<'a> {
        variant: &'a str,
        arch: &'a str,
        image_id: &'a str,
        image_name: &'a str,
        image_version: &'a str,
        region: &'a str,
    }
    let mut new_parameters = HashMap::new();
    for (region, image) in amis {
        let context = TemplateContext {
            variant: build_context.variant,
            arch: build_context.arch,
            image_id: &image.id,
            image_name: &image.name,
            image_version: build_context.image_version,
            region: region.name(),
        };

        for tp in &template_parameters.parameters {
            let mut tt = TinyTemplate::new();
            tt.add_template("name", &tp.name)
                .context(error::AddTemplate { template: &tp.name })?;
            tt.add_template("value", &tp.value)
                .context(error::AddTemplate {
                    template: &tp.value,
                })?;
            let name_suffix = tt
                .render("name", &context)
                .context(error::RenderTemplate { template: &tp.name })?;
            let value = tt
                .render("value", &context)
                .context(error::RenderTemplate {
                    template: &tp.value,
                })?;

            new_parameters.insert(
                SsmKey::new(region.clone(), join_name(ssm_prefix, &name_suffix)),
                value,
            );
        }
    }

    Ok(new_parameters)
}

/// Render the names of the given template parameters using the fixed data about the current build.
/// Returns a mapping of templated name to rendered name, so we can associate rendered names to a
/// common source name
pub(crate) fn render_parameter_names(
    template_parameters: &TemplateParameters,
    ssm_prefix: &str,
    build_context: &BuildContext<'_>,
) -> Result<HashMap<String, String>> {
    let mut new_parameters = HashMap::new();
    for tp in &template_parameters.parameters {
        let mut tt = TinyTemplate::new();
        tt.add_template("name", &tp.name)
            .context(error::AddTemplate { template: &tp.name })?;
        let name_suffix = tt
            .render("name", &build_context)
            .context(error::RenderTemplate { template: &tp.name })?;
        new_parameters.insert(tp.name.clone(), join_name(ssm_prefix, &name_suffix));
    }

    Ok(new_parameters)
}

/// Make sure prefix and parameter name are separated by one slash
fn join_name(ssm_prefix: &str, name_suffix: &str) -> String {
    if ssm_prefix.ends_with('/') && name_suffix.starts_with('/') {
        format!("{}{}", ssm_prefix, &name_suffix[1..])
    } else if ssm_prefix.ends_with('/') || name_suffix.starts_with('/') {
        format!("{}{}", ssm_prefix, name_suffix)
    } else {
        format!("{}/{}", ssm_prefix, name_suffix)
    }
}

mod error {
    use snafu::Snafu;
    use std::io;
    use std::path::PathBuf;

    #[derive(Debug, Snafu)]
    #[snafu(visibility = "pub(super)")]
    pub(crate) enum Error {
        #[snafu(display("Error building template from '{}': {}", template, source))]
        AddTemplate {
            template: String,
            source: tinytemplate::error::Error,
        },

        #[snafu(display("Failed to {} '{}': {}", op, path.display(), source))]
        File {
            op: String,
            path: PathBuf,
            source: io::Error,
        },

        #[snafu(display("Invalid config file at '{}': {}", path.display(), source))]
        InvalidToml {
            path: PathBuf,
            source: toml::de::Error,
        },

        #[snafu(display("Found no parameter templates in {}", path.display()))]
        NoTemplates {
            path: PathBuf,
        },

        #[snafu(display("Error rendering template from '{}': {}", template, source))]
        RenderTemplate {
            template: String,
            source: tinytemplate::error::Error,
        },
    }
}
pub(crate) use error::Error;
type Result<T> = std::result::Result<T, error::Error>;
