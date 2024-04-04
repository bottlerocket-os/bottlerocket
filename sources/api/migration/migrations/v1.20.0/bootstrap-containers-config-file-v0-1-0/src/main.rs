use migration_helpers::common_migrations::AddPrefixesMigration;
use migration_helpers::{migrate, Result};
use std::process;

fn run() -> Result<()> {
    migrate(AddPrefixesMigration(vec![
        "configuration-files.bootstrap-containers-toml",
    ]))
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        process::exit(1);
    }
}
