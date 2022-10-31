use crate::crds::{BottlerocketInput, MigrationDirection, MigrationInput};
use crate::error::{self, Result};
use aws_sdk_ec2::model::{Filter, Image};
use aws_sdk_ec2::Region;
use bottlerocket_types::agent_config::{ClusterType, Ec2Config, MigrationConfig};
use maplit::btreemap;
use model::{DestructionPolicy, Resource, Test};
use serde::Deserialize;
use snafu::{ensure, OptionExt, ResultExt};
use std::collections::HashMap;
use std::fs::File;

/// Get the AMI for the given `region` from the `ami_input` file.
pub(crate) fn ami(ami_input: &str, region: &str) -> Result<String> {
    let file = File::open(ami_input).context(error::IOSnafu {
        what: "Unable to open amis.json",
    })?;
    // Convert the `ami_input` file to a `HashMap` that maps regions to AMI id.
    let amis: HashMap<String, AmiImage> =
        serde_json::from_reader(file).context(error::SerdeJsonSnafu {
            what: format!("Unable to deserialize '{}'", ami_input),
        })?;
    // Make sure there are some AMIs present in the `ami_input` file.
    ensure!(
        !amis.is_empty(),
        error::InvalidSnafu {
            what: format!("{} is empty", ami_input)
        }
    );
    Ok(amis
        .get(region)
        .context(error::InvalidSnafu {
            what: format!("AMI not found for region '{}'", region),
        })?
        .id
        .clone())
}

/// Queries EC2 for the given AMI name. If found, returns Ok(Some(id)), if not returns Ok(None).
pub(crate) async fn get_ami_id<S1, S2, S3>(name: S1, arch: S2, region: S3) -> Result<String>
where
    S1: Into<String>,
    S2: Into<String>,
    S3: Into<String>,
{
    // Create the `aws_config` that will be used to search EC2 for AMIs.
    // TODO: Follow chain of assumed roles for creating config like pubsys uses.
    let config = aws_config::from_env()
        .region(Region::new(region.into()))
        .load()
        .await;
    let ec2_client = aws_sdk_ec2::Client::new(&config);
    // Find all images named `name` on `arch` in the `region`.
    let describe_images = ec2_client
        .describe_images()
        .owners("self")
        .filters(Filter::builder().name("name").values(name).build())
        .filters(
            Filter::builder()
                .name("image-type")
                .values("machine")
                .build(),
        )
        .filters(Filter::builder().name("architecture").values(arch).build())
        .filters(
            Filter::builder()
                .name("virtualization-type")
                .values("hvm")
                .build(),
        )
        .send()
        .await?
        .images;
    let images: Vec<&Image> = describe_images.iter().flatten().collect();
    // Make sure there is exactly 1 image that matches the parameters.
    if images.len() > 1 {
        return Err(error::Error::Invalid {
            what: "Unable to determine AMI. Multiple images were found".to_string(),
        });
    };
    if let Some(image) = images.last().as_ref() {
        Ok(image
            .image_id()
            .context(error::InvalidSnafu {
                what: "No image id for AMI",
            })?
            .to_string())
    } else {
        Err(error::Error::Invalid {
            what: "Unable to determine AMI. No images were found".to_string(),
        })
    }
}

/// Get the standard Bottlerocket AMI name.
pub(crate) fn ami_name(arch: &str, variant: &str, version: &str, commit_id: &str) -> String {
    format!(
        "bottlerocket-{}-{}-{}-{}",
        variant, arch, version, commit_id
    )
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct AmiImage {
    pub(crate) id: String,
}

/// Create a CRD to launch Bottlerocket instances on an EKS or ECS cluster.
pub(crate) async fn ec2_crd<'a>(
    bottlerocket_input: BottlerocketInput<'a>,
    cluster_type: ClusterType,
    region: &str,
) -> Result<Resource> {
    let cluster_name = bottlerocket_input
        .cluster_crd_name
        .as_ref()
        .expect("A cluster provider is required");

    // Create the labels for this EC2 provider.
    let labels = bottlerocket_input.crd_input.labels(btreemap! {
        "testsys/type".to_string() => "instances".to_string(),
        "testsys/cluster".to_string() => cluster_name.to_string(),
        "testsys/region".to_string() => region.to_string()
    });

    // Find all resources using the same cluster.
    let conflicting_resources = bottlerocket_input
        .crd_input
        .existing_crds(
            &labels,
            &["testsys/cluster", "testsys/type", "testsys/region"],
        )
        .await?;

    let mut ec2_builder = Ec2Config::builder();
    ec2_builder
        .node_ami(bottlerocket_input.image_id)
        .instance_count(2)
        .instance_types::<Vec<String>>(
            bottlerocket_input
                .crd_input
                .config
                .instance_type
                .iter()
                .cloned()
                .collect(),
        )
        .cluster_name_template(cluster_name, "clusterName")
        .region_template(cluster_name, "region")
        .instance_profile_arn_template(cluster_name, "iamInstanceProfileArn")
        .assume_role(bottlerocket_input.crd_input.config.agent_role.clone())
        .cluster_type(cluster_type.clone())
        .depends_on(cluster_name)
        .image(
            bottlerocket_input
                .crd_input
                .images
                .ec2_resource_agent_image
                .as_ref()
                .expect("Missing default image for EC2 resource agent"),
        )
        .set_image_pull_secret(
            bottlerocket_input
                .crd_input
                .images
                .testsys_agent_pull_secret
                .clone(),
        )
        .set_labels(Some(labels))
        .set_conflicts_with(conflicting_resources.into())
        .set_secrets(Some(bottlerocket_input.crd_input.config.secrets.clone()))
        .destruction_policy(DestructionPolicy::OnTestSuccess);

    // Add in the EKS specific configuration.
    if cluster_type == ClusterType::Eks {
        ec2_builder
            .subnet_ids_template(cluster_name, "privateSubnetIds")
            .endpoint_template(cluster_name, "endpoint")
            .certificate_template(cluster_name, "certificate")
            .cluster_dns_ip_template(cluster_name, "clusterDnsIp")
            .security_groups_template(cluster_name, "securityGroups");
    } else {
        // The default VPC doesn't attach private subnets to an ECS cluster, so public subnet ids
        // are used instead.
        ec2_builder
            .subnet_ids_template(cluster_name, "publicSubnetIds")
            // TODO If this is not set, the crd cannot be serialized since it is a `Vec` not
            // `Option<Vec>`.
            .security_groups(Vec::new());
    }

    ec2_builder
        .build(format!(
            "{}-instances-{}",
            cluster_name, bottlerocket_input.test_type
        ))
        .map_err(|e| error::Error::Build {
            what: "EC2 instance provider CRD".to_string(),
            error: e.to_string(),
        })
}

/// Create a CRD for migrating Bottlerocket instances using SSM commands.
pub(crate) fn migration_crd(migration_input: MigrationInput) -> Result<Test> {
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

    // Create the migration CRD.
    MigrationConfig::builder()
        .aws_region_template(cluster_resource_name, "region")
        .instance_ids_template(bottlerocket_resource_name, "ids")
        .migrate_to_version(migration_version)
        .tuf_repo(migration_input.crd_input.tuf_metadata())
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
