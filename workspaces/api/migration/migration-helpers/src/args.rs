//! Helpers for parsing arguments common to migrations.

use std::env;
use std::process;

use crate::{MigrationType, Result};

/// Stores user-supplied arguments.
pub struct Args {
    pub datastore_path: String,
    pub migration_type: MigrationType,
}

/// Informs the user about proper usage of the program and exits.
fn usage() -> ! {
    let program_name = env::args().next().unwrap_or_else(|| "program".to_string());
    eprintln!(
        r"Usage: {}
            --datastore-path PATH
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
    let mut datastore_path = None;

    let mut iter = args.skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_ref() {
            "--datastore-path" => {
                datastore_path = Some(
                    iter.next()
                        .unwrap_or_else(|| usage_msg("Did not give argument to --datastore-path")),
                )
            }

            "--forward" => migration_type = Some(MigrationType::Forward),
            "--backward" => migration_type = Some(MigrationType::Backward),

            _ => usage(),
        }
    }

    Ok(Args {
        datastore_path: datastore_path.unwrap_or_else(|| usage()),
        migration_type: migration_type.unwrap_or_else(|| usage()),
    })
}
