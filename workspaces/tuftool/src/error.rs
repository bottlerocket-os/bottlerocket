#![allow(clippy::default_trait_access)]

use snafu::{Backtrace, Snafu};
use std::path::PathBuf;

pub(crate) type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub(crate)")]
pub(crate) enum Error {
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

    #[snafu(display("Failed to read {}: {}", path.display(), source))]
    FileRead {
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

    #[snafu(display("Private key rejected: {}", source))]
    KeyRejected {
        source: ring::error::KeyRejected,
        backtrace: Backtrace,
    },

    #[snafu(display("Unrecognized private key format"))]
    KeyUnrecognized { backtrace: Backtrace },

    #[snafu(display("Path {} is not valid UTF-8", path.display()))]
    PathUtf8 { path: PathBuf, backtrace: Backtrace },

    // the source error is zero-sized with a fixed message, no sense in displaying it
    #[snafu(display("Path {} is not within {}", path.display(), base.display()))]
    Prefix {
        path: PathBuf,
        base: PathBuf,
        source: std::path::StripPrefixError,
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

    #[snafu(display("Failed to walk directory tree: {}", source))]
    WalkDir {
        source: walkdir::Error,
        backtrace: Backtrace,
    },
}
