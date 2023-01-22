//! This module allows application of settings from URIs or stdin.  The inputs are expected to be
//! TOML settings files, in the same format as user data, or the JSON equivalent.  The inputs are
//! pulled and applied to the API server in a single transaction.

use crate::rando;
use futures::future::{join, ready, TryFutureExt};
use futures::stream::{self, StreamExt};
use reqwest::Url;
use serde::de::{Deserialize, IntoDeserializer};
use snafu::{futures::try_future::TryFutureExt as SnafuTryFutureExt, OptionExt, ResultExt};
use std::path::Path;
use tokio::io::AsyncReadExt;

/// Reads settings in TOML or JSON format from files at the requested URIs (or from stdin, if given
/// "-"), then commits them in a single transaction and applies them to the system.
pub async fn apply<P>(socket_path: P, input_sources: Vec<String>) -> Result<()>
where
    P: AsRef<Path>,
{
    // We want to retrieve URIs in parallel because they're arbitrary and could be slow.  First, we
    // build a list of request futures, and we store the source of the data with the future for
    // inclusion in later error messages.
    let mut get_requests = Vec::with_capacity(input_sources.len());
    for input_source in &input_sources {
        let get_future = get(input_source);
        let info_future = ready(input_source);
        get_requests.push(join(info_future, get_future));
    }

    // Stream out the requests and await responses (in order).
    let get_request_stream = stream::iter(get_requests).buffered(4);
    let get_responses: Vec<(&String, Result<String>)> = get_request_stream.collect().await;

    // Reformat the responses to (model-verified) JSON we can send to the API.
    let mut changes = Vec::with_capacity(get_responses.len());
    for (input_source, get_response) in get_responses {
        let response = get_response?;
        let json = format_change(&response, input_source)?;
        changes.push((input_source, json));
    }

    // We use a specific transaction ID so we don't commit any other changes that may be pending.
    let transaction = format!("apiclient-apply-{}", rando());

    // Send the settings changes to the server in the same transaction.  (They're quick local
    // requests, so don't add the complexity of making them run concurrently.)
    for (input_source, json) in changes {
        let uri = format!("/settings?tx={}", transaction);
        let method = "PATCH";
        let (_status, _body) = crate::raw_request(&socket_path, &uri, method, Some(json))
            .await
            .context(error::PatchSnafu {
                input_source,
                uri,
                method,
            })?;
    }

    // Commit the transaction and apply it to the system.
    let uri = format!("/tx/commit_and_apply?tx={}", transaction);
    let method = "POST";
    let (_status, _body) = crate::raw_request(&socket_path, &uri, method, None)
        .await
        .context(error::CommitApplySnafu { uri })?;

    Ok(())
}

/// Retrieves the given source location and returns the result in a String.
async fn get<S>(input_source: S) -> Result<String>
where
    S: Into<String>,
{
    let input_source = input_source.into();

    // Read from stdin if "-" was given.
    if input_source == "-" {
        let mut output = String::new();
        tokio::io::stdin()
            .read_to_string(&mut output)
            .context(error::StdinReadSnafu)
            .await?;
        return Ok(output);
    }

    // Otherwise, the input should be a URI; parse it to know what kind.
    // Until reqwest handles file:// URIs: https://github.com/seanmonstar/reqwest/issues/178
    let uri = Url::parse(&input_source).context(error::UriSnafu {
        input_source: &input_source,
    })?;
    if uri.scheme() == "file" {
        // Turn the URI to a file path, and return a future that reads it.
        let path = uri.to_file_path().ok().context(error::FileUriSnafu {
            input_source: &input_source,
        })?;
        tokio::fs::read_to_string(path)
            .context(error::FileReadSnafu { input_source })
            .await
    } else {
        // Return a future that contains the text of the (non-file) URI.
        reqwest::get(uri)
            .and_then(|response| ready(response.error_for_status()))
            .and_then(|response| response.text())
            .context(error::ReqwestSnafu {
                uri: input_source,
                method: "GET",
            })
            .await
    }
}

