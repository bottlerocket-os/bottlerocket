use migration_helpers::common_migrations::AddPrefixesMigration;
use migration_helpers::{migrate, Result};
use std::process;

// Create the new config file
fn run() -> Result<()> {
    migrate(AddPrefixesMigration(vec![
        "configuration-files.host-containers-toml",
    ]))
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        process::exit(1);
    }
}
