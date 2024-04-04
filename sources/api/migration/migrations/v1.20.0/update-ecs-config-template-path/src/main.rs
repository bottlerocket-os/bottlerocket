use migration_helpers::common_migrations::ReplaceStringMigration;
use migration_helpers::{migrate, Result};
use std::process;

/// We updated the 'template-path' for 'ecs-config'
fn run() -> Result<()> {
    migrate(ReplaceStringMigration {
        setting: "configuration-files.ecs-config.template-path",
        old_val: "/usr/share/templates/ecs.config",
        new_val: "/usr/share/templates/ecs-base-conf",
    })
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        process::exit(1);
    }
}
