use migration_helpers::common_migrations::{ListReplacement, ReplaceListsMigration};
use migration_helpers::{migrate, Result};
use std::process;

/// Handle new configuration files for kubelet configuration.
fn run() -> Result<()> {
    migrate(ReplaceListsMigration(vec![ListReplacement {
        setting: "services.kubernetes.configuration-files",
        old_vals: &[
            "kubelet-env",
            "kubelet-config",
            "kubelet-kubeconfig",
            "kubelet-bootstrap-kubeconfig",
            "kubelet-exec-start-conf",
            "kubernetes-ca-crt",
            "proxy-env",
        ],
        new_vals: &[
            "kubelet-env",
            "kubelet-config",
            "kubelet-kubeconfig",
            "kubelet-bootstrap-kubeconfig",
            "kubelet-exec-start-conf",
            "kubernetes-ca-crt",
            "proxy-env",
            "kubelet-server-crt",
            "kubelet-server-key",
            "credential-provider-config-yaml",
        ],
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
