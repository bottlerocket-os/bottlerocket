use migration_helpers::common_migrations::ReplaceStringMigration;
use migration_helpers::{migrate, Result};
use std::process;

const OLD_CONTROL_SOURCE_VAL: &str = "public.ecr.aws/bottlerocket/bottlerocket-control:v0.5.4";
const NEW_CONTROL_SOURCE_VAL: &str = "public.ecr.aws/bottlerocket/bottlerocket-control:v0.5.5";

/// We bumped the version of the default control container from v0.5.4 to v0.5.5
fn run() -> Result<()> {
    migrate(ReplaceStringMigration {
        setting: "settings.host-containers.control.source",
        old_val: OLD_CONTROL_SOURCE_VAL,
        new_val: NEW_CONTROL_SOURCE_VAL,
    })
}

// Returning a Result from main makes it print a Debug representation of the error, but with Snafu
// we have nice Display representations of the error, so we wrap "main" (run) and print any error.
// https://github.com/shepmaster/snafu/issues/110
fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        process::exit(1);
    }
}
