use migration_helpers::common_migrations::{NoOpMigration, ReplaceStringMigration};
use migration_helpers::{migrate, Result};
use std::process;

/// We updated the 'path' string for 'Containerd configuration template'.
fn run() -> Result<()> {
    if cfg!(variant_runtime = "ecs") {
        migrate(ReplaceStringMigration {
            setting: "configuration-files.containerd-config-toml.path",
            old_val: "/usr/share/templates/containerd-config-toml_basic",
            new_val: "/usr/share/templates/containerd-config",
        })
    } else if cfg!(variant_runtime = "k8s") {
        if cfg!(variant_flavor = "nvidia") {
            migrate(ReplaceStringMigration {
                setting: "configuration-files.containerd-config-toml.template-path",
                old_val: "/usr/share/templates/containerd-config-toml_k8s_nvidia_containerd_sock",
                new_val: "/usr/share/templates/containerd-config",
            })
        } else {
            migrate(ReplaceStringMigration {
                setting: "configuration-files.containerd-config-toml.template-path",
                old_val: "/usr/share/templates/containerd-config-toml_k8s_containerd_sock",
                new_val: "/usr/share/templates/containerd-config",
            })
        }
    } else {
        migrate(NoOpMigration)
    }
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        process::exit(1);
    }
}
