use migration_helpers::common_migrations::{ListReplacement, ReplaceListsMigration};
use migration_helpers::{migrate, Result};
use std::process;

fn run() -> Result<()> {
    migrate(ReplaceListsMigration(vec![ListReplacement {
        setting: "services.bootconfig.configuration-files",
        old_vals: &[],
        new_vals: &["prairiedog-toml"],
    }]))
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        process::exit(1);
    }
}
