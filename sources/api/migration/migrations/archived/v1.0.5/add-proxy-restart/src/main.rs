use migration_helpers::common_migrations::{ListReplacement, ReplaceListsMigration};
use migration_helpers::{migrate, Result};
use std::process;

/// We updated the restart-commands and configuration-files settings for several existing services.
/// We need to replace them upon downgrades and upgrades
fn run() -> Result<()> {
    migrate(ReplaceListsMigration(vec![
        ListReplacement {
            setting: "services.containerd.configuration-files",
            old_vals: &["containerd-config-toml"],
            new_vals: &["containerd-config-toml", "proxy-env"],
        },
        ListReplacement {
            setting: "services.containerd.restart-commands",
            old_vals: &[],
            new_vals: &["/bin/systemctl try-restart containerd.service"],
        },
        ListReplacement {
            setting: "services.kubernetes.configuration-files",
            old_vals: &[
                "kubelet-env",
                "kubelet-config",
                "kubelet-kubeconfig",
                "kubernetes-ca-crt",
            ],
            new_vals: &[
                "kubelet-env",
                "kubelet-config",
                "kubelet-kubeconfig",
                "kubernetes-ca-crt",
                "proxy-env",
            ],
        },
        ListReplacement {
            setting: "services.kubernetes.restart-commands",
            old_vals: &[],
            new_vals: &["/bin/systemctl try-restart kubelet.service"],
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