/// Takes a string of TOML or JSON settings data, verifies that it fits the model, and reserializes
/// it to JSON for sending to the API.
fn format_change(input: &str, input_source: &str) -> Result<String> {
    // Try to parse the input as (arbitrary) TOML.  If that fails, try to parse it as JSON.
    let mut json_val = match toml::from_str::<toml::Value>(input) {
        Ok(toml_val) => {
            // We need JSON for the API.  serde lets us convert between Deserialize-able types by
            // reusing the deserializer.  Turn the TOML value into a JSON value.
            let d = toml_val.into_deserializer();
            serde_json::Value::deserialize(d).context(error::TomlToJsonSnafu { input_source })?
        }
        Err(toml_err) => {
            // TOML failed, try JSON; include the toml parsing error, because if they intended to
            // give TOML we should still tell them what was wrong with it.
            serde_json::from_str(input).context(error::InputTypeSnafu {
                input_source,
                toml_err,
            })
        }?,
    };

    // Remove outer "settings" layer before sending to API or deserializing it into the model,
    // neither of which expects it.
    let json_object = json_val
        .as_object_mut()
        .context(error::ModelTypeSnafu { input_source })?;
    let json_inner = json_object
        .remove("settings")
        .context(error::MissingSettingsSnafu { input_source })?;

    // Deserialize into the model to confirm the settings are valid.
    let _settings = model::Settings::deserialize(&json_inner)
        .context(error::ModelDeserializeSnafu { input_source })?;

    // Return JSON text we can send to the API.
    serde_json::to_string(&json_inner).context(error::JsonSerializeSnafu { input_source })
}

mod error {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub enum Error {
        #[snafu(display("Failed to commit combined settings to '{}': {}", uri, source))]
        CommitApply {
            uri: String,
            #[snafu(source(from(crate::Error, Box::new)))]
            source: Box<crate::Error>,
        },

        #[snafu(display("Failed to read given file '{}': {}", input_source, source))]
        FileRead {
            input_source: String,
            source: std::io::Error,
        },

        #[snafu(display("Given invalid file URI '{}'", input_source))]
        FileUri { input_source: String },

        #[snafu(display(
            "Input '{}' is not valid TOML or JSON.  (TOML error: {})  (JSON error: {})",
            input_source,
            toml_err,
            source
        ))]
        InputType {
            input_source: String,
            toml_err: toml::de::Error,
            source: serde_json::Error,
        },

        #[snafu(display(
            "Failed to serialize settings from '{}' to JSON: {}",
            input_source,
            source
        ))]
        JsonSerialize {
            input_source: String,
            source: serde_json::Error,
        },

        #[snafu(display(
            "Settings from '{}' did not contain a 'settings' key at top level",
            input_source
        ))]
        MissingSettings { input_source: String },

        #[snafu(display(
            "Failed to deserialize settings from '{}' into this variant's model: {}",
            input_source,
            source
        ))]
        ModelDeserialize {
            input_source: String,
            source: serde_json::Error,
        },

        #[snafu(display("Settings from '{}' are not a TOML table / JSON object", input_source))]
        ModelType { input_source: String },

        #[snafu(display(
            "Failed to {} settings from '{}' to '{}': {}",
            method,
            input_source,
            uri,
            source
        ))]
        Patch {
            input_source: String,
            uri: String,
            method: String,
            #[snafu(source(from(crate::Error, Box::new)))]
            source: Box<crate::Error>,
        },

        #[snafu(display("Failed {} request to '{}': {}", method, uri, source))]
        Reqwest {
            method: String,
            uri: String,
            source: reqwest::Error,
        },

        #[snafu(display("Failed to read standard input: {}", source))]
        StdinRead { source: std::io::Error },

        #[snafu(display(
            "Failed to translate TOML from '{}' to JSON for API: {}",
            input_source,
            source
        ))]
        TomlToJson {
            input_source: String,
            source: toml::de::Error,
        },

        #[snafu(display("Given invalid URI '{}': {}", input_source, source))]
        Uri {
            input_source: String,
            source: url::ParseError,
        },
    }
}
pub use error::Error;
pub type Result<T> = std::result::Result<T, error::Error>;
