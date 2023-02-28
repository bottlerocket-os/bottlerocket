use migration_helpers::common_migrations::{ListReplacement, ReplaceListsMigration};
use migration_helpers::{migrate, Result};
use std::process;

/// We templatized the configuration file for the Docker daemon.
/// We also added a new configuration file for host-containers and bootstrap-containers
fn run() -> Result<()> {
    migrate(ReplaceListsMigration(vec![
        ListReplacement {
            setting: "services.docker.configuration-files",
            old_vals: &["proxy-env"],
            new_vals: &["docker-daemon-config", "proxy-env"],
        },
        ListReplacement {
            setting: "services.bootstrap-containers.configuration-files",
            old_vals: &[],
            new_vals: &["host-ctr-toml"],
        },
        ListReplacement {
            setting: "services.host-containers.configuration-files",
            old_vals: &[],
            new_vals: &["host-ctr-toml"],
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
