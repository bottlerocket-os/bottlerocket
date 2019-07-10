//! Contains the error type for this library.

#![allow(clippy::default_trait_access)]

use crate::serde::Role;
use snafu::{Backtrace, Snafu};
use std::fmt::{self, Debug, Display};
use std::path::PathBuf;

/// Alias for `Result<T, Error>`.
pub type Result<T> = std::result::Result<T, Error>;

/// The error type for this library.
#[derive(Debug, Snafu)]
#[snafu(visibility = "pub(crate)")]
pub enum Error {
    /// A repository base URL provided to [`Repository::load`][crate::Repository::load] is missing
    /// a trailing slash.
    #[snafu(display("Base URL {:?} is missing trailing slash", url))]
    BaseUrlMissingTrailingSlash { url: String, backtrace: Backtrace },

    /// The library failed to create a file in the datastore.
    #[snafu(display("Failed to create file at datastore path {}: {}", path.display(), source))]
    DatastoreCreate {
        path: PathBuf,
        source: std::io::Error,
        backtrace: Backtrace,
    },

    /// The library failed to get metadata for a file in the datastore.
    #[snafu(display("Failed to get file metadata from datastore path {}: {}", path.display(), source))]
    DatastoreMetadata {
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

    /// A file in the datastore has insecure permissions (writable by group or others). This
    /// library does not attempt to correct insecure files.
    #[snafu(display("Datastore path {} is writable by its group or others (mode {:o})", path.display(), mode))]
    DatastorePermissions {
        path: PathBuf,
        mode: u32,
        backtrace: Backtrace,
    },

    /// The library failed to remove a file in the datastore.
    #[snafu(display("Failed to remove file at datastore path {}: {}", path.display(), source))]
    DatastoreRemove {
        path: PathBuf,
        source: std::io::Error,
        backtrace: Backtrace,
    },

    /// A duplicate key ID was present in the root metadata.
    #[snafu(display("Duplicate key ID: {}", keyid))]
    DuplicateKeyId { keyid: String },

    /// A metadata file has expired.
    #[snafu(display("{} metadata is expired", role))]
    ExpiredMetadata { role: Role, backtrace: Backtrace },

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

    /// Failed to decode a hexadecimal-encoded string.
    #[snafu(display("Invalid hex string: {}", source))]
    HexDecode {
        source: hex::FromHexError,
        backtrace: Backtrace,
    },

    /// The library failed to create a URL from a base URL and a path.
    #[snafu(display("Failed to join \"{}\" to URL \"{}\": {}", path, url, source))]
    JoinUrl {
        path: String,
        url: reqwest::Url,
        source: reqwest::UrlError,
        backtrace: Backtrace,
    },

    /// The library failed to serialize an object to JSON.
    #[snafu(display("Failed to serialize {} to JSON: {}", what, source))]
    JsonSerialization {
        what: String,
        source: serde_json::Error,
        backtrace: Backtrace,
    },

    /// A key ID in the root metadata file did not match its contents.
    #[snafu(display("Key ID mismatch: calculated {}, expected {}", calculated, expected))]
    KeyIdMismatch {
        calculated: String,
        expected: String,
        backtrace: Backtrace,
    },

    /// A file's maximum size exceeded a limit set by the consumer of this library or the metadata.
    #[snafu(display("Maximum size {} exceeded", max_size))]
    MaxSizeExceeded {
        max_size: usize,
        backtrace: Backtrace,
    },

    /// A required reference to a metadata file is missing from a metadata file.
    #[snafu(display("Meta for {:?} missing from {} metadata", file, role))]
    MetaMissing {
        file: &'static str,
        role: Role,
        backtrace: Backtrace,
    },

    /// A required role is missing from the root metadata file.
    #[snafu(display("Role {} missing from root metadata", role))]
    MissingRole { role: Role, backtrace: Backtrace },

    /// A downloaded metadata file has an older version than a previously downloaded metadata file.
    #[snafu(display(
        "Found version {} of {} metadata when we had previously fetched version {}",
        new_version,
        role,
        current_version
    ))]
    OlderMetadata {
        role: Role,
        current_version: u64,
        new_version: u64,
        backtrace: Backtrace,
    },

    /// The library failed to parse a metadata file, either because it was not valid JSON or it did
    /// not conform to the expected schema.
    #[snafu(display("Failed to parse {} metadata: {}", role, source))]
    ParseMetadata {
        role: Role,
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
        source: reqwest::UrlError,
        backtrace: Backtrace,
    },

    /// Failed to decode a PEM-encoded key.
    #[snafu(display("Invalid PEM string: {}", source))]
    PemDecode {
        source: Compat<pem::PemError>,
        backtrace: Backtrace,
    },

    /// An HTTP request failed.
    #[snafu(display("Failed to request \"{}\": {}", url, source))]
    Request {
        url: reqwest::Url,
        source: reqwest::Error,
        backtrace: Backtrace,
    },

    /// Failed to decode a `SubjectPublicKeyInfo` formatted RSA public key.
    #[snafu(display("Invalid SubjectPublicKeyInfo-formatted RSA public key"))]
    RsaDecode { backtrace: Backtrace },

    /// A signature threshold specified in root.json was not met when verifying a signature.
    #[snafu(display(
        "Signature threshold of {} not met for role {} ({} valid signatures)",
        threshold,
        role,
        valid,
    ))]
    SignatureThreshold {
        role: Role,
        threshold: u64,
        valid: u64,
        backtrace: Backtrace,
    },

    /// A metadata file could not be verified.
    #[snafu(display("Failed to verify {} metadata: {}", role, source))]
    VerifyMetadata {
        role: Role,
        #[snafu(source(from(Error, Box::new)))]
        source: Box<Error>,
        backtrace: Backtrace,
    },

    /// The trusted root metadata file could not be verified.
    #[snafu(display("Failed to verify trusted root metadata: {}", source))]
    VerifyTrustedMetadata {
        #[snafu(source(from(Error, Box::new)))]
        source: Box<Error>,
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
        role: Role,
        fetched: u64,
        expected: u64,
        backtrace: Backtrace,
    },
}

/// Wrapper for error types that don't impl [`std::error::Error`].
///
/// This should not have to exist, and yet...
pub struct Compat<T>(pub T);

impl<T: Debug> Debug for Compat<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl<T: Display> Display for Compat<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl<T: Debug + Display> std::error::Error for Compat<T> {}

// used in `std::io::Read` implementations
impl From<Error> for std::io::Error {
    fn from(err: Error) -> Self {
        Self::new(std::io::ErrorKind::Other, err)
    }
}
