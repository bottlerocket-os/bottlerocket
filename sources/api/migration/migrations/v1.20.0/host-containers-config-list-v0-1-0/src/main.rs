use migration_helpers::common_migrations::{ListReplacement, ReplaceListsMigration};
use migration_helpers::{migrate, Result};
use std::process;

// Add new config file to host-containers
fn run() -> Result<()> {
    migrate(ReplaceListsMigration(vec![ListReplacement {
        setting: "services.host-containers.configuration-files",
        old_vals: &["host-ctr-toml"],
        new_vals: &["host-ctr-toml", "host-containers-toml"],
    }]))
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        process::exit(1);
    }
}
