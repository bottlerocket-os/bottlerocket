use migration_helpers::common_migrations::{ListReplacement, ReplaceListsMigration};
use migration_helpers::{migrate, Result};
use std::process;

/// We updated the configuration files and restart commands to support running kubelet in
/// standalone mode, and for configuring it to use TLS auth. They need to be restored to
/// the prior values on downgrade.
fn run() -> Result<()> {
    migrate(ReplaceListsMigration(vec![
        ListReplacement {
            setting: "services.kubernetes.configuration-files",
            old_vals: &[
                "kubelet-env",
                "kubelet-config",
                "kubelet-kubeconfig",
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
            ],
        },
        ListReplacement {
            setting: "services.kubernetes.restart-commands",
            old_vals: &["/bin/systemctl try-restart kubelet.service"],
            new_vals: &[
                "/usr/bin/systemctl daemon-reload",
                "/bin/systemctl try-restart kubelet.service",
            ],
        },
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
