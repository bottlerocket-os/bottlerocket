//! schnauzer-v2
//!
//! A settings generator for rendering handlebars templates using data from the Bottlerocket API.
use self::clirequirements::CLIExtensionRequirement;
use argh::FromArgs;
use schnauzer::{
    render_template, render_template_file,
    template::{ExtensionRequirement, Template, TemplateFrontmatter},
    BottlerocketTemplateImporter,
};
use simplelog::{Config as LogConfig, LevelFilter, SimpleLogger};
use snafu::ResultExt;
use std::{path::PathBuf, process};

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

async fn run() -> Result<()> {
    let args: Args = argh::from_env();

    SimpleLogger::init(args.log_level, LogConfig::default()).context(error::LoggingSetupSnafu)?;

    let importer = BottlerocketTemplateImporter::new(args.api_socket);

    let rendered_template = match args.subcommand {
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

            render_template(&importer, &template)
                .await
                .context(error::RenderTemplateSnafu)?
        }
        Subcommand::RenderFile(RenderFileArgs { path }) => render_template_file(&importer, &path)
            .await
            .context(error::RenderTemplateSnafu)?,
    };

    println!(
        "{}",
        serde_json::to_string(&rendered_template).context(error::JSONSerializeSnafu)?
    );
    Ok(())
}

// Returning a Result from main makes it print a Debug representation of the error, but with Snafu
// we have nice Display representations of the error, so we wrap "main" (run) and print any error.
// https://github.com/shepmaster/snafu/issues/110
#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("{}", e);
        process::exit(1);
    }
}

pub mod error {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub))]
    pub enum Error {
        #[snafu(display("Failed to parse template requirements: '{}'", source))]
        FrontmatterParse { source: schnauzer::template::Error },

        #[snafu(display("Failed to write output to JSON: '{}'", source))]
        JSONSerialize { source: serde_json::Error },

        #[snafu(display("Failed to initialize logging: '{}'", source))]
        LoggingSetup { source: log::SetLoggerError },

        #[snafu(display("Failed to render template: '{}'", source))]
        RenderTemplate { source: schnauzer::RenderError },

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

pub use error::Error;
type Result<T> = std::result::Result<T, error::Error>;
