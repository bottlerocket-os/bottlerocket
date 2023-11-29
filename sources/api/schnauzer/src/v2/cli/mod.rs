//! schnauzer-v2
//!
//! A settings generator for rendering handlebars templates using data from the Bottlerocket API.
use self::clirequirements::CLIExtensionRequirement;
use crate::import::{HelperResolver, SettingsResolver, TemplateImporter};
use crate::{
    render_template, render_template_file,
    template::{ExtensionRequirement, Template, TemplateFrontmatter},
    BottlerocketTemplateImporter,
};
use argh::FromArgs;
use simplelog::{Config as LogConfig, LevelFilter, SimpleLogger};
use snafu::{OptionExt, ResultExt};
use std::path::PathBuf;

mod clirequirements;

/// Stores user-supplied arguments
#[derive(Debug, FromArgs)]
struct Args {
    /// log-level trace|debug|info|warn|error
    #[argh(option, default = "LevelFilter::Info")]
    log_level: LevelFilter,

    /// path to Bottlerocket API socket
    #[argh(option, default = "constants::API_SOCKET.into()")]
    api_socket: PathBuf,

    #[argh(subcommand)]
    subcommand: Subcommand,
}

#[derive(Debug, FromArgs)]
#[argh(subcommand)]
enum Subcommand {
    Render(RenderArgs),
    RenderFile(RenderFileArgs),
}

#[derive(Debug, FromArgs)]
#[argh(subcommand, name = "render")]
/// Render a template string
struct RenderArgs {
    /// extensions required to render this template, e.g. extension@version(helpers=[helper1, helper2])
    #[argh(option)]
    requires: Vec<CLIExtensionRequirement>,

    /// template to render
    #[argh(option)]
    template: String,
}

#[derive(Debug, FromArgs)]
#[argh(subcommand, name = "render-file")]
/// Render a template from a file
struct RenderFileArgs {
    /// template file to render
    #[argh(option)]
    path: PathBuf,
}

/// Run the schnauzer-v2 CLI from a set of parsed arguments and a custom importer.
async fn run_with_parsed_args<SR, HR>(
    args: Args,
    template_importer: &dyn TemplateImporter<SettingsResolver = SR, HelperResolver = HR>,
) -> Result<String>
where
    SR: SettingsResolver,
    HR: HelperResolver,
{
    match args.subcommand {
        Subcommand::Render(RenderArgs { requires, template }) => {
            // let frontmatter: TemplateFrontmatter = requires.try_into()?;
            let frontmatter: TemplateFrontmatter = requires
                .into_iter()
                .map(Into::<ExtensionRequirement>::into)
                .collect::<Vec<_>>()
                .try_into()
                .context(error::FrontmatterParseSnafu)?;

            let template = Template {
                frontmatter,
                body: template,
            };

            render_template(template_importer, &template)
                .await
                .context(error::RenderTemplateSnafu)
        }
        Subcommand::RenderFile(RenderFileArgs { path }) => {
            render_template_file(template_importer, &path)
                .await
                .context(error::RenderTemplateSnafu)
        }
    }
}

/// Run the schnauzer-v2 CLI, parsing arguments from the given set of strings and a given template
/// importer.
pub async fn run_with_args<I, T, SR, HR>(
    iter: I,
    template_importer: &dyn TemplateImporter<SettingsResolver = SR, HelperResolver = HR>,
) -> Result<String>
where
    I: IntoIterator<Item = T>,
    T: AsRef<str>,
    SR: SettingsResolver,
    HR: HelperResolver,
{
    let all_inputs: Vec<String> = iter.into_iter().map(|s| s.as_ref().to_string()).collect();

    let mut input_iter = all_inputs.iter().map(AsRef::as_ref);
    let command_name = [input_iter.next().context(error::ParseCLICommandSnafu)?];
    let args: Vec<&str> = input_iter.collect();

    let args =
        Args::from_args(&command_name, &args).map_err(|e| error::CLIError::ParseCLIArgs {
            parser_output: e.output,
        })?;

    run_with_parsed_args(args, template_importer).await
}

/// Run the schnauzer-v2 CLI, parsing arguments from the command line.
///
/// Uses the BottlerocketTemplateImporter, which reads settings and helpers from the Bottlerocket
/// API.
pub async fn run() -> Result<()> {
    let args: Args = argh::from_env();
    SimpleLogger::init(args.log_level, LogConfig::default()).context(error::LoggingSetupSnafu)?;

    let template_importer = BottlerocketTemplateImporter::new(args.api_socket.clone());

    let rendered_template = run_with_parsed_args(args, &template_importer).await?;

    println!(
        "{}",
        serde_json::to_string(&rendered_template).context(error::JSONSerializeSnafu)?
    );
    Ok(())
}

pub mod error {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub))]
    pub enum CLIError {
        #[snafu(display("Failed to parse template requirements: '{}'", source))]
        FrontmatterParse { source: crate::template::Error },

        #[snafu(display("Failed to write output to JSON: '{}'", source))]
        JSONSerialize { source: serde_json::Error },

        #[snafu(display("Failed to initialize logging: '{}'", source))]
        LoggingSetup { source: log::SetLoggerError },

        #[snafu(display("Failed to parse CLI arguments: {}", parser_output))]
        ParseCLIArgs { parser_output: String },

        #[snafu(display("Failed to parse CLI arguments: No CLI command given"))]
        ParseCLICommand,

        #[snafu(display("Failed to render template: '{}'", source))]
        RenderTemplate { source: crate::RenderError },

        #[snafu(display(
            "Could not parse extension requirement from '{}': {}'",
            requirement,
            reason
        ))]
        RequirementsParse {
            requirement: String,
            reason: &'static str,
        },
    }
}

pub use error::CLIError;
type Result<T> = std::result::Result<T, error::CLIError>;
