//! Contains the error type for this library.

#![allow(clippy::default_trait_access)]

use crate::schema::RoleType;
use chrono::{DateTime, Utc};
use snafu::{Backtrace, Snafu};
use std::path::PathBuf;

/// Alias for `Result<T, Error>`.
pub type Result<T> = std::result::Result<T, Error>;

/// The error type for this library.
#[derive(Debug, Snafu)]
#[snafu(visibility = "pub(crate)")]
pub enum Error {
    /// The library failed to create a file in the datastore.
    #[snafu(display("Failed to create file at datastore path {}: {}", path.display(), source))]
    DatastoreCreate {
        path: PathBuf,
        source: std::io::Error,
        backtrace: Backtrace,
    },

    /// The library failed to open a file in the datastore.
    #[snafu(display("Failed to open file from datastore path {}: {}", path.display(), source))]
    DatastoreOpen {
        path: PathBuf,
        source: std::io::Error,
        backtrace: Backtrace,
    },

    /// The library failed to remove a file in the datastore.
    #[snafu(display("Failed to remove file at datastore path {}: {}", path.display(), source))]
    DatastoreRemove {
        path: PathBuf,
        source: std::io::Error,
        backtrace: Backtrace,
    },

    /// The library failed to serialize an object to JSON to the datastore.
    #[snafu(display("Failed to serialize {} to JSON at datastore path {}: {}", what, path.display(), source))]
    DatastoreSerialize {
        what: String,
        path: PathBuf,
        source: serde_json::Error,
        backtrace: Backtrace,
    },

    /// A metadata file has expired.
    #[snafu(display("{} metadata is expired", role))]
    ExpiredMetadata {
        role: RoleType,
        backtrace: Backtrace,
    },

    /// A downloaded target's checksum does not match the checksum listed in the repository
    /// metadata.
    #[snafu(display(
        "Hash mismatch for {}: calculated {}, expected {}",
        context,
        calculated,
        expected,
    ))]
    HashMismatch {
        context: String,
        calculated: String,
        expected: String,
        backtrace: Backtrace,
    },

    /// The library failed to create a URL from a base URL and a path.
    #[snafu(display("Failed to join \"{}\" to URL \"{}\": {}", path, url, source))]
    JoinUrl {
        path: String,
        url: url::Url,
        source: url::ParseError,
        backtrace: Backtrace,
    },

    /// A file's maximum size exceeded a limit set by the consumer of this library or the metadata.
    #[snafu(display("Maximum size {} (specified by {}) exceeded", max_size, specifier))]
    MaxSizeExceeded {
        max_size: u64,
        specifier: &'static str,
        backtrace: Backtrace,
    },

    /// The maximum root updates setting was exceeded.
    #[snafu(display("Maximum root updates {} exceeded", max_root_updates))]
    MaxUpdatesExceeded {
        max_root_updates: u64,
        backtrace: Backtrace,
    },

    /// A required reference to a metadata file is missing from a metadata file.
    #[snafu(display("Meta for {:?} missing from {} metadata", file, role))]
    MetaMissing {
        file: &'static str,
        role: RoleType,
        backtrace: Backtrace,
    },

    /// A downloaded metadata file has an older version than a previously downloaded metadata file.
    #[snafu(display(
        "Found version {} of {} metadata when we had previously fetched version {}",
        new_version,
        role,
        current_version
    ))]
    OlderMetadata {
        role: RoleType,
        current_version: u64,
        new_version: u64,
        backtrace: Backtrace,
    },

    /// The library failed to parse a metadata file, either because it was not valid JSON or it did
    /// not conform to the expected schema.
    //
    // Invalid JSON errors read like:
    // * EOF while parsing a string at line 1 column 14
    //
    // Schema non-conformance errors read like:
    // * invalid type: integer `2`, expected a string at line 1 column 11
    // * missing field `sig` at line 1 column 16
    #[snafu(display("Failed to parse {} metadata: {}", role, source))]
    ParseMetadata {
        role: RoleType,
        source: serde_json::Error,
        backtrace: Backtrace,
    },

    /// The library failed to parse the trusted root metadata file, either because it was not valid
    /// JSON or it did not conform to the expected schema. The *trusted* root metadata file is the
    /// file is either the `root` argument passed to `Repository::load`, or the most recently
    /// cached and validated root metadata file.
    #[snafu(display("Failed to parse trusted root metadata: {}", source))]
    ParseTrustedMetadata {
        source: serde_json::Error,
        backtrace: Backtrace,
    },

    /// Failed to parse a URL provided to [`Repository::load`][crate::Repository::load].
    #[snafu(display("Failed to parse URL {:?}: {}", url, source))]
    ParseUrl {
        url: String,
        source: url::ParseError,
        backtrace: Backtrace,
    },

    /// System time is behaving irrationally, went back in time
    #[snafu(display(
        "System time stepped backward: system time '{}', last known time '{}'",
        sys_time,
        latest_known_time,
    ))]
    SystemTimeSteppedBackward {
        sys_time: DateTime<Utc>,
        latest_known_time: DateTime<Utc>,
    },

    /// A transport error occurred while fetching a URL.
    #[snafu(display("Failed to fetch {}: {}", url, source))]
    Transport {
        url: url::Url,
        source: Box<dyn std::error::Error + Send + Sync>,
        backtrace: Backtrace,
    },

    /// A metadata file could not be verified.
    #[snafu(display("Failed to verify {} metadata: {}", role, source))]
    VerifyMetadata {
        role: RoleType,
        source: crate::schema::Error,
        backtrace: Backtrace,
    },

    /// The trusted root metadata file could not be verified.
    #[snafu(display("Failed to verify trusted root metadata: {}", source))]
    VerifyTrustedMetadata {
        source: crate::schema::Error,
        backtrace: Backtrace,
    },

    /// A fetched metadata file did not have the version we expected it to have.
    #[snafu(display(
        "{} metadata version mismatch: fetched {}, expected {}",
        role,
        fetched,
        expected
    ))]
    VersionMismatch {
        role: RoleType,
        fetched: u64,
        expected: u64,
        backtrace: Backtrace,
    },
}

// used in `std::io::Read` implementations
impl From<Error> for std::io::Error {
    fn from(err: Error) -> Self {
        Self::new(std::io::ErrorKind::Other, err)
    }
}
