use crate::crds::BottlerocketInput;
use crate::error::{self, Result};
use aws_sdk_ec2::config::Region;
use aws_sdk_ec2::types::{Filter, Image};
use bottlerocket_types::agent_config::{
    ClusterType, CustomUserData, Ec2Config, Ec2KarpenterConfig, KarpenterDeviceMapping,
};
use maplit::btreemap;
use serde::Deserialize;
use snafu::{ensure, OptionExt, ResultExt};
use std::collections::HashMap;
use std::fs::File;
use std::iter::repeat_with;
use testsys_model::{DestructionPolicy, Resource};

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
pub(crate) async fn get_ami_id<S1, S2, S3>(
    name: S1,
    arch: S2,
    region: S3,
    account: Option<&str>,
) -> Result<String>
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
        .owners(account.unwrap_or("self"))
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
    if !bottlerocket_input
        .crd_input
        .config
        .block_device_mapping
        .is_empty()
    {
        return Err(error::Error::Invalid {
            what: "Custom block mappings are not supported for ec2 instance launch".to_string(),
        });
    }

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
        .custom_user_data(
            bottlerocket_input
                .crd_input
                .encoded_userdata()?
                .map(|encoded_userdata| CustomUserData::Merge { encoded_userdata }),
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
        .destruction_policy(
            bottlerocket_input
                .crd_input
                .config
                .dev
                .bottlerocket_destruction_policy
                .to_owned()
                .unwrap_or(DestructionPolicy::OnTestSuccess),
        );

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

    let suffix: String = repeat_with(fastrand::lowercase).take(4).collect();
    ec2_builder
        .build(format!("{}-instances-{}", cluster_name, suffix))
        .context(error::BuildSnafu {
            what: "EC2 instance provider CRD",
        })
}

/// Create a CRD to launch Bottlerocket instances on an EKS or ECS cluster.
pub(crate) async fn ec2_karpenter_crd<'a>(
    bottlerocket_input: BottlerocketInput<'a>,
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

    // If no mappings were provided use a standard mapping as a default
    let device_mappings = if bottlerocket_input
        .crd_input
        .config
        .block_device_mapping
        .is_empty()
    {
        vec![
            KarpenterDeviceMapping {
                name: "/dev/xvda".to_string(),
                volume_type: "gp3".to_string(),
                volume_size: 4,
                delete_on_termination: true,
            },
            KarpenterDeviceMapping {
                name: "/dev/xvdb".to_string(),
                volume_type: "gp3".to_string(),
                volume_size: 20,
                delete_on_termination: true,
            },
        ]
    } else {
        bottlerocket_input
            .crd_input
            .config
            .block_device_mapping
            .clone()
    };

    let mut ec2_builder = Ec2KarpenterConfig::builder();
    ec2_builder
        .node_ami(bottlerocket_input.image_id)
        .instance_types::<Vec<String>>(
            bottlerocket_input
                .crd_input
                .config
                .instance_type
                .iter()
                .cloned()
                .collect(),
        )
        .custom_user_data(
            bottlerocket_input
                .crd_input
                .encoded_userdata()?
                .map(|encoded_userdata| CustomUserData::Merge { encoded_userdata }),
        )
        .cluster_name_template(cluster_name, "clusterName")
        .region_template(cluster_name, "region")
        .subnet_ids_template(cluster_name, "privateSubnetIds")
        .endpoint_template(cluster_name, "endpoint")
        .cluster_sg_template(cluster_name, "clustersharedSg")
        .device_mappings(device_mappings)
        .assume_role(bottlerocket_input.crd_input.config.agent_role.clone())
        .depends_on(cluster_name)
        .image(
            bottlerocket_input
                .crd_input
                .images
                .ec2_karpenter_resource_agent_image
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
        .destruction_policy(
            bottlerocket_input
                .crd_input
                .config
                .dev
                .bottlerocket_destruction_policy
                .to_owned()
                .unwrap_or(DestructionPolicy::OnTestSuccess),
        );

    let suffix: String = repeat_with(fastrand::lowercase).take(4).collect();
    ec2_builder
        .build(format!("{}-karpenter-{}", cluster_name, suffix))
        .context(error::BuildSnafu {
            what: "EC2 instance provider CRD",
        })
}
