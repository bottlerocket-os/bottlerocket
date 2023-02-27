use migration_helpers::common_migrations::AddPrefixesMigration;
use migration_helpers::{migrate, Result};
use std::process;

/// We added a new setting for configuring the AWS client configuration. This
/// can be used by any client expecting to find settings in the default
/// `~/.aws/*` location.
fn run() -> Result<()> {
    migrate(AddPrefixesMigration(vec![
        "services.aws",
        "configuration-files.aws-config",
        "configuration-files.aws-credentials",
    ]))
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
