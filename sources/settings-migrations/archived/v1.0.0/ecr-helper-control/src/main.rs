use migration_helpers::common_migrations::ReplaceTemplateMigration;
use migration_helpers::{migrate, Result};
use std::process;

const OLD_CONTROL_CTR_TEMPLATE: &str =
    "328549459982.dkr.ecr.{{ settings.aws.region }}.amazonaws.com/bottlerocket-control:v0.4.1";
const NEW_CONTROL_CTR_TEMPLATE: &str =
    "{{ ecr-prefix settings.aws.region }}/bottlerocket-control:v0.4.1";

/// We added a helper to lookup an ECR registry number by region.
fn run() -> Result<()> {
    migrate(ReplaceTemplateMigration {
        setting: "metadata.settings.host-containers.control.source",
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
