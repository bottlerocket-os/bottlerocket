//! This module contains utilities for constructing a Handlebars registry to render a given template.
//!
//! The registry is built after inspecting the template frontmatter to determine which settings extensions and handlebar
//! helpers should be used.
use super::import::helpers::HelperResolver;
use super::template::TemplateFrontmatter;
use handlebars::Handlebars;
use snafu::ResultExt;

/// Creates a Handlebars registry which can be used to render a template with the given frontmatter.
pub async fn construct_handlebars_registry<'a>(
    helper_resolver: &impl HelperResolver,
    frontmatter: &'a TemplateFrontmatter,
) -> Result<Handlebars<'a>> {
    let mut template_registry = Handlebars::new();

    // Strict mode will panic if a key exists in the template
    // but isn't provided in the data given to the renderer.
    template_registry.set_strict_mode(true);

    register_requested_helpers(helper_resolver, &mut template_registry, frontmatter).await?;

    Ok(template_registry)
}

/// Makes all requested settings extensions' helpers available to the template registry.
async fn register_requested_helpers(
    helper_resolver: &impl HelperResolver,
    template_registry: &mut Handlebars<'_>,
    frontmatter: &TemplateFrontmatter,
) -> Result<()> {
    for requirements in frontmatter.extension_requirements() {
        helper_resolver
            .register_template_helpers(template_registry, &requirements)
            .await
            .context(error::RegisterExtensionHelpersSnafu {
                setting_extension: requirements.name.clone(),
            })?
    }
    Ok(())
}

pub mod error {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub))]
    pub enum Error {
        #[snafu(display(
            "Failed to register template helpers from extension '{}': '{}'",
            setting_extension,
            source
        ))]
        RegisterExtensionHelpers {
            source: Box<dyn std::error::Error>,
            setting_extension: String,
        },
    }
}

pub use error::Error;
type Result<T> = std::result::Result<T, error::Error>;
