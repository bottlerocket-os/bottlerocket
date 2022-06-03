use crate::aws_resources::AwsK8s;
use anyhow::{anyhow, ensure, Context, Result};
use bottlerocket_types::agent_config::TufRepoConfig;
use buildsys::Variant;
use clap::Parser;
use model::test_manager::TestManager;
use model::SecretName;
use pubsys_config::InfraConfig;
use serde::Deserialize;
use serde_plain::derive_fromstr_from_deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;

/// Run a set of tests on a given arch and variant
#[derive(Debug, Parser)]
pub(crate) struct Run {
    /// The type of test to run
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

    /// Use this named repo infrastructure from Infra.toml for upgrade/downgrade testing.
    #[clap(long, env = "PUBLISH_REPO", default_value = "default")]
    repo: String,

    /// The path to `amis.json`
    #[clap(long)]
    ami_input: String,

    /// Override for the region the tests should be run in. If none is provided the first region in
    /// Infra.toml will be used. This is the region that the aws client is created with for testing
    /// and resource agents.
    #[clap(long, env = "TESTSYS_TARGET_REGION")]
    target_region: Option<String>,

    /// The name of the cluster for resource agents (eks resource agent, ecs resource agent). Note:
    /// This is not the name of the `testsys cluster` this is the name of the cluster that tests
    /// should be run on. If no cluster name is provided, the bottlerocket cluster
    /// naming convention `<arch>-<variant>` will be used.
    #[clap(long, env = "TESTSYS_TARGET_CLUSTER_NAME")]
    target_cluster_name: Option<String>,

    /// The custom kube conformance image that should be used by sonobuoy.
    #[clap(long)]
    kube_conformance_image: Option<String>,

    /// The role that should be assumed by the agents
    #[clap(long, env = "TESTSYS_ASSUME_ROLE")]
    assume_role: Option<String>,

    /// Specify the instance type that should be used
    #[clap(long)]
    instance_type: Option<String>,

    /// Add secrets to the testsys agents (`--secret aws-credentials=my-secret`)
    #[clap(long, short, parse(try_from_str = parse_key_val), number_of_values = 1)]
    secret: Vec<(String, SecretName)>,

    #[clap(flatten)]
    agent_images: TestsysImages,

    // Migrations
    /// Override the ami used for migrations. The ami will be pulled from ssm parameters for aws
    /// variants if no override is provided.
    #[clap(long, env = "TESTSYS_STARTING_IMAGE_ID")]
    starting_image_id: Option<String>,

    /// The starting version of bottlerocket migrations. This is required for all migrations tests.
    /// This is the version that will be created and migrated to `migration_ending_version`.
    #[clap(long, env = "TESTSYS_STARTING_VERSION")]
    migration_starting_version: Option<String>,

    /// The target version of bottlerocket migrations. This is required for all migration
    /// tests. This is the version that will be migrated to.
    #[clap(long, env = "BUILDSYS_VERSION_IMAGE")]
    migration_target_version: Option<String>,
}

