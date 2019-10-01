//! This module handles argument parsing for the migrator binary.

use data_store_version::Version;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::str::FromStr;

/// Informs the user about proper usage of the program and exits.
fn usage() -> ! {
    let program_name = env::args().next().unwrap_or_else(|| "program".to_string());
    eprintln!(
        r"Usage: {}
            --datastore-path PATH
            --migration-directories PATH[:PATH:PATH...]
            (--migrate-to-version x.y | --migrate-to-version-from-file PATH)
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
    pub(crate) migration_directories: Vec<PathBuf>,
    pub(crate) migrate_to_version: Version,
    pub(crate) verbosity: usize,
}

impl Args {
    /// Parses user arguments into an Args structure.
    pub(crate) fn from_env(args: env::Args) -> Self {
        // Required parameters.
        let mut datastore_path = None;
        let mut migration_directories = None;
        let mut migrate_to_version = None;
        // Optional parameters with their defaults.
        let mut verbosity = 2; // default to INFO level

        let mut iter = args.skip(1);
        while let Some(arg) = iter.next() {
            match arg.as_ref() {
                "--datastore-path" => {
                    let path_str = iter
                        .next()
                        .unwrap_or_else(|| usage_msg("Did not give argument to --datastore-path"));
                    trace!("Given --datastore-path: {}", path_str);

                    // On first boot, the data store won't exist yet, because storewolf runs after.
                    if !Path::new(&path_str).exists() {
                        eprintln!(
                            "Data store does not exist at given path, exiting ({})",
                            path_str
                        );
                        process::exit(0);
                    }

                    let canonical = fs::canonicalize(path_str).unwrap_or_else(|e| {
                        usage_msg(format!(
                            "Could not canonicalize given data store path: {}",
                            e
                        ))
                    });
                    trace!("Canonicalized data store path: {}", canonical.display());
                    datastore_path = Some(canonical);
                }

                "--migration-directories" => {
                    let paths_str = iter.next().unwrap_or_else(|| {
                        usage_msg("Did not give argument to --migration-directories")
                    });
                    trace!("Given --migration-directories: {}", paths_str);
                    let paths: Vec<_> = paths_str.split(':').map(PathBuf::from).collect();
                    if paths.is_empty() {
                        usage_msg("Found no paths in argument to --migration-directories");
                    }
                    migration_directories = Some(paths);
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

                "--migrate-to-version-from-file" => {
                    let path_str = iter.next().unwrap_or_else(|| {
                        usage_msg("Did not give argument to --migrate-to-version-from-file")
                    });
                    trace!("Given --migrate-to-version-from-file: {}", path_str);

                    let version_str = fs::read_to_string(&path_str).unwrap_or_else(|e| {
                        usage_msg(format!("Could not read version from {}: {}", path_str, e))
                    });
                    trace!("Read version string from file: {}", version_str);

                    let version = Version::from_str(&version_str).unwrap_or_else(|e| {
                        usage_msg(format!("Invalid version in {}: {}", path_str, e))
                    });
                    migrate_to_version = Some(version)
                }

                "-v" | "--verbose" => verbosity += 1,

                _ => usage(),
            }
        }

        Self {
            datastore_path: datastore_path.unwrap_or_else(|| usage()),
            migration_directories: migration_directories.unwrap_or_else(|| usage()),
            migrate_to_version: migrate_to_version.unwrap_or_else(|| usage()),
            verbosity,
        }
    }
}
