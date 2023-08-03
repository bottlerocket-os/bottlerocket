use migration_helpers::common_migrations::{MetadataReplacement, ReplaceMetadataMigration};
use migration_helpers::{migrate, Result};
use std::process;

//
fn run() -> Result<()> {
    migrate(ReplaceMetadataMigration(vec![
        MetadataReplacement {
            setting: "settings.host-containers.admin.source",
            metadata: "setting-generator",
            old_val: "schnauzer settings.host-containers.admin.source",
            new_val: "schnauzer-v2 render --requires 'aws@v1(ecr-prefix)' --template '{{ ecr-prefix settings.aws.region }}/bottlerocket-admin:v0.10.1'",
        },
        MetadataReplacement {
            setting: "settings.host-containers.control.source",
            metadata: "setting-generator",
            old_val: "schnauzer settings.host-containers.control.source",
            new_val: "schnauzer-v2 render --requires 'aws@v1(ecr-prefix)' --template '{{ ecr-prefix settings.aws.region }}/bottlerocket-control:v0.7.2'",
        },
        MetadataReplacement {
            setting: "settings.updates.targets-base-url",
            metadata: "setting-generator",
            old_val: "schnauzer settings.updates.targets-base-url",
            new_val: "schnauzer-v2 render --requires 'aws@v1' --requires 'updates@v1(tuf-prefix)' --template '{{ tuf-prefix settings.aws.region }}/targets/'",
        },
        MetadataReplacement {
            setting: "settings.updates.metadata-base-url",
            metadata: "setting-generator",
            old_val: "schnauzer settings.updates.metadata-base-url",
            new_val: "schnauzer-v2 render --requires 'aws@v1' --requires 'updates@v1(metadata-prefix, tuf-prefix)' --template '{{ tuf-prefix settings.aws.region }}{{ metadata-prefix settings.aws.region }}/2020-07-07/{{ os.variant_id }}/{{ os.arch }}/'",
        },
    ]))
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
