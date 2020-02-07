#![deny(rust_2018_idioms)]

use migration_helpers::{migrate, Result};
use migration_helpers::common_migrations::ReplaceStringMigration;
use std::process;

const SETTING: &str = "configuration-files.containerd-config-toml.template-path";
// Old version with no variant
const DEFAULT_CTRD_CONFIG_OLD: &str = "/usr/share/templates/containerd-config-toml";
// Any users coming from old versions would be using the aws-k8s variant because no other existed :)
const DEFAULT_CTRD_CONFIG_NEW: &str = "/usr/share/templates/containerd-config-toml_aws-k8s";

/// We changed the path to our containerd configuration template so that we could support image
/// variants with different configs.  We need to update old images to the new path, and on
/// downgrade, new images to the old path.
fn run() -> Result<()> {
    migrate(ReplaceStringMigration {
        setting: SETTING,
        old_val: DEFAULT_CTRD_CONFIG_OLD,
        new_val: DEFAULT_CTRD_CONFIG_NEW
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
