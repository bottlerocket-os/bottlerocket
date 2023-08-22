use crate::aws_resources::{ami, ami_name, ec2_crd, get_ami_id};
use crate::crds::{
    BottlerocketInput, ClusterInput, CrdCreator, CrdInput, CreateCrdOutput, MigrationInput,
    TestInput,
};
use crate::error::{self, Result};
use crate::migration::migration_crd;
use bottlerocket_types::agent_config::{
    ClusterType, EcsClusterConfig, EcsTestConfig, EcsWorkloadTestConfig, WorkloadTest,
};
use log::debug;
use maplit::btreemap;
use snafu::{OptionExt, ResultExt};
use std::collections::BTreeMap;
use testsys_model::{Crd, DestructionPolicy, Test};

/// A `CrdCreator` responsible for creating crd related to `aws-ecs` variants.
pub(crate) struct AwsEcsCreator {
    pub(crate) region: String,
    pub(crate) ami_input: String,
    pub(crate) migrate_starting_commit: Option<String>,
}

#[async_trait::async_trait]
impl CrdCreator for AwsEcsCreator {
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

    /// Create an ECS cluster CRD with the `cluster_name` in `cluster_input`.
    async fn cluster_crd<'a>(&self, cluster_input: ClusterInput<'a>) -> Result<CreateCrdOutput> {
        debug!("Creating ECS cluster CRD");
        // Create labels that will be used for identifying existing CRDs for an ECS cluster.
        let labels = cluster_input.crd_input.labels(btreemap! {
            "testsys/type".to_string() => "cluster".to_string(),
            "testsys/cluster".to_string() => cluster_input.cluster_name.to_string(),
            "testsys/region".to_string() => self.region.clone()
        });

        // Check if the cluster already has a CRD in the TestSys cluster.
        if let Some(cluster_crd) = cluster_input
            .crd_input
            .existing_crds(
                &labels,
                &["testsys/cluster", "testsys/type", "testsys/region"],
            )
            .await?
            .pop()
        {
            // Return the name of the existing CRD for the cluster.
            debug!("ECS cluster CRD already exists with name '{}'", cluster_crd);
            return Ok(CreateCrdOutput::ExistingCrd(cluster_crd));
        }

        // Create the CRD for ECS cluster creation.
        let ecs_crd = EcsClusterConfig::builder()
            .cluster_name(cluster_input.cluster_name)
            .region(Some(self.region.to_owned()))
            .assume_role(cluster_input.crd_input.config.agent_role.clone())
            .destruction_policy(
                cluster_input
                    .crd_input
                    .config
                    .dev
                    .cluster_destruction_policy
                    .to_owned()
                    .unwrap_or(DestructionPolicy::OnTestSuccess),
            )
            .image(
                cluster_input
                    .crd_input
                    .images
                    .ecs_resource_agent_image
                    .as_ref()
                    .expect("The default ecs resource provider image uri is missing."),
            )
            .set_image_pull_secret(
                cluster_input
                    .crd_input
                    .images
                    .testsys_agent_pull_secret
                    .to_owned(),
            )
            .set_labels(Some(labels))
            .set_secrets(Some(cluster_input.crd_input.config.secrets.clone()))
            .build(cluster_input.cluster_name)
            .context(error::BuildSnafu {
                what: "ECS cluster CRD",
            })?;

        Ok(CreateCrdOutput::NewCrd(Box::new(Crd::Resource(ecs_crd))))
    }

    /// Create an EC2 provider CRD to launch Bottlerocket instances on the cluster created by
    /// `cluster_crd`.
    async fn bottlerocket_crd<'a>(
        &self,
        bottlerocket_input: BottlerocketInput<'a>,
    ) -> Result<CreateCrdOutput> {
        Ok(CreateCrdOutput::NewCrd(Box::new(Crd::Resource(
            ec2_crd(bottlerocket_input, ClusterType::Ecs, &self.region).await?,
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
        let cluster_resource_name = test_input
            .cluster_crd_name
            .as_ref()
            .expect("A cluster name is required for migrations");
        let bottlerocket_resource_name = test_input
            .bottlerocket_crd_name
            .as_ref()
            .expect("A cluster name is required for migrations");

        // Create labels that are used to help filter status.
        let labels = test_input.crd_input.labels(btreemap! {
            "testsys/type".to_string() => test_input.test_type.to_string(),
            "testsys/cluster".to_string() => cluster_resource_name.to_string(),
        });

        let test_crd = EcsTestConfig::builder()
            .cluster_name_template(cluster_resource_name, "clusterName")
            .region(Some(self.region.to_owned()))
            .task_count(1)
            .assume_role(test_input.crd_input.config.agent_role.to_owned())
            .resources(bottlerocket_resource_name)
            .resources(cluster_resource_name)
            .set_depends_on(Some(test_input.prev_tests))
            .set_retries(Some(5))
            .image(
                test_input
                    .crd_input
                    .images
                    .ecs_test_agent_image
                    .to_owned()
                    .expect("The default ECS testing image is missing"),
            )
            .set_image_pull_secret(
                test_input
                    .crd_input
                    .images
                    .testsys_agent_pull_secret
                    .to_owned(),
            )
            .keep_running(
                test_input
                    .crd_input
                    .config
                    .dev
                    .keep_tests_running
                    .unwrap_or(false),
            )
            .set_secrets(Some(test_input.crd_input.config.secrets.to_owned()))
            .set_labels(Some(labels))
            .build(format!(
                "{}-{}",
                cluster_resource_name,
                test_input
                    .name_suffix
                    .unwrap_or(test_input.crd_input.test_flavor.as_str())
            ))
            .context(error::BuildSnafu {
                what: "ECS test CRD",
            })?;

        Ok(CreateCrdOutput::NewCrd(Box::new(Crd::Test(test_crd))))
    }

    async fn workload_crd<'a>(&self, test_input: TestInput<'a>) -> Result<CreateCrdOutput> {
        Ok(CreateCrdOutput::NewCrd(Box::new(Crd::Test(workload_crd(
            &self.region,
            test_input,
        )?))))
    }

    fn additional_fields(&self, _test_type: &str) -> BTreeMap<String, String> {
        btreemap! {"region".to_string() => self.region.clone()}
    }
}

/// Create a workload CRD for K8s testing.
pub(crate) fn workload_crd(region: &str, test_input: TestInput) -> Result<Test> {
    let cluster_resource_name = test_input
        .cluster_crd_name
        .as_ref()
        .expect("A cluster name is required for ECS workload tests");
    let bottlerocket_resource_name = test_input
        .bottlerocket_crd_name
        .as_ref()
        .expect("A bottlerocket resource name is required for ECS workload tests");

    let labels = test_input.crd_input.labels(btreemap! {
        "testsys/type".to_string() => test_input.test_type.to_string(),
        "testsys/cluster".to_string() => cluster_resource_name.to_string(),
    });
    let gpu = test_input.crd_input.variant.variant_flavor() == Some("nvidia");
    let plugins: Vec<_> = test_input
        .crd_input
        .config
        .workloads
        .iter()
        .map(|(name, image)| WorkloadTest {
            name: name.to_string(),
            image: image.to_string(),
            gpu,
        })
        .collect();
    if plugins.is_empty() {
        return Err(error::Error::Invalid {
            what: "There were no plugins specified in the workload test.
            Workloads can be specified in `Test.toml` or via the command line."
                .to_string(),
        });
    }

    EcsWorkloadTestConfig::builder()
        .resources(bottlerocket_resource_name)
        .resources(cluster_resource_name)
        .set_depends_on(Some(test_input.prev_tests))
        .set_retries(Some(5))
        .image(
            test_input
                .crd_input
                .images
                .ecs_workload_agent_image
                .to_owned()
                .expect("The default K8s workload testing image is missing"),
        )
        .set_image_pull_secret(
            test_input
                .crd_input
                .images
                .testsys_agent_pull_secret
                .to_owned(),
        )
        .keep_running(true)
        .region(region.to_string())
        .cluster_name_template(cluster_resource_name, "clusterName")
        .assume_role(test_input.crd_input.config.agent_role.to_owned())
        .tests(plugins)
        .set_secrets(Some(test_input.crd_input.config.secrets.to_owned()))
        .set_labels(Some(labels))
        .build(format!(
            "{}{}",
            cluster_resource_name,
            test_input.name_suffix.unwrap_or("-test")
        ))
        .context(error::BuildSnafu {
            what: "Workload CRD",
        })
}
