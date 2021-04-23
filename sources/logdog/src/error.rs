//! Provides the list of errors for `logdog`.

use std::io;
use std::path::PathBuf;

use reqwest::Url;
use snafu::{Backtrace, Snafu};

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub(crate)")]
pub(crate) enum Error {
    #[snafu(display("Error creating the command stderr file '{}': {}", path.display(), source))]
    CommandErrFile {
        source: io::Error,
        path: PathBuf,
        backtrace: Backtrace,
    },

    #[snafu(display("Error completing command '{}': {}", command, source))]
    CommandFinish {
        command: String,
        source: io::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("The request has no command: '{}'", request))]
    CommandMissing {
        request: String,
        backtrace: Backtrace,
    },

    #[snafu(display("Error creating the command stdout file '{}': {}", path.display(), source))]
    CommandOutputFile {
        source: io::Error,
        path: PathBuf,
        backtrace: Backtrace,
    },

    #[snafu(display("Error parsing command '{}': {}", command, source))]
    CommandParse {
        source: shell_words::ParseError,
        command: String,
        backtrace: Backtrace,
    },

    #[snafu(display("Error starting command '{}': {}", command, source))]
    CommandSpawn {
        command: String,
        source: io::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("The output directory '{}' could not be created: {}.", path.display(), source))]
    CreateOutputDirectory {
        source: io::Error,
        path: PathBuf,
        backtrace: Backtrace,
    },

    #[snafu(display("Error creating the error file '{}': {}", path.display(), source))]
    ErrorFile {
        source: io::Error,
        path: PathBuf,
        backtrace: Backtrace,
    },

    #[snafu(display("Error writing to the error file '{}': {}", path.display(), source))]
    ErrorWrite {
        source: io::Error,
        path: PathBuf,
        backtrace: Backtrace,
    },

    #[snafu(display("Unable to copy file from '{}' to '{}' for request '{}': {}", from, to.display(), request, source))]
    FileCopy {
        source: std::io::Error,
        request: String,
        from: String,
        to: PathBuf,
    },

    #[snafu(display("No file to copy from given for request '{}'", request))]
    FileFromEmpty { request: String },

    #[snafu(display("Output filename is missing in request: '{}'", request))]
    FilenameMissing { request: String },

    #[snafu(display("Unable to create HTTP client for '{}': {}", url, source))]
    HttpClient { url: Url, source: reqwest::Error },

    #[snafu(display("HTTP request '{}' has no URL", request))]
    HttpMissingUrl { request: String },

    #[snafu(display("HTTP error for '{}': {}", url, source))]
    HttpResponse { url: Url, source: reqwest::Error },

    #[snafu(display("HTTP response body for '{}' could not be read: {}", request, source))]
    HttpResponseBytes {
        request: String,
        source: reqwest::Error,
    },

    #[snafu(display("Unable to send HTTP request to '{}': {}", url, source))]
    HttpSend { url: Url, source: reqwest::Error },

    #[snafu(display("Unable to parse '{}' to a URL: {}", url, source))]
    HttpUrlParse {
        url: String,
        source: url::ParseError,
    },

    #[snafu(display(
    "Unable to write HTTP response for '{}' to '{}': {}",
    request,
    path.display(),
    source
    ))]
    HttpWriteBytes {
        request: String,
        path: PathBuf,
        source: std::io::Error,
    },

    #[snafu(display("Empty command."))]
    ModeMissing {},

    #[snafu(display("Error parsing glob pattern '{}': {}", pattern, source))]
    ParseGlobPattern {
        pattern: String,
        source: glob::PatternError,
    },

    #[snafu(display("The logdog configuration has a 'glob' line with no glob instructions."))]
    PatternMissing {},

    #[snafu(display("Cannot write to / as a file."))]
    RootAsFile { backtrace: Backtrace },

    #[snafu(display("Error closing the tarball '{}': {}", path.display(), source))]
    TarballClose {
        source: io::Error,
        path: PathBuf,
        backtrace: Backtrace,
    },

    #[snafu(display("Error creating the tarball file '{}': {}", path.display(), source))]
    TarballFileCreate {
        source: io::Error,
        path: PathBuf,
        backtrace: Backtrace,
    },

    #[snafu(display(
    "Output file '{}' can not be written to the directory that is being compressed '{}'.",
    outfile.display(),
    indir.display()
    ))]
    TarballOutputIsInInputDir {
        indir: PathBuf,
        outfile: PathBuf,
        backtrace: Backtrace,
    },

    #[snafu(display("Error writing to the tarball '{}': {}", path.display(), source))]
    TarballWrite {
        source: io::Error,
        path: PathBuf,
        backtrace: Backtrace,
    },

    #[snafu(display("Error creating tempdir: {}", source))]
    TempDirCreate {
        source: io::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("Unknown request type '{}' in '{}'", mode, request))]
    UnhandledRequest { mode: String, request: String },
}

pub(crate) type Result<T> = std::result::Result<T, Error>;
