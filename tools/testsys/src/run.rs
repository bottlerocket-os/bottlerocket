use crate::aws_ecs::AwsEcsCreator;
use crate::aws_k8s::AwsK8sCreator;
use crate::crds::{CrdCreator, CrdInput};
use crate::error;
use crate::error::Result;
use bottlerocket_variant::Variant;
use clap::Parser;
use log::{debug, info};
use model::test_manager::TestManager;
use model::SecretName;
use pubsys_config::InfraConfig;
use serde::{Deserialize, Serialize};
use serde_plain::{derive_display_from_serialize, derive_fromstr_from_deserialize};
use snafu::{OptionExt, ResultExt};
use std::path::PathBuf;
use testsys_config::{GenericVariantConfig, TestConfig};

/// Run a set of tests for a given arch and variant
#[derive(Debug, Parser)]
pub(crate) struct Run {
    /// The type of test to run. Options are `quick` and `conformance`.
    test_flavor: TestType,

    /// The architecture to test. Either x86_64 or aarch64.
    #[clap(long, env = "BUILDSYS_ARCH")]
    arch: String,

    /// The variant to test
    #[clap(long, env = "BUILDSYS_VARIANT")]
    variant: String,

    /// The path to `Infra.toml`
    #[clap(long, env = "PUBLISH_INFRA_CONFIG_PATH", parse(from_os_str))]
    infra_config_path: PathBuf,

    /// The path to `Test.toml`
    #[clap(long, env = "TESTSYS_TEST_CONFIG_PATH", parse(from_os_str))]
    test_config_path: PathBuf,

    /// Use this named repo infrastructure from Infra.toml for upgrade/downgrade testing.
    #[clap(long, env = "PUBLISH_REPO")]
    repo: Option<String>,

    /// The path to `amis.json`
    #[clap(long, env = "AMI_INPUT")]
    ami_input: String,

    /// Override for the region the tests should be run in. If none is provided the first region in
    /// Infra.toml will be used. This is the region that the aws client is created with for testing
    /// and resource agents.
    #[clap(long, env = "TESTSYS_TARGET_REGION")]
    target_region: Option<String>,

    #[clap(flatten)]
    agent_images: TestsysImages,

    #[clap(flatten)]
    config: CliConfig,

    // Migrations
    /// Override the starting image used for migrations. The image will be pulled from available
    /// amis in the users account if no override is provided.
    #[clap(long, env = "TESTSYS_STARTING_IMAGE_ID")]
    starting_image_id: Option<String>,

    /// The starting version for migrations. This is required for all migrations tests.
    /// This is the version that will be created and migrated to `migration-target-version`.
    #[clap(long, env = "TESTSYS_STARTING_VERSION")]
    migration_starting_version: Option<String>,

    /// The commit id of the starting version for migrations. This is required for all migrations
    /// tests unless `starting-image-id` is provided. This is the version that will be created and
    /// migrated to `migration-target-version`.
    #[clap(long, env = "TESTSYS_STARTING_COMMIT")]
    migration_starting_commit: Option<String>,

    /// The target version for migrations. This is required for all migration tests. This is the
    /// version that will be migrated to.
    #[clap(long, env = "BUILDSYS_VERSION_IMAGE")]
    migration_target_version: Option<String>,
}

/// This is a CLI parsable version of `testsys_config::GenericVariantConfig`.
#[derive(Debug, Parser)]
struct CliConfig {
    /// The repo containing images necessary for conformance testing. It may be omitted to use the
    /// default conformance image registry.
    #[clap(long, env = "TESTSYS_CONFORMANCE_REGISTRY")]
    conformance_registry: Option<String>,

    /// The name of the cluster for resource agents (EKS resource agent, ECS resource agent). Note:
    /// This is not the name of the `testsys cluster` this is the name of the cluster that tests
    /// should be run on. If no cluster name is provided, the bottlerocket cluster
    /// naming convention `{{arch}}-{{variant}}` will be used.
    #[clap(long, env = "TESTSYS_TARGET_CLUSTER_NAME")]
    target_cluster_name: Option<String>,

    /// The image that should be used for conformance testing. It may be omitted to use the default
    /// testing image.
    #[clap(long, env = "TESTSYS_CONFORMANCE_IMAGE")]
    conformance_image: Option<String>,

