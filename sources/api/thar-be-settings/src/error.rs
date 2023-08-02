use core::num;
use http::StatusCode;
use snafu::Snafu;
use std::io;
use std::path::PathBuf;

/// Potential errors during configuration application
#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum Error {
    #[snafu(display("Failed to read changed settings from {}", from))]
    ReadInput {
        from: &'static str,
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

    #[snafu(display("Failed to set template {} to mode {}: {}", path.display(), mode, source))]
    TemplateMode {
        path: PathBuf,
        mode: String,
        source: num::ParseIntError,
    },

    #[snafu(display("Failed to run restart command - '{}': {}", command, source))]
    CommandExecutionFailure { command: String, source: io::Error },

    #[snafu(display("Reload command failed - '{}': {}", command, stderr))]
    FailedReloadCommand { command: String, stderr: String },

    #[snafu(display("Restart command failed - '{}': {}", command, stderr))]
    FailedRestartCommand { command: String, stderr: String },

    #[snafu(display("Restart command is invalid (empty, space prefix, etc.) - {}", command))]
    InvalidRestartCommand { command: String },

    #[snafu(display("Configuration file '{}' failed to render: {}", template, source))]
    TemplateRender {
        template: String,
        #[snafu(source(from(schnauzer::RenderError, Box::new)))]
        source: Box<schnauzer::RenderError>,
    },

    #[snafu(display("Error sending {} to {}: {}", method, uri, source))]
    APIRequest {
        method: String,
        uri: String,
        #[snafu(source(from(apiclient::Error, Box::new)))]
        source: Box<apiclient::Error>,
    },

    #[snafu(display("Error {} when sending {} to {}: {}", code, method, uri, response_body))]
    APIResponse {
        method: String,
        uri: String,
        code: StatusCode,
        response_body: String,
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
        source: serde_json::Error,
    },

    #[snafu(display("Error GETing JSON from '{}': {}", uri, source))]
    GetJson {
        uri: String,
        source: schnauzer::v1::Error,
    },
}
