use migration_helpers::common_migrations::AddPrefixesMigration;
use migration_helpers::{migrate, Result};
use std::process;

/// We added new settings for running kubelet in standalone mode, and for using TLS auth.
/// We also added new configuration files to apply these settings. They need to be removed
/// when we downgrade.
fn run() -> Result<()> {
    migrate(AddPrefixesMigration(vec![
        "settings.kubernetes.bootstrap-token",
        "settings.kubernetes.authentication-mode",
        "settings.kubernetes.standalone-mode",
        "configuration-files.kubelet-bootstrap-kubeconfig",
        "configuration-files.kubelet-exec-start-conf",
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
