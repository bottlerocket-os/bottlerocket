use migration_helpers::common_migrations::{MetadataReplacement, ReplaceMetadataMigration};
use migration_helpers::{migrate, Result};
use std::process;

fn build_metadata_migrations() -> Vec<MetadataReplacement> {
    let mut migrations = vec![];

    // On AWS platforms, we use regional ECR repositories.
    // Elsewhere, we use ecr-public, which is global.
    #[cfg(variant_platform = "aws")]
    {
        migrations.append(&mut vec![
            MetadataReplacement {
                setting: "settings.host-containers.admin.source",
                metadata: "setting-generator",
                old_val: "schnauzer settings.host-containers.admin.source",
                new_val: "schnauzer-v2 render --requires 'aws@v1(helpers=[ecr-prefix])' --template '{{ ecr-prefix settings.aws.region }}/bottlerocket-admin:v0.11.1'",
            },
            MetadataReplacement {
                setting: "settings.host-containers.control.source",
                metadata: "setting-generator",
                old_val: "schnauzer settings.host-containers.control.source",
                new_val: "schnauzer-v2 render --requires 'aws@v1(helpers=[ecr-prefix])' --template '{{ ecr-prefix settings.aws.region }}/bottlerocket-control:v0.7.5'",
            },
            MetadataReplacement {
                setting: "settings.updates.metadata-base-url",
                metadata: "setting-generator",
                old_val: "schnauzer settings.updates.metadata-base-url",
                new_val: "schnauzer-v2 render --requires 'aws@v1' --requires 'updates@v1(helpers=[metadata-prefix, tuf-prefix])' --template '{{ tuf-prefix settings.aws.region }}{{ metadata-prefix settings.aws.region }}/2020-07-07/{{ os.variant_id }}/{{ os.arch }}/'",
            },
            MetadataReplacement {
                setting: "settings.updates.targets-base-url",
                metadata: "setting-generator",
                old_val: "schnauzer settings.updates.targets-base-url",
                new_val: "schnauzer-v2 render --requires 'aws@v1' --requires 'updates@v1(helpers=[tuf-prefix])' --template '{{ tuf-prefix settings.aws.region }}/targets/'",
            },
        ]);
    }
    #[cfg(not(variant_platform = "aws"))]
    {
        migrations.append(&mut vec![
            MetadataReplacement {
                setting: "settings.updates.metadata-base-url",
                metadata: "setting-generator",
                old_val: "schnauzer settings.updates.metadata-base-url",
                new_val: "schnauzer-v2 render --template 'https://updates.bottlerocket.aws/2020-07-07/{{ os.variant_id }}/{{ os.arch }}/'",
            },
        ]);
    }

    #[cfg(variant_family = "aws-k8s")]
    {
        migrations.append(&mut vec![
            MetadataReplacement {
                setting: "settings.kubernetes.pod-infra-container-image",
                metadata: "setting-generator",
                old_val: "schnauzer settings.kubernetes.pod-infra-container-image",
                new_val: "schnauzer-v2 render --requires 'aws@v1' --requires 'kubernetes@v1(helpers=[pause-prefix])' --template '{{ pause-prefix settings.aws.region }}/eks/pause:3.1-eksbuild.1'",
            },
        ]);
    }

    migrations
}

fn run() -> Result<()> {
    migrate(ReplaceMetadataMigration(build_metadata_migrations()))
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
