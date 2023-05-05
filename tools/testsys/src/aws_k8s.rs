use crate::aws_resources::{ami, ami_name, ec2_crd, ec2_karpenter_crd, get_ami_id};
use crate::crds::{
    BottlerocketInput, ClusterInput, CrdCreator, CrdInput, CreateCrdOutput, MigrationInput,
    TestInput,
};
use crate::error::{self, Result};
use crate::migration::migration_crd;
use crate::sonobuoy::{sonobuoy_crd, workload_crd};
use bottlerocket_types::agent_config::{
    ClusterType, CreationPolicy, EksClusterConfig, EksctlConfig, K8sVersion,
};
use maplit::btreemap;
use serde_yaml::Value;
use snafu::{OptionExt, ResultExt};
use std::collections::BTreeMap;
use std::str::FromStr;
use testsys_config::ResourceAgentType;
use testsys_model::{Crd, DestructionPolicy};

/// A `CrdCreator` responsible for creating crd related to `aws-k8s` variants.
pub(crate) struct AwsK8sCreator {
    pub(crate) region: String,
    pub(crate) ami_input: String,
    pub(crate) migrate_starting_commit: Option<String>,
}

#[async_trait::async_trait]
impl CrdCreator for AwsK8sCreator {
    /// Determine the AMI from `amis.json`.
    async fn image_id(&self, _: &CrdInput) -> Result<String> {
        ami(&self.ami_input, &self.region)
    }

    /// Determine the starting image from EC2 using standard Bottlerocket naming conventions.
    async fn starting_image_id(&self, crd_input: &CrdInput) -> Result<String> {
        get_ami_id(ami_name(&crd_input.arch,&crd_input.variant,crd_input.starting_version
            .as_ref()
            .context(error::InvalidSnafu{
                what: "The starting version must be provided for migration testing"
            })?, self.migrate_starting_commit
            .as_ref()
            .context(error::InvalidSnafu{
                what: "The commit for the starting version must be provided if the starting image id is not"
            })?)
           , &crd_input.arch,
           & self.region,
           crd_input.config.dev.image_account_id.as_deref(),
        )
        .await
    }

