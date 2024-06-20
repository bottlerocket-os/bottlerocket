use migration_helpers::{common_migrations::AddSettingsMigration, migrate, Result};
use std::process;

/// Mode settings were added for a handful of the templated kubelet configuration files.
fn run() -> Result<()> {
    migrate(AddSettingsMigration(&[
        "configuration-files.kubelet-config.mode",
        "configuration-files.kubelet-kubeconfig.mode",
        "configuration-files.kubelet-bootstrap-kubeconfig.mode",
        "configuration-files.kubelet-exec-start-conf.mode",
        "configuration-files.credential-provider-config-yaml.mode",
        "configuration-files.kubernetes-ca-crt.mode",
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
