use migration_helpers::common_migrations::AddSettingsMigration;
use migration_helpers::{migrate, Result};
use std::process;

/// We added new `settings.ntp` options for configuring chrony:
/// - `refclocks` to point chrony at hardware or software reference clocks, such as the PTP
///   hardware clock exposed by the Amazon ENA driver
/// - `allow`, `cmdallow`, and `bindcmdaddress` to grant remote access to the chrony daemon
fn run() -> Result<()> {
    migrate(AddSettingsMigration(&[
        "settings.ntp.refclocks",
        "settings.ntp.allow",
        "settings.ntp.cmdallow",
        "settings.ntp.bindcmdaddress",
    ]))
}

// Returning a Result from main makes it print a Debug representation of the error, but with Snafu
// we have nice Display representations of the error, so we wrap "main" (run) and print any error.
// https://github.com/shepmaster/snafu/issues/110
fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        process::exit(1);
    }
}