    /// The role that should be assumed by the agents
    #[clap(long, env = "TESTSYS_ASSUME_ROLE")]
    assume_role: Option<String>,

    /// Specify the instance type that should be used. This is only applicable for aws-* variants.
    /// It can be omitted for non-aws variants and can be omitted to use default instance types.
    #[clap(long, env = "TESTSYS_INSTANCE_TYPE")]
    instance_type: Option<String>,

    /// Add secrets to the testsys agents (`--secret awsCredentials=my-secret`)
    #[clap(long, short, parse(try_from_str = parse_key_val), number_of_values = 1)]
    secret: Vec<(String, SecretName)>,
}

impl From<CliConfig> for GenericVariantConfig {
    fn from(val: CliConfig) -> Self {
        GenericVariantConfig {
            cluster_names: val.target_cluster_name.into_iter().collect(),
            instance_type: val.instance_type,
            secrets: val.secret.into_iter().collect(),
            agent_role: val.assume_role,
            conformance_image: val.conformance_image,
            conformance_registry: val.conformance_registry,
        }
    }
}

impl Run {
    pub(crate) async fn run(self, client: TestManager) -> Result<()> {
        // agent config (eventually with configuration)
        let variant = Variant::new(&self.variant).context(error::VariantSnafu {
            variant: self.variant,
        })?;
        debug!("Using variant '{}'", variant);

        // Use Test.toml or default
        let test_config = TestConfig::from_path_or_default(&self.test_config_path)?;

        let test_opts = test_config.test.to_owned().unwrap_or_default();

        let variant_config =
            test_config.reduced_config(&variant, &self.arch, Some(self.config.into()));

        // If a lock file exists, use that, otherwise use Infra.toml or default
        let infra_config = InfraConfig::from_path_or_lock(&self.infra_config_path, true)?;

        let repo_config = infra_config
            .repo
            .unwrap_or_default()
            .remove(
                &self
                    .repo
                    .or(test_opts.repo)
                    .unwrap_or_else(|| "default".to_string()),
            )
            .unwrap_or_default();

        let images = vec![
            Some(self.agent_images.into()),
            Some(test_opts.testsys_images),
            test_opts
                .testsys_image_registry
                .map(testsys_config::TestsysImages::new),
            Some(testsys_config::TestsysImages::public_images()),
        ]
        .into_iter()
        .flatten()
        .fold(Default::default(), testsys_config::TestsysImages::merge);

        // The `CrdCreator` is responsible for creating crds for the given architecture and variant.
        let crd_creator: Box<dyn CrdCreator> = match variant.family() {
            "aws-k8s" => {
                debug!("Using family 'aws-k8s'");
                let aws_config = infra_config.aws.unwrap_or_default();
                let region = aws_config
                    .regions
                    .front()
                    .map(String::to_string)
                    .unwrap_or_else(|| "us-west-2".to_string());
                Box::new(AwsK8sCreator {
                    region,
                    ami_input: self.ami_input,
                    migrate_starting_commit: self.migration_starting_commit,
                })
            }
            "aws-ecs" => {
                debug!("Using family 'aws-ecs'");
                let aws_config = infra_config.aws.unwrap_or_default();
                let region = aws_config
                    .regions
                    .front()
                    .map(String::to_string)
                    .unwrap_or_else(|| "us-west-2".to_string());
                Box::new(AwsEcsCreator {
                    region,
                    ami_input: self.ami_input,
                    migrate_starting_commit: self.migration_starting_commit,
                })
            }
            unsupported => {
                return Err(error::Error::Unsupported {
                    what: unsupported.to_string(),
                })
            }
        };

        let crd_input = CrdInput {
            client: &client,
            arch: self.arch,
            variant,
            config: variant_config,
            repo_config,
            starting_version: self.migration_starting_version,
            migrate_to_version: self.migration_target_version,
            starting_image_id: self.starting_image_id,
            images,
        };

        let crds = crd_creator
            .create_crds(self.test_flavor, &crd_input)
            .await?;

        debug!("Adding crds to testsys cluster");
        for crd in crds {
            let crd = client.create_object(crd).await?;
            info!("Successfully added '{}'", crd.name().unwrap());
        }

        Ok(())
    }
}

