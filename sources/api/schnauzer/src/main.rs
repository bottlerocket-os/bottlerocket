/*!
# Introduction

schnauzer is called by sundog as a setting generator.
Its sole parameter is the name of the setting to generate.

The setting we're generating is expected to have a metadata key already set: "template".
"template" is an arbitrary string with mustache template variables that reference other settings.

For example, if we're generating "settings.x" and we have template "foo-{{ settings.bar }}", we look up the value of "settings.bar" in the API.
If the returned value is "baz", our generated value will be "foo-baz".

(The name "schnauzer" comes from the fact that Schnauzers are search and rescue dogs (similar to this search and replace task) and because they have mustaches.)
*/

use snafu::{ensure, OptionExt, ResultExt};
use std::collections::HashMap;
use std::string::String;
use std::{env, process};

// Setting generators do not require dynamic socket paths at this moment.
const API_METADATA_URI_BASE: &str = "/metadata/";

mod error {
    use http::StatusCode;
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub(super) enum Error {
        #[snafu(display("Error {}ing to {}: {}", method, uri, source))]
        APIRequest {
            method: String,
            uri: String,
            #[snafu(source(from(apiclient::Error, Box::new)))]
            source: Box<apiclient::Error>,
        },

        #[snafu(display("Error {} when {}ing to '{}': {}", code, method, uri, response_body))]
        Response {
            method: String,
            uri: String,
            code: StatusCode,
            response_body: String,
        },

        #[snafu(display("Error deserializing to JSON: {}", source))]
        DeserializeJson { source: serde_json::error::Error },

        #[snafu(display("Error serializing to JSON '{}': {}", output, source))]
        SerializeOutput {
            output: String,
            source: serde_json::error::Error,
        },

        #[snafu(display("Missing metadata {} for key: {}", meta, key))]
        MissingMetadata { meta: String, key: String },

        #[snafu(display("Metadata {} expected to be {}, got: {}", meta, expected, value))]
        MetadataWrongType {
            meta: String,
            expected: String,
            value: String,
        },

        #[snafu(display("Failed to build template registry: {}", source))]
        BuildTemplateRegistry { source: schnauzer::Error },

        #[snafu(display("Failed to get settings from API: {}", source))]
        GetSettings { source: schnauzer::Error },

        #[snafu(display(
            "Failed to render setting '{}' from template '{}': {}",
            setting_name,
            template,
            source
        ))]
        RenderTemplate {
            setting_name: String,
            template: String,
            #[snafu(source(from(handlebars::RenderError, Box::new)))]
            source: Box<handlebars::RenderError>,
        },
    }
}
type Result<T> = std::result::Result<T, error::Error>;

/// Returns the value of a metadata key for a given data key, erroring if the value is not a
/// string or is empty.
async fn get_metadata(key: &str, meta: &str) -> Result<String> {
    let uri = &format!("{}{}?keys={}", API_METADATA_URI_BASE, meta, key);
    let method = "GET";
    let (code, response_body) = apiclient::raw_request(constants::API_SOCKET, &uri, method, None)
        .await
        .context(error::APIRequestSnafu { method, uri })?;
    ensure!(
        code.is_success(),
        error::ResponseSnafu {
            method,
            uri,
            code,
            response_body
        }
    );

    // Metadata responses are of the form `{"data_key": METADATA}` so we pull out the value.
    let mut response_map: HashMap<String, serde_json::Value> =
        serde_json::from_str(&response_body).context(error::DeserializeJsonSnafu)?;
    let response_val = response_map
        .remove(key)
        .context(error::MissingMetadataSnafu { meta, key })?;

    // Ensure it's a non-empty string
    let response_str = response_val
        .as_str()
        .with_context(|| error::MetadataWrongTypeSnafu {
            meta,
            expected: "string",
            value: response_val.to_string(),
        })?;
    ensure!(
        !response_str.is_empty(),
        error::MissingMetadataSnafu { meta, key }
    );
    Ok(response_str.to_string())
}

/// Print usage message.
fn usage() -> ! {
    let program_name = env::args().next().unwrap_or_else(|| "program".to_string());
    eprintln!("Usage: {} SETTING_KEY", program_name);
    process::exit(2);
}

/// Parses args for the setting key name.
fn parse_args(mut args: env::Args) -> String {
    let arg = args.nth(1).unwrap_or_else(|| "--help".to_string());
    if arg == "--help" || arg == "-h" {
        usage()
    }
    arg
}

async fn run() -> Result<()> {
    let setting_name = parse_args(env::args());

    let registry =
        schnauzer::build_template_registry().context(error::BuildTemplateRegistrySnafu)?;
    let template = get_metadata(&setting_name, "templates").await?;
    let settings = schnauzer::get_settings(constants::API_SOCKET)
        .await
        .context(error::GetSettingsSnafu)?;

    let setting =
        registry
            .render_template(&template, &settings)
            .context(error::RenderTemplateSnafu {
                setting_name,
                template,
            })?;

    // sundog expects JSON-serialized output so that many types can be represented, allowing the
    // API model to use more accurate types.
    let output = serde_json::to_string(&setting)
        .context(error::SerializeOutputSnafu { output: &setting })?;

    println!("{}", output);
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
