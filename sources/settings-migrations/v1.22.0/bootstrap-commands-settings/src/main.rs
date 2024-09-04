use migration_helpers::common_migrations::AddPrefixesMigration;
use migration_helpers::{migrate, Result};
use std::process;

fn run() -> Result<()> {
    migrate(AddPrefixesMigration(vec![
        "settings.bootstrap-commands",
        "services.bootstrap-commands",
        "configuration-files.bootstrap-commands-toml",
    ]))
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        process::exit(1);
    }
}
