use migration_helpers::common_migrations::ReplaceStringMigration;
use migration_helpers::{migrate, Result};
use std::process;

/// We updated the 'path' string for 'ecs-config'
fn run() -> Result<()> {
    migrate(ReplaceStringMigration {
        setting: "configuration-files.ecs-config.path",
        old_val: "/etc/ecs/ecs.config",
        new_val: "/etc/systemd/system/ecs.service.d/10-base.conf",
    })
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        process::exit(1);
    }
}
