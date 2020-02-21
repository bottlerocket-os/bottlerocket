#![deny(rust_2018_idioms)]

use migration_helpers::common_migrations::ReplaceTemplateMigration;
use migration_helpers::{migrate, Result};
use std::process;

const OLD_ADMIN_CTR_TEMPLATE: &str =
    "328549459982.dkr.ecr.{{ settings.aws.region }}.amazonaws.com/thar-admin:v0.2";
const NEW_ADMIN_CTR_TEMPLATE: &str =
    "328549459982.dkr.ecr.{{ settings.aws.region }}.amazonaws.com/bottlerocket-admin:v0.3";
const OLD_CONTROL_CTR_TEMPLATE: &str =
    "328549459982.dkr.ecr.{{ settings.aws.region }}.amazonaws.com/thar-control:v0.2";
const NEW_CONTROL_CTR_TEMPLATE: &str =
    "328549459982.dkr.ecr.{{ settings.aws.region }}.amazonaws.com/bottlerocket-control:v0.3";

/// We bumped the versions of the default admin container and the default control container from v0.2 to v0.3
/// This migration also includes a name change for the host-container images
fn run() -> Result<()> {
    migrate(ReplaceTemplateMigration {
        setting: "settings.host-containers.admin.source",
        old_template: OLD_ADMIN_CTR_TEMPLATE,
        new_template: NEW_ADMIN_CTR_TEMPLATE,
    })?;
    migrate(ReplaceTemplateMigration {
        setting: "settings.host-containers.control.source",
        old_template: OLD_CONTROL_CTR_TEMPLATE,
        new_template: NEW_CONTROL_CTR_TEMPLATE,
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
