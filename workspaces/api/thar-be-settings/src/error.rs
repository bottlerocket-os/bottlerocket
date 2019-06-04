use snafu::Snafu;
use std::io;
use std::path::PathBuf;

/// Potential errors during configuration application
#[derive(Debug, Snafu)]
#[snafu(visibility = "pub(crate)")]
pub enum TBSError {
    #[snafu(display("Failed to read changed settings from {}", location))]
    ReadInput {
        location: &'static str,
        source: io::Error,
    },

    #[snafu(display("{} - input '{}' - {}", reason, input, source))]
    InvalidInput {
        reason: &'static str,
        input: String,
        source: serde_json::error::Error,
    },

    #[snafu(display("Failed to write template {} to disk at {}: {}", pathtype, path.display(), source))]
    TemplateWrite {
        path: PathBuf,
        pathtype: &'static str,
        source: io::Error,
    },

    #[snafu(display("Restart command failed - '{}': {}", command, source))]
    FailedRestartCommand { command: String, source: io::Error },

    #[snafu(display("Restart command is invalid (empty, space prefix, etc.) - {}", command))]
    InvalidRestartCommand { command: String },

    #[snafu(display("Configuration file '{}' failed to render: {}", template, source))]
    TemplateRender {
        template: String,
        source: handlebars::RenderError,
    },

    #[snafu(display("Failure to read template '{}' from '{}': {}", name, path.display(), source))]
    TemplateRegister {
        name: String,
        path: PathBuf,
        source: handlebars::TemplateFileError,
    },

    #[snafu(display("Error sending {} to '{}': {}", method, uri, source))]
    APIRequest {
        method: &'static str,
        uri: String,
        source: reqwest::Error,
    },

    #[snafu(display("Error response from {} to '{}': {}", method, uri, source))]
    APIResponse {
        method: &'static str,
        uri: String,
        source: reqwest::Error,
    },

    #[snafu(display(
        "Error deserializing response as JSON from {} to '{}': {}",
        method,
        uri,
        source
    ))]
    ResponseJson {
        method: &'static str,
        uri: String,
        source: reqwest::Error,
    },
}
