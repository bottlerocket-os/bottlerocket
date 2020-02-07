#![deny(rust_2018_idioms)]

use migration_helpers::{migrate, Result};
use migration_helpers::common_migrations::ReplaceStringMigration;
use std::process;

const DEFAULT_ADMIN_CTR_IMG_OLD: &str =
    "328549459982.dkr.ecr.us-west-2.amazonaws.com/thar-admin:v0.1";
const DEFAULT_ADMIN_CTR_IMG_NEW: &str =
    "328549459982.dkr.ecr.us-west-2.amazonaws.com/thar-admin:v0.2";
const DEFAULT_CONTROL_CTR_IMG_OLD: &str =
    "328549459982.dkr.ecr.us-west-2.amazonaws.com/thar-control:v0.1";
const DEFAULT_CONTROL_CTR_IMG_NEW: &str =
    "328549459982.dkr.ecr.us-west-2.amazonaws.com/thar-control:v0.2";

/// We bumped the versions of the default admin container and the default control container from v0.1 to v0.2
fn run() -> Result<()> {
    migrate(ReplaceStringMigration {
        setting: "settings.host-containers.admin.source",
        old_val: DEFAULT_ADMIN_CTR_IMG_OLD,
        new_val: DEFAULT_ADMIN_CTR_IMG_NEW,
    })?;
    migrate(ReplaceStringMigration {
        setting: "settings.host-containers.control.source",
        old_val: DEFAULT_CONTROL_CTR_IMG_OLD,
        new_val: DEFAULT_CONTROL_CTR_IMG_NEW,
    })
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