impl Run {
    pub(crate) async fn run(self, client: TestManager) -> Result<()> {
        let variant =
            Variant::new(&self.variant).context("The provided variant cannot be interpreted.")?;
        let secrets = if self.secret.is_empty() {
            None
        } else {
            Some(self.secret.into_iter().collect())
        };
        // If a lock file exists, use that, otherwise use Infra.toml or default
        let infra_config = InfraConfig::from_path_or_lock(&self.infra_config_path, true)
            .context("Unable to read infra config")?;

        let aws = infra_config.aws.unwrap_or_default();

        // If the user gave an override region, use that, otherwise use the first region from the
        // config.
        let region = if let Some(region) = self.target_region {
            region
        } else {
            aws.regions
                .clone()
                .pop_front()
                .context("No region was provided and no regions found in infra config")?
        };

        let repo_config = infra_config
            .repo
            .unwrap_or_default()
            .get(&self.repo)
            .and_then(|repo| {
                if let (Some(metadata_base_url), Some(targets_url)) =
                    (&repo.metadata_base_url, &repo.targets_url)
                {
                    Some(TufRepoConfig {
                        metadata_url: format!(
                            "{}{}/{}",
                            metadata_base_url, &self.variant, &self.arch
                        ),
                        targets_url: targets_url.to_string(),
                    })
                } else {
                    None
                }
            });

        match variant.family() {
            "aws-k8s" => {
                let file = File::open(&self.ami_input).context("Unable to open amis.json")?;
                let ami_input: HashMap<String, Image> = serde_json::from_reader(file)
                    .context(format!("Unable to deserialize '{}'", self.ami_input))?;
                ensure!(!ami_input.is_empty(), "amis.json is empty");
                let bottlerocket_ami = &ami_input
                    .get(&region)
                    .context(format!("ami not found for region '{}'", region))?
                    .id;
                let aws_k8s = AwsK8s {
                    arch: self.arch,
                    variant: self.variant,
                    region,
                    assume_role: self.assume_role,
                    instance_type: self.instance_type,
                    ami: bottlerocket_ami.to_string(),
                    secrets,
                    kube_conformance_image: self.kube_conformance_image,
                    target_cluster_name: self.target_cluster_name,
                    tuf_repo: repo_config,
                    starting_version: self.migration_starting_version,
                    starting_image_id: self.starting_image_id,
                    migrate_to_version: self.migration_target_version,
                    capabilities: None,
                };
                let crds = aws_k8s
                    .create_crds(self.test_flavor, &self.agent_images)
                    .await?;
                for crd in crds {
                    let crd = client
                        .create_object(crd)
                        .await
                        .context("Unable to create object")
                        .unwrap();
                    println!("Successfully added '{:#?}'", crd.name());
                }
            }
            other => {
                return Err(anyhow!(
                    "testsys has not yet added support for the '{}' variant family",
                    other
                ))
            }
        };

        Ok(())
    }
}

fn parse_key_val(s: &str) -> Result<(String, SecretName)> {
    let mut iter = s.splitn(2, '=');
    let key = iter.next().context("Key is missing")?;
    let value = iter.next().context("Value is missing")?;
    Ok((key.to_string(), SecretName::new(value)?))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum TestType {
    /// Run conformance testing on a given arch and variant
    Conformance,
    /// Run a quick test on a given arch and variant
    Quick,
    /// Run an upgrade downgrade test on a given arch and variant
    Migration,
}

derive_fromstr_from_deserialize!(TestType);

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct Image {
    pub(crate) id: String,
    #[serde(rename = "name")]
    pub(crate) _name: String,
}

#[derive(Debug, Parser)]
pub(crate) struct TestsysImages {
    /// Eks resource agent uri. If not provided the latest released resource agent will be used.
    #[clap(
        long = "eks-resource-agent-image",
        env = "TESTSYS_EKS_RESOURCE_AGENT_IMAGE",
        default_value = "public.ecr.aws/bottlerocket-test/eks-resource-agent:v0.0.1"
    )]
    pub(crate) eks_resource: String,

    /// Ec2 resource agent uri. If not provided the latest released resource agent will be used.
    #[clap(
        long = "ec2-resource-agent-image",
        env = "TESTSYS_EC2_RESOURCE_AGENT_IMAGE",
        default_value = "public.ecr.aws/bottlerocket-test/ec2-resource-agent:v0.0.1"
    )]
    pub(crate) ec2_resource: String,

    /// Sonobuoy test agent uri. If not provided the latest released test agent will be used.
    #[clap(
        long = "sonobuoy-test-agent-image",
        env = "TESTSYS_SONOBUOY_TEST_AGENT_IMAGE",
        default_value = "public.ecr.aws/bottlerocket-test/sonobuoy-test-agent:v0.0.1"
    )]
    pub(crate) sonobuoy_test: String,

    /// Migration test agent uri. If not provided the latest released test agent will be used.
    #[clap(
        long = "migration-test-agent-image",
        env = "TESTSYS_MIGRATION_TEST_AGENT_IMAGE",
        default_value = "public.ecr.aws/bottlerocket-test/migration-test-agent:v0.0.1"
    )]
    pub(crate) migration_test: String,

    /// Images pull secret. This is the name of a Kubernetes secret that will be used to
    /// pull the container image from a private registry. For example, if you created a pull secret
    /// with `kubectl create secret docker-registry regcred` then you would pass
    /// `--images-pull-secret regcred`.
    #[clap(long = "images-pull-secret", env = "TESTSYS_IMAGES_PULL_SECRET")]
    pub(crate) secret: Option<String>,
}
