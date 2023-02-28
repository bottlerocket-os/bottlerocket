use migration_helpers::common_migrations::{ListReplacement, ReplaceListsMigration};
use migration_helpers::{migrate, Result};
use std::process;

/// We changed corndog to use subcommands so it can handle different kernel settings without having
/// to apply them all every time.
fn run() -> Result<()> {
    migrate(ReplaceListsMigration(vec![ListReplacement {
        setting: "services.sysctl.restart-commands",
        old_vals: &["/usr/bin/corndog"],
        new_vals: &["/usr/bin/corndog sysctl"],
    }]))
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
