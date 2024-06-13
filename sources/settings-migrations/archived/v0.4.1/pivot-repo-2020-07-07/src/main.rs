use migration_helpers::common_migrations::ReplaceTemplateMigration;
use migration_helpers::{migrate, Result};
use std::process;

const BEFORE_PIVOT_REPO_URL: &str =
    "https://updates.bottlerocket.aws/2020-02-02/{{ os.variant_id }}/{{ os.arch }}/";
const AFTER_PIVOT_REPO_URL: &str =
    "https://updates.bottlerocket.aws/2020-07-07/{{ os.variant_id }}/{{ os.arch }}/";

/// Starting with v0.4.1 we use a new set of repos that does not contain
/// unsigned migrations
fn run() -> Result<()> {
    migrate(ReplaceTemplateMigration {
        setting: "settings.updates.metadata-base-url",
        old_template: BEFORE_PIVOT_REPO_URL,
        new_template: AFTER_PIVOT_REPO_URL,
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
