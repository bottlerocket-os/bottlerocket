#![allow(deprecated)]
use migration_helpers::common_migrations::ReplaceTemplateMigration;
use migration_helpers::{migrate, Result};
use std::process;

const OLD_CONTROL_CTR_TEMPLATE: &str =
    "{{ ecr-prefix settings.aws.region }}/bottlerocket-control:v0.7.3";
const NEW_CONTROL_CTR_TEMPLATE: &str =
    "{{ ecr-prefix settings.aws.region }}/bottlerocket-control:v0.7.4";

/// We bumped the version of the default control container
fn run() -> Result<()> {
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
