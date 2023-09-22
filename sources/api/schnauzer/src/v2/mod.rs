use self::import::{HelperResolver, SettingsResolver, TemplateImporter};
use snafu::ResultExt;
use std::path::Path;

pub mod import;
mod registry;
pub mod template;

#[cfg(feature = "testfakes")]
pub use self::import::fake;

pub use self::import::BottlerocketTemplateImporter;
pub use template::{ExtensionRequirement, Template, TemplateFrontmatter};

/// Renders a Bottlerocket config template
pub async fn render_template<SR, HR>(
    template_importer: &dyn TemplateImporter<SettingsResolver = SR, HelperResolver = HR>,
    template: &Template,
) -> Result<String>
where
    SR: SettingsResolver,
    HR: HelperResolver,
{
    let handlebars = registry::construct_handlebars_registry(
        template_importer.helper_resolver(),
        &template.frontmatter,
    )
    .await
    .context(error::ConstructHandlebarsRegistrySnafu)?;

    let settings = template_importer
        .settings_resolver()
        .fetch_settings(template.frontmatter.extension_requirements())
        .await
        .context(error::RetrieveSettingsSnafu)?;

    handlebars
        .render_template(&template.body, &settings)
        .context(error::RenderTemplateSnafu {
            template: template.clone(),
        })
}

/// Renders a Bottlerocket config template represented by the input string.
pub async fn render_template_str<SR, HR, S>(
    template_importer: &dyn TemplateImporter<SettingsResolver = SR, HelperResolver = HR>,
    template_str: S,
) -> Result<String>
where
    SR: SettingsResolver,
    HR: HelperResolver,
    S: AsRef<str>,
{
    let template: Template = template_str
        .as_ref()
        .parse()
        .context(error::TemplateParseSnafu)?;

    render_template(template_importer, &template).await
}

/// Renders a Bottlerocket config template from a given filepath.
pub async fn render_template_file<SR, HR, P>(
    template_importer: &dyn TemplateImporter<SettingsResolver = SR, HelperResolver = HR>,
    input_file: P,
) -> Result<String>
where
    SR: SettingsResolver,
    HR: HelperResolver,
    P: AsRef<Path>,
{
    let template_str =
        std::fs::read_to_string(&input_file).context(error::TemplateFileReadSnafu)?;

    render_template_str(template_importer, &template_str)
        .await
        .context(error::TemplateFileRenderSnafu {
            filepath: input_file.as_ref().to_path_buf(),
        })
}

pub mod error {
    use super::Template;
    use std::path::PathBuf;

    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub))]
    pub enum RenderError {
        #[snafu(display("Failed to construct Handlebars registry: {}", source))]
        ConstructHandlebarsRegistry { source: super::registry::Error },

        #[snafu(display("Failed to render template '{:?}': {}", template, source))]
        RenderTemplate {
            template: Template,
            #[snafu(source(from(handlebars::RenderError, Box::new)))]
            source: Box<handlebars::RenderError>,
        },

        #[snafu(display("Failed to retrieve settings from Bottlerocket API: {}", source))]
        RetrieveSettings { source: Box<dyn std::error::Error> },

        #[snafu(display("Failed to parse template: {}", source))]
        TemplateParse { source: super::template::Error },

        #[snafu(display("Failed to read template file: {}", source))]
        TemplateFileRead { source: std::io::Error },

        #[snafu(display("Failed to render template file ('{}'): {}", filepath.display(), source))]
        TemplateFileRender {
            #[snafu(source(from(RenderError, Box::new)))]
            source: Box<RenderError>,
            filepath: PathBuf,
        },
    }
}

pub use error::RenderError;
type Result<T> = std::result::Result<T, error::RenderError>;
