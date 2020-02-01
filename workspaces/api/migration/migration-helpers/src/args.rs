//! Helpers for parsing arguments common to migrations.

use std::env;
use std::process;

use crate::{MigrationType, Result};

/// Stores user-supplied arguments.
pub struct Args {
    pub source_datastore: String,
    pub target_datastore: String,
    pub migration_type: MigrationType,
}

/// Informs the user about proper usage of the program and exits.
fn usage() -> ! {
    let program_name = env::args().next().unwrap_or_else(|| "program".to_string());
    eprintln!(
        r"Usage: {}
            --source-datastore PATH
            --target-datastore PATH
            ( --forward | --backward )",
        program_name
    );
    process::exit(2);
}

/// Prints a more specific message before exiting through usage().
fn usage_msg<S: AsRef<str>>(msg: S) -> ! {
    eprintln!("{}\n", msg.as_ref());
    usage();
}

/// Parses user arguments into an Args structure.
pub(crate) fn parse_args(args: env::Args) -> Result<Args> {
    let mut migration_type = None;
    let mut source_datastore = None;
    let mut target_datastore = None;
    let mut datastore_path = None;

    let mut iter = args.skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_ref() {
            "--source-datastore" => {
                source_datastore =
                    Some(iter.next().unwrap_or_else(|| {
                        usage_msg("Did not give argument to --source-datastore")
                    }))
            }

            "--target-datastore" => {
                target_datastore =
                    Some(iter.next().unwrap_or_else(|| {
                        usage_msg("Did not give argument to --target-datastore")
                    }))
            }

            // Support the argument of the old migrator interface, with some caveats
            "--datastore-path" => {
                datastore_path = Some(
                    iter.next()
                        .unwrap_or_else(|| usage_msg("Did not give argument to --datastore-path")),
                );
            }

            "--forward" => migration_type = Some(MigrationType::Forward),
            "--backward" => migration_type = Some(MigrationType::Backward),

            _ => usage(),
        }
    }

    if let Some(datastore_path) = datastore_path {
        // For compatibility with old migration interface that had single source+target; other code
        // in migration-helpers checks if source==target to see if it needs to do a workaround.
        if source_datastore.is_some() || target_datastore.is_some() {
            usage_msg("--datastore-path is only for backward compatibility and \
                cannot be used with --source-datastore / --target-datastore");
        }
        source_datastore = Some(datastore_path.clone());
        target_datastore = Some(datastore_path);
    } else {
        // In no other case should they be the same; we use it for compatibility checks.
        if source_datastore == target_datastore {
            usage_msg("--source-datastore and --target-datastore cannot be the same");
        }
    }

    Ok(Args {
        source_datastore: source_datastore.unwrap_or_else(|| usage()),
        target_datastore: target_datastore.unwrap_or_else(|| usage()),
        migration_type: migration_type.unwrap_or_else(|| usage()),
    })
}
