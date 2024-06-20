use migration_helpers::common_migrations::{AddPrefixesMigration, AddSettingsMigration};
use migration_helpers::{migrate, Result};
use std::process;

/// We added new settings for AWS credential configuration.
fn run() -> Result<()> {
    if cfg!(variant_platform = "aws") {
        migrate(AddSettingsMigration(&[
            "settings.aws.config",
            "settings.aws.credentials",
            "settings.aws.profile",
        ]))
    } else {
        // Non-AWS variants did not have any AWS setting until this point,
        // so need to completely clean up on downgrade.
        migrate(AddPrefixesMigration(vec!["settings.aws"]))
    }
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
