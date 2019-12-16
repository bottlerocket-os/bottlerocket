use handlebars::Handlebars;
use snafu::ResultExt;

use crate::{error, helpers, Result};

/// Build the template registry using the ConfigFile structs
/// and let handlebars parse the templates
pub fn build_template_registry(
    files: &model::ConfigurationFiles,
) -> Result<handlebars::Handlebars> {
    let mut template_registry = Handlebars::new();
    // Strict mode will panic if a key exists in the template
    // but isn't provided in the data given to the renderer
    template_registry.set_strict_mode(true);

    debug!("Building template registry of configuration files");
    for (name, metadata) in files {
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

    // TODO if we start writing lots of helpers, registering them
    // should probably exist in a "setup" function of its own
    // that we can call from here. For now, KISS.
    template_registry.register_helper("base64_decode", Box::new(helpers::base64_decode));
    template_registry.register_helper("join_map", Box::new(helpers::join_map));
    template_registry.register_helper("default", Box::new(helpers::default));

    Ok(template_registry)
}