fn parse_key_val(s: &str) -> Result<(String, SecretName)> {
    let mut iter = s.splitn(2, '=');
    let key = iter.next().context(error::InvalidSnafu {
        what: "Key is missing",
    })?;
    let value = iter.next().context(error::InvalidSnafu {
        what: "Value is missing",
    })?;
    Ok((
        key.to_string(),
        SecretName::new(value).context(error::SecretNameSnafu { secret_name: value })?,
    ))
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KnownTestType {
    /// Conformance testing is a full integration test that asserts that Bottlerocket is working for
    /// customer workloads. For k8s variants, for example, this will run the full suite of sonobuoy
    /// conformance tests.
    Conformance,
    /// Run a quick test that ensures a basic workload can run on Bottlerocket. For example, on k8s
    /// variance this will run sonobuoy in "quick" mode. For ECS variants, this will run a simple
    /// ECS task.
    Quick,
    /// Migration testing ensures that all bottlerocket migrations work as expected. Instances will
    /// be created at the starting version, migrated to the target version and back to the starting
    /// version with validation testing.
    Migration,
}

/// If a test type is one that is supported by TestSys it will be created as `Known(KnownTestType)`.
/// All other test types will be stored as `Unknown(<TEST-TYPE>)`.
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub(crate) enum TestType {
    Known(KnownTestType),
    Unknown(String),
}

derive_fromstr_from_deserialize!(TestType);
derive_display_from_serialize!(TestType);
derive_display_from_serialize!(KnownTestType);

/// This is a CLI parsable version of `testsys_config::TestsysImages`
#[derive(Debug, Parser)]
pub(crate) struct TestsysImages {
    /// EKS resource agent URI. If not provided the latest released resource agent will be used.
    #[clap(
        long = "eks-resource-agent-image",
        env = "TESTSYS_EKS_RESOURCE_AGENT_IMAGE"
    )]
    pub(crate) eks_resource: Option<String>,

    /// ECS resource agent URI. If not provided the latest released resource agent will be used.
    #[clap(
        long = "ecs-resource-agent-image",
        env = "TESTSYS_ECS_RESOURCE_AGENT_IMAGE"
    )]
    pub(crate) ecs_resource: Option<String>,

    /// EC2 resource agent URI. If not provided the latest released resource agent will be used.
    #[clap(
        long = "ec2-resource-agent-image",
        env = "TESTSYS_EC2_RESOURCE_AGENT_IMAGE"
    )]
    pub(crate) ec2_resource: Option<String>,

    /// Sonobuoy test agent URI. If not provided the latest released test agent will be used.
    #[clap(
        long = "sonobuoy-test-agent-image",
        env = "TESTSYS_SONOBUOY_TEST_AGENT_IMAGE"
    )]
    pub(crate) sonobuoy_test: Option<String>,

    /// ECS test agent URI. If not provided the latest released test agent will be used.
    #[clap(long = "ecs-test-agent-image", env = "TESTSYS_ECS_TEST_AGENT_IMAGE")]
    pub(crate) ecs_test: Option<String>,

    /// Migration test agent URI. If not provided the latest released test agent will be used.
    #[clap(
        long = "migration-test-agent-image",
        env = "TESTSYS_MIGRATION_TEST_AGENT_IMAGE"
    )]
    pub(crate) migration_test: Option<String>,

    /// Images pull secret. This is the name of a Kubernetes secret that will be used to
    /// pull the container image from a private registry. For example, if you created a pull secret
    /// with `kubectl create secret docker-registry regcred` then you would pass
    /// `--images-pull-secret regcred`.
    #[clap(long = "images-pull-secret", env = "TESTSYS_IMAGES_PULL_SECRET")]
    pub(crate) secret: Option<String>,
}

impl From<TestsysImages> for testsys_config::TestsysImages {
    fn from(val: TestsysImages) -> Self {
        testsys_config::TestsysImages {
            eks_resource_agent_image: val.eks_resource,
            ecs_resource_agent_image: val.ecs_resource,
            ec2_resource_agent_image: val.ec2_resource,
            sonobuoy_test_agent_image: val.sonobuoy_test,
            ecs_test_agent_image: val.ecs_test,
            migration_test_agent_image: val.migration_test,
            testsys_agent_pull_secret: val.secret,
        }
    }
}
