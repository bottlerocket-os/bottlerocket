//! Provides the list of errors for `logdog`.

use std::io;
use std::path::PathBuf;

use snafu::{Backtrace, Snafu};

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub(crate)")]
pub(crate) enum Error {
    #[snafu(display("Encountered an empty command."))]
    EmptyCommand { backtrace: Backtrace },
    #[snafu(display("Error creating the tarball file '{}': {}", path.display(), source))]
    TarballFileCreate {
        source: io::Error,
        path: PathBuf,
        backtrace: Backtrace,
    },
    #[snafu(display("Error writing to the tarball '{}': {}", path.display(), source))]
    TarballWrite {
        source: io::Error,
        path: PathBuf,
        backtrace: Backtrace,
    },
    #[snafu(display("Error closing the tarball '{}': {}", path.display(), source))]
    TarballClose {
        source: io::Error,
        path: PathBuf,
        backtrace: Backtrace,
    },
    #[snafu(display("The output directory '{}' could not be created: {}.", path.display(), source))]
    CreateOutputDirectory {
        source: io::Error,
        path: PathBuf,
        backtrace: Backtrace,
    },
    #[snafu(display("Output file '{}' can not be written to the directory that is being compressed '{}'.", outfile.display(), indir.display()))]
    TarballOutputIsInInputDir {
        indir: PathBuf,
        outfile: PathBuf,
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
    #[snafu(display("Error creating the command stderr file '{}': {}", path.display(), source))]
    CommandErrFile {
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
    #[snafu(display("Error starting command '{}': {}", command, source))]
    CommandSpawn {
        command: String,
        source: io::Error,
        backtrace: Backtrace,
    },
    #[snafu(display("Error completing command '{}': {}", command, source))]
    CommandFinish {
        command: String,
        source: io::Error,
        backtrace: Backtrace,
    },
    #[snafu(display("Cannot write to / as a file."))]
    RootAsFile { backtrace: Backtrace },
    #[snafu(display("Error creating tempdir: {}", source))]
    TempDirCreate {
        source: io::Error,
        backtrace: Backtrace,
    },
}

pub(crate) type Result<T> = std::result::Result<T, Error>;
