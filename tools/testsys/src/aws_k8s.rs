use crate::aws_resources::{ami, ami_name, ec2_crd, get_ami_id};
use crate::crds::{
    BottlerocketInput, ClusterInput, CrdCreator, CrdInput, CreateCrdOutput, MigrationInput,
    TestInput,
};
use crate::error::{self, Result};
use crate::migration::migration_crd;
use crate::sonobuoy::sonobuoy_crd;
use bottlerocket_types::agent_config::{
    ClusterType, CreationPolicy, EksClusterConfig, EksctlConfig, K8sVersion,
};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use maplit::btreemap;
use model::constants::NAMESPACE;
use model::{Agent, Configuration, Crd, DestructionPolicy, Resource, ResourceSpec};
use snafu::{OptionExt, ResultExt};
use std::collections::BTreeMap;
use std::str::FromStr;

/// A `CrdCreator` responsible for creating crd related to `aws-k8s` variants.
pub(crate) struct AwsK8sCreator {
    pub(crate) region: String,
    pub(crate) ami_input: String,
    pub(crate) migrate_starting_commit: Option<String>,
}

#[async_trait::async_trait]
impl CrdCreator for AwsK8sCreator {
    /// Determine the AMI from `amis.json`.
    fn image_id(&self, _: &CrdInput) -> Result<String> {
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
        )
        .await
    }

    /// Create an EKS cluster CRD with the `cluster_name` in `cluster_input`.
    async fn cluster_crd<'a>(&self, cluster_input: ClusterInput<'a>) -> Result<CreateCrdOutput> {
        let labels = cluster_input.crd_input.labels(btreemap! {
            "testsys/type".to_string() => "cluster".to_string(),
            "testsys/cluster".to_string() => cluster_input.cluster_name.to_string(),
            "testsys/region".to_string() => self.region.clone()
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

        let eks_crd = Resource {
            metadata: ObjectMeta {
                name: Some(cluster_input.cluster_name.to_string()),
                namespace: Some(NAMESPACE.into()),
                labels: Some(labels),
                ..Default::default()
            },
            spec: ResourceSpec {
                depends_on: None,
                conflicts_with: None,
                agent: Agent {
                    name: "eks-provider".to_string(),
                    image: cluster_input
                        .crd_input
                        .images
                        .eks_resource_agent_image
                        .to_owned()
                        .expect("Missing default image for EKS resource agent"),
                    pull_secret: cluster_input
                        .crd_input
                        .images
                        .testsys_agent_pull_secret
                        .clone(),
                    keep_running: false,
                    timeout: None,
                    configuration: Some(
                        EksClusterConfig {
                            creation_policy: Some(CreationPolicy::IfNotExists),
                            assume_role: cluster_input.crd_input.config.agent_role.clone(),
                            config: EksctlConfig::Args {
                                cluster_name: cluster_input.cluster_name.to_string(),
                                region: Some(self.region.clone()),
                                zones: None,
                                version: Some(cluster_version),
                            },
                        }
                        .into_map()
                        .context(error::IntoMapSnafu {
                            what: "eks crd config".to_string(),
                        })?,
                    ),
                    secrets: Some(cluster_input.crd_input.config.secrets.clone()),
                    ..Default::default()
                },
                destruction_policy: DestructionPolicy::Never,
            },
            status: None,
        };
        Ok(CreateCrdOutput::NewCrd(Box::new(Crd::Resource(eks_crd))))
    }

    /// Create an EC2 provider CRD to launch Bottlerocket instances on the cluster created by
    /// `cluster_crd`.
    async fn bottlerocket_crd<'a>(
        &self,
        bottlerocket_input: BottlerocketInput<'a>,
    ) -> Result<CreateCrdOutput> {
        Ok(CreateCrdOutput::NewCrd(Box::new(Crd::Resource(
            ec2_crd(bottlerocket_input, ClusterType::Eks, &self.region).await?,
        ))))
    }

    async fn migration_crd<'a>(
        &self,
        migration_input: MigrationInput<'a>,
    ) -> Result<CreateCrdOutput> {
        Ok(CreateCrdOutput::NewCrd(Box::new(Crd::Test(migration_crd(
            migration_input,
            None,
        )?))))
    }

    async fn test_crd<'a>(&self, test_input: TestInput<'a>) -> Result<CreateCrdOutput> {
        Ok(CreateCrdOutput::NewCrd(Box::new(Crd::Test(sonobuoy_crd(
            test_input,
        )?))))
    }

    fn additional_fields(&self, _test_type: &str) -> BTreeMap<String, String> {
        btreemap! {"region".to_string() => self.region.clone()}
    }
}
