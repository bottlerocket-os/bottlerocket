use migration_helpers::common_migrations::{ListRestriction, RestrictListsMigration};
use migration_helpers::{migrate, Result};
use std::process;

// Beta options for cpu-manager-policy-options became available without feature gates:
// - strict-cpu-reservation: 1.32 or higher
// - distribute-cpus-across-numa: 1.33 or higher
// - prefer-align-cpus-by-uncorecache: 1.34 or higher
//
// On downgrade, we remove these newer options to prevent kubelet from receiving
// incompatible configuration values. We keep full-pcpus-only as it's stable.
fn run() -> Result<()> {
    migrate(RestrictListsMigration(vec![ListRestriction {
        setting: "settings.kubernetes.cpu-manager-policy-options",
        allowed_vals: &["full-pcpus-only"],
    }]))
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        process::exit(1);
    }
}