    /// Create an EKS cluster CRD with the `cluster_name` in `cluster_input`.
    async fn cluster_crd<'a>(&self, cluster_input: ClusterInput<'a>) -> Result<CreateCrdOutput> {
        let cluster_version =
            K8sVersion::from_str(cluster_input.crd_input.variant.version().context(
                error::MissingSnafu {
                    item: "K8s version".to_string(),
                    what: "aws-k8s variant".to_string(),
                },
            )?)
            .map_err(|_| error::Error::K8sVersion {
                version: cluster_input.crd_input.variant.to_string(),
            })?;

        let (cluster_name, region, config) = match cluster_input.cluster_config {
            Some(config) => {
                let (cluster_name, region) = cluster_config_data(config)?;
                (
                    cluster_name,
                    region,
                    EksctlConfig::File {
                        encoded_config: base64::encode(config),
                    },
                )
            }
            None => (
                cluster_input.cluster_name.to_string(),
                self.region.clone(),
                EksctlConfig::Args {
                    cluster_name: cluster_input.cluster_name.to_string(),
                    region: Some(self.region.clone()),
                    zones: None,
                    version: Some(cluster_version),
                },
            ),
        };

        let labels = cluster_input.crd_input.labels(btreemap! {
            "testsys/type".to_string() => "cluster".to_string(),
            "testsys/cluster".to_string() => cluster_name.to_string(),
            "testsys/region".to_string() => region.clone()
        });

        // Check if the cluster already has a crd
        if let Some(cluster_crd) = cluster_input
            .crd_input
            .existing_crds(
                &labels,
                &["testsys/cluster", "testsys/type", "testsys/region"],
            )
            .await?
            .pop()
        {
            return Ok(CreateCrdOutput::ExistingCrd(cluster_crd));
        }

        let eks_crd = EksClusterConfig::builder()
            .creation_policy(CreationPolicy::IfNotExists)
            .assume_role(cluster_input.crd_input.config.agent_role.clone())
            .config(config)
            .image(
                cluster_input
                    .crd_input
                    .images
                    .eks_resource_agent_image
                    .to_owned()
                    .expect("Missing default image for EKS resource agent"),
            )
            .set_image_pull_secret(
                cluster_input
                    .crd_input
                    .images
                    .testsys_agent_pull_secret
                    .clone(),
            )
            .set_labels(Some(labels))
            .set_secrets(Some(cluster_input.crd_input.config.secrets.clone()))
            .destruction_policy(
                cluster_input
                    .crd_input
                    .config
                    .dev
                    .cluster_destruction_policy
                    .to_owned()
                    .unwrap_or(DestructionPolicy::Never),
            )
            .build(cluster_name)
            .context(error::BuildSnafu {
                what: "EKS cluster CRD",
            })?;

        Ok(CreateCrdOutput::NewCrd(Box::new(Crd::Resource(eks_crd))))
    }

    /// Create an EC2 provider CRD to launch Bottlerocket instances on the cluster created by
    /// `cluster_crd`.
    async fn bottlerocket_crd<'a>(
        &self,
        bottlerocket_input: BottlerocketInput<'a>,
    ) -> Result<CreateCrdOutput> {
        Ok(CreateCrdOutput::NewCrd(Box::new(Crd::Resource(
            match bottlerocket_input
                .crd_input
                .config
                .resource_agent_type
                .to_owned()
                .unwrap_or_default()
            {
                ResourceAgentType::Ec2 => {
                    ec2_crd(bottlerocket_input, ClusterType::Eks, &self.region).await?
                }
                ResourceAgentType::Karpenter => {
                    ec2_karpenter_crd(bottlerocket_input, &self.region).await?
                }
            },
        ))))
    }

    async fn migration_crd<'a>(
        &self,
        migration_input: MigrationInput<'a>,
    ) -> Result<CreateCrdOutput> {
        Ok(CreateCrdOutput::NewCrd(Box::new(Crd::Test(migration_crd(
            migration_input,
            None,
            "ids",
        )?))))
    }

    async fn test_crd<'a>(&self, test_input: TestInput<'a>) -> Result<CreateCrdOutput> {
        Ok(CreateCrdOutput::NewCrd(Box::new(Crd::Test(sonobuoy_crd(
            test_input,
        )?))))
    }

    async fn workload_crd<'a>(&self, test_input: TestInput<'a>) -> Result<CreateCrdOutput> {
        Ok(CreateCrdOutput::NewCrd(Box::new(Crd::Test(workload_crd(
            test_input,
        )?))))
    }

    fn additional_fields(&self, _test_type: &str) -> BTreeMap<String, String> {
        btreemap! {"region".to_string() => self.region.clone()}
    }
}

/// Converts a eksctl cluster config to a `serde_yaml::Value` and extracts the cluster name and
/// region from it.
fn cluster_config_data(cluster_config: &str) -> Result<(String, String)> {
    let config: Value = serde_yaml::from_str(cluster_config).context(error::SerdeYamlSnafu {
        what: "Unable to deserialize cluster config",
    })?;

    let (cluster_name, region) = config
        .get("metadata")
        .map(|metadata| {
            (
                metadata.get("name").and_then(|name| name.as_str()),
                metadata.get("region").and_then(|region| region.as_str()),
            )
        })
        .context(error::MissingSnafu {
            item: "metadata",
            what: "eksctl config",
        })?;
    Ok((
        cluster_name
            .context(error::MissingSnafu {
                item: "name",
                what: "eksctl config metadata",
            })?
            .to_string(),
        region
            .context(error::MissingSnafu {
                item: "region",
                what: "eksctl config metadata",
            })?
            .to_string(),
    ))
}
