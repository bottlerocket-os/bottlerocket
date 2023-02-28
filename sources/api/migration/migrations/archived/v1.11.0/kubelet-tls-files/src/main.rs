use migration_helpers::common_migrations::AddPrefixesMigration;
use migration_helpers::{migrate, Result};
use std::process;

/// We added support for adding new kubelet TLS certs/keys for communicating with the Kubernetes API server.
fn run() -> Result<()> {
    migrate(AddPrefixesMigration(vec![
        "configuration-files.kubelet-server-crt",
        "configuration-files.kubelet-server-key",
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
