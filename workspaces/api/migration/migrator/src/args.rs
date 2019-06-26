//! This module handles argument parsing for the migrator binary.

use crate::version::Version;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;
use std::str::FromStr;

/// Informs the user about proper usage of the program and exits.
fn usage() -> ! {
    let program_name = env::args().next().unwrap_or_else(|| "program".to_string());
    eprintln!(
        r"Usage: {}
            --datastore-path PATH
            --migrate-to-version x.y
            [ --no-color ]
            [ --verbose --verbose ... ]",
        program_name
    );
    process::exit(2);
}

/// Prints a more specific message before exiting through usage().
fn usage_msg<S: AsRef<str>>(msg: S) -> ! {
    eprintln!("{}\n", msg.as_ref());
    usage();
}

/// Stores user-supplied arguments.
pub(crate) struct Args {
    pub(crate) datastore_path: PathBuf,
    pub(crate) migrate_to_version: Version,
    pub(crate) color: stderrlog::ColorChoice,
    pub(crate) verbosity: usize,
}

impl Args {
    /// Parses user arguments into an Args structure.
    pub(crate) fn from_env(args: env::Args) -> Self {
        // Required parameters.
        let mut datastore_path = None;
        let mut migrate_to_version = None;
        // Optional parameters with their defaults.
        let mut verbosity = 2; // default to INFO level
        let mut color = stderrlog::ColorChoice::Auto;

        let mut iter = args.skip(1);
        while let Some(arg) = iter.next() {
            match arg.as_ref() {
                "--datastore-path" => {
                    let path_str = iter
                        .next()
                        .unwrap_or_else(|| usage_msg("Did not give argument to --datastore-path"));
                    trace!("Given --datastore-path: {}", path_str);
                    let canonical = fs::canonicalize(path_str).unwrap_or_else(|e| {
                        usage_msg(format!(
                            "Could not canonicalize given data store path: {}",
                            e
                        ))
                    });
                    trace!("Canonicalized data store path: {}", canonical.display());
                    datastore_path = Some(canonical);
                }

                "--migrate-to-version" => {
                    let version_str = iter.next().unwrap_or_else(|| {
                        usage_msg("Did not give argument to --migrate-to-version")
                    });
                    trace!("Given --migrate-to-version: {}", version_str);
                    let version = Version::from_str(&version_str).unwrap_or_else(|e| {
                        usage_msg(format!("Invalid argument to --migrate-to-version: {}", e))
                    });
                    migrate_to_version = Some(version)
                }

                "-v" | "--verbose" => verbosity += 1,

                "--no-color" => color = stderrlog::ColorChoice::Never,

                _ => usage(),
            }
        }

        Self {
            datastore_path: datastore_path.unwrap_or_else(|| usage()),
            migrate_to_version: migrate_to_version.unwrap_or_else(|| usage()),
            color,
            verbosity,
        }
    }
}
