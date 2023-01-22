//! This module handles argument parsing for the migrator binary.

use bottlerocket_release::BottlerocketRelease;
use semver::Version;
use simplelog::LevelFilter;
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
            --migration-directory PATH
            --root-path PATH
            --metadata-directory PATH
            (--migrate-to-version x.y | --migrate-to-version-from-os-release)
            [ --no-color ]
            [ --log-level trace|debug|info|warn|error ]",
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
    pub(crate) log_level: LevelFilter,
    pub(crate) migration_directory: PathBuf,
    pub(crate) migrate_to_version: Version,
    pub(crate) root_path: PathBuf,
    pub(crate) metadata_directory: PathBuf,
}

impl Args {
    /// Parses user arguments into an Args structure.
    pub(crate) fn from_env(args: env::Args) -> Self {
        // Required parameters.
        let mut datastore_path = None;
        let mut log_level = None;
        let mut migration_directory = None;
        let mut migrate_to_version = None;
        let mut root_path = None;
        let mut metadata_path = None;

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

                "--log-level" => {
                    let log_level_str = iter
                        .next()
                        .unwrap_or_else(|| usage_msg("Did not give argument to --log-level"));
                    log_level = Some(LevelFilter::from_str(&log_level_str).unwrap_or_else(|_| {
                        usage_msg(format!("Invalid log level '{}'", log_level_str))
                    }));
                }

                "--migration-directory" => {
                    let path_str = iter.next().unwrap_or_else(|| {
                        usage_msg("Did not give argument to --migration-directory")
                    });
                    trace!("Given --migration-directory: {}", path_str);
                    migration_directory = Some(PathBuf::from(path_str));
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

                "--migrate-to-version-from-os-release" => {
                    let br = BottlerocketRelease::new().unwrap_or_else(|e| {
                        usage_msg(format!("Unable to get version from os-release: {}", e))
                    });
                    migrate_to_version = Some(br.version_id)
                }

                "--root-path" => {
                    let path_str = iter
                        .next()
                        .unwrap_or_else(|| usage_msg("Did not give argument to --root-path"));
                    trace!("Given --root-path: {}", path_str);
                    root_path = Some(PathBuf::from(path_str));
                }

                "--metadata-directory" => {
                    let path_str = iter.next().unwrap_or_else(|| {
                        usage_msg("Did not give argument to --metadata-directory")
                    });
                    trace!("Given --metadata-directory: {}", path_str);
                    metadata_path = Some(PathBuf::from(path_str));
                }
                _ => usage_msg(format!("Unable to parse input '{}'", arg)),
            }
        }

        Self {
            datastore_path: datastore_path
                .unwrap_or_else(|| usage_msg("--datastore-path must be specified")),
            log_level: log_level.unwrap_or(LevelFilter::Info),
            migration_directory: migration_directory
                .unwrap_or_else(|| usage_msg("--migration-directory must be specified")),
            migrate_to_version: migrate_to_version.unwrap_or_else(|| {
                usage_msg(
                    "Desired version could not be determined; pass --migrate-to-version or \
                    --migrate-to-version-from-os-release",
                )
            }),
            root_path: root_path.unwrap_or_else(|| usage_msg("--root-path must be specified")),
            metadata_directory: metadata_path
                .unwrap_or_else(|| usage_msg("--metadata-directory must be specified")),
        }
    }
}
