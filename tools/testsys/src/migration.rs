use crate::crds::{MigrationDirection, MigrationInput};
use crate::error::{self, Result};
use bottlerocket_types::agent_config::MigrationConfig;
use maplit::btreemap;
use model::Test;
use snafu::OptionExt;

/// Create a CRD for migrating Bottlerocket instances using SSM commands.
pub(crate) fn migration_crd(
    migration_input: MigrationInput,
    aws_region_override: Option<String>,
) -> Result<Test> {
    let cluster_resource_name = migration_input
        .cluster_crd_name
        .as_ref()
        .expect("A cluster name is required for migrations");
    let bottlerocket_resource_name = migration_input
        .bottlerocket_crd_name
        .as_ref()
        .expect("A cluster name is required for migrations");

    let labels = migration_input.crd_input.labels(btreemap! {
        "testsys/type".to_string() => "migration".to_string(),
        "testsys/cluster".to_string() => cluster_resource_name.to_string(),
    });

    // Determine which version should be migrated to from `migration_input`.
    let migration_version = match migration_input.migration_direction {
        MigrationDirection::Upgrade => migration_input
            .crd_input
            .migrate_to_version
            .as_ref()
            .context(error::InvalidSnafu {
                what: "The target migration version is required",
            }),
        MigrationDirection::Downgrade => migration_input
            .crd_input
            .starting_version
            .as_ref()
            .context(error::InvalidSnafu {
                what: "The starting migration version is required",
            }),
    }?;

    // Construct the migration CRD.
    let mut migration_config = MigrationConfig::builder();

    // Use the specified aws-region for the migration test.
    // If no region is specified, derive the appropriate region based on the region of the
    // cluster resource CRD (assuming it's an ECS or EKS cluster).
    if let Some(aws_region) = aws_region_override {
        migration_config.aws_region(aws_region)
    } else {
        migration_config.aws_region_template(cluster_resource_name, "region")
    };

    migration_config
        .instance_ids_template(bottlerocket_resource_name, "ids")
        .migrate_to_version(migration_version)
        .tuf_repo(migration_input.crd_input.tuf_repo_config())
        .assume_role(migration_input.crd_input.config.agent_role.clone())
        .resources(bottlerocket_resource_name)
        .resources(cluster_resource_name)
        .set_depends_on(Some(migration_input.prev_tests))
        .image(
            migration_input
                .crd_input
                .images
                .migration_test_agent_image
                .as_ref()
                .expect("Missing default image for migration test agent"),
        )
        .set_image_pull_secret(
            migration_input
                .crd_input
                .images
                .testsys_agent_pull_secret
                .to_owned(),
        )
        .keep_running(true)
        .set_secrets(Some(migration_input.crd_input.config.secrets.to_owned()))
        .set_labels(Some(labels))
        .build(format!(
            "{}{}",
            cluster_resource_name,
            migration_input.name_suffix.unwrap_or_default()
        ))
        .map_err(|e| error::Error::Build {
            what: "migration CRD".to_string(),
            error: e.to_string(),
        })
}
