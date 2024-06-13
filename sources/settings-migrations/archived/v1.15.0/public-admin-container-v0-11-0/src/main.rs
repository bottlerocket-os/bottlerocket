use migration_helpers::common_migrations::ReplaceStringMigration;
use migration_helpers::{migrate, Result};
use std::process;

const OLD_ADMIN_CTR_SOURCE_VAL: &str = "public.ecr.aws/bottlerocket/bottlerocket-admin:v0.10.2";
const NEW_ADMIN_CTR_SOURCE_VAL: &str = "public.ecr.aws/bottlerocket/bottlerocket-admin:v0.11.0";

/// We bumped the version of the default admin container
fn run() -> Result<()> {
    migrate(ReplaceStringMigration {
        setting: "settings.host-containers.admin.source",
        old_val: OLD_ADMIN_CTR_SOURCE_VAL,
        new_val: NEW_ADMIN_CTR_SOURCE_VAL,
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
