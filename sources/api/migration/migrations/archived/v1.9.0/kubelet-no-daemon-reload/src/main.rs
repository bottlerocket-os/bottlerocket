use migration_helpers::common_migrations::{ListReplacement, ReplaceListsMigration};
use migration_helpers::{migrate, Result};
use std::process;

/// We updated the restart commands for kubelet to avoid an unnecessary reload
/// of systemd. They need to be restored to the prior values on downgrade.
fn run() -> Result<()> {
    migrate(ReplaceListsMigration(vec![ListReplacement {
        setting: "services.kubernetes.restart-commands",
        old_vals: &[
            "/usr/bin/systemctl daemon-reload",
            "/usr/bin/systemctl try-restart kubelet.service",
        ],
        new_vals: &["/usr/bin/systemctl try-restart kubelet.service"],
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
