#![allow(clippy::default_trait_access)]

use snafu::{Backtrace, Snafu};
use std::path::PathBuf;

#[cfg(any(feature = "rusoto-native-tls", feature = "rusoto-rustls"))]
use crate::deref::OptionDeref;

pub(crate) type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub(crate)")]
pub(crate) enum Error {
    #[snafu(display("Cannot determine current directory: {}", source))]
    CurrentDir {
        source: std::io::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("Failed to {} {} to {}: {}", action, src.display(), dst.display(), source))]
    FileCopy {
        action: crate::copylike::Copylike,
        src: PathBuf,
        dst: PathBuf,
        source: std::io::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("Failed to create {}: {}", path.display(), source))]
    FileCreate {
        path: PathBuf,
        source: std::io::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("Failed to open {}: {}", path.display(), source))]
    FileOpen {
        path: PathBuf,
        source: std::io::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("Failed to parse {}: {}", path.display(), source))]
    FileParseJson {
        path: PathBuf,
        source: serde_json::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("Failed to copy {} to {}: {}", source.file.path().display(), path.display(), source.error))]
    FilePersist {
        path: PathBuf,
        source: tempfile::PersistError,
        backtrace: Backtrace,
    },

    #[snafu(display("Failed to read {}: {}", path.display(), source))]
    FileRead {
        path: PathBuf,
        source: std::io::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("Failed to create temporary file in {}: {}", path.display(), source))]
    FileTempCreate {
        path: PathBuf,
        source: std::io::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("Failed to write to {}: {}", path.display(), source))]
    FileWrite {
        path: PathBuf,
        source: std::io::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("Failed to write to {}: {}", path.display(), source))]
    FileWriteJson {
        path: PathBuf,
        source: serde_json::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("Failed to initialize global thread pool: {}", source))]
    InitializeThreadPool {
        source: rayon::ThreadPoolBuildError,
        backtrace: Backtrace,
    },

    #[snafu(display("{}: {}", path.display(), source))]
    Key {
        path: PathBuf,
        #[snafu(source(from(Error, Box::new)))]
        #[snafu(backtrace)]
        source: Box<Self>,
    },

    #[snafu(display("Duplicate key ID: {}", key_id))]
    KeyDuplicate {
        key_id: String,
        backtrace: Backtrace,
    },

    #[snafu(display("Failed to calculate key ID: {}", source))]
    KeyId {
        #[snafu(backtrace)]
        source: tough_schema::Error,
    },

    #[snafu(display("Private key rejected: {}", source))]
    KeyRejected {
        source: ring::error::KeyRejected,
        backtrace: Backtrace,
    },

    #[snafu(display("Unrecognized private key format"))]
    KeyUnrecognized { backtrace: Backtrace },

    #[snafu(display("Path {} is not valid UTF-8", path.display()))]
    PathUtf8 { path: PathBuf, backtrace: Backtrace },

    #[snafu(display("Path {} does not have a parent", path.display()))]
    PathParent { path: PathBuf, backtrace: Backtrace },

    // the source error is zero-sized with a fixed message, no sense in displaying it
    #[snafu(display("Path {} is not within {}", path.display(), base.display()))]
    Prefix {
        path: PathBuf,
        base: PathBuf,
        source: std::path::StripPrefixError,
        backtrace: Backtrace,
    },

    #[cfg(any(feature = "rusoto-native-tls", feature = "rusoto-rustls"))]
    #[snafu(display("Error creating AWS credentials provider: {}", source))]
    RusotoCreds {
        source: rusoto_credential::CredentialsError,
        backtrace: Backtrace,
    },

    #[cfg(any(feature = "rusoto-native-tls", feature = "rusoto-rustls"))]
    #[snafu(display("Unknown AWS region \"{}\": {}", region, source))]
    RusotoRegion {
        region: String,
        source: rusoto_core::region::ParseRegionError,
        backtrace: Backtrace,
    },

    #[cfg(any(feature = "rusoto-native-tls", feature = "rusoto-rustls"))]
    #[snafu(display("Error creating AWS request dispatcher: {}", source))]
    RusotoTls {
        source: rusoto_core::request::TlsError,
        backtrace: Backtrace,
    },

    #[snafu(display("Failed to sign message"))]
    Sign {
        source: ring::error::Unspecified,
        backtrace: Backtrace,
    },

    #[snafu(display("Failed to serialize role for signing: {}", source))]
    SignJson {
        source: serde_json::Error,
        backtrace: Backtrace,
    },

    #[cfg(any(feature = "rusoto-native-tls", feature = "rusoto-rustls"))]
    #[snafu(display(
        "Failed to get aws-ssm://{}{}: {}",
        profile.deref_shim().unwrap_or(""),
        parameter_name,
        source,
    ))]
    SsmGetParameter {
        profile: Option<String>,
        parameter_name: String,
        source: rusoto_core::RusotoError<rusoto_ssm::GetParameterError>,
        backtrace: Backtrace,
    },

    #[cfg(any(feature = "rusoto-native-tls", feature = "rusoto-rustls"))]
    #[snafu(display("Missing field in SSM response: {}", field))]
    SsmMissingField {
        field: &'static str,
        backtrace: Backtrace,
    },

    #[snafu(display("Unrecognized or invalid public key"))]
    UnrecognizedKey { backtrace: Backtrace },

    #[snafu(display("Unrecognized URL scheme \"{}\"", scheme))]
    UnrecognizedScheme {
        scheme: String,
        backtrace: Backtrace,
    },

    #[snafu(display("Failed to parse URL \"{}\": {}", url, source))]
    UrlParse {
        url: String,
        source: url::ParseError,
        backtrace: Backtrace,
    },

    #[snafu(display("Failed to walk directory tree: {}", source))]
    WalkDir {
        source: walkdir::Error,
        backtrace: Backtrace,
    },
}
