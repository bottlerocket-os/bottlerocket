use migration_helpers::common_migrations::{ListReplacement, ReplaceListsMigration};
use migration_helpers::{migrate, Result};
use std::process;

/// We updated the 'affected-services' list metadata for 'settings.ecs' to remove
/// ecs-settings-applier on upgrade, and to add it on downgrade.
fn run() -> Result<()> {
    migrate(ReplaceListsMigration(vec![ListReplacement {
        setting: "services.ecs.restart-commands",
        old_vals: &[
            "/usr/bin/ecs-settings-applier",
            "/bin/systemctl try-reload-or-restart ecs.service",
        ],
        new_vals: &["/bin/systemctl try-reload-or-restart ecs.service"],
    }]))
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        process::exit(1);
    }
}
