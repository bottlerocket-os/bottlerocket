use crate::aws_ecs::AwsEcsCreator;
use crate::aws_k8s::AwsK8sCreator;
use crate::crds::{CrdCreator, CrdInput};
use crate::error;
use crate::error::Result;
use crate::vmware_k8s::VmwareK8sCreator;
use bottlerocket_variant::Variant;
use clap::Parser;
use log::{debug, info};
use pubsys_config::vmware::{
    Datacenter, DatacenterBuilder, DatacenterCreds, DatacenterCredsBuilder, DatacenterCredsConfig,
    VMWARE_CREDS_PATH,
};
use pubsys_config::InfraConfig;
use serde::{Deserialize, Serialize};
use serde_plain::{derive_display_from_serialize, derive_fromstr_from_deserialize};
use snafu::{OptionExt, ResultExt};
use std::fs::read_to_string;
use std::path::PathBuf;
use std::str::FromStr;
use testsys_config::{GenericVariantConfig, TestConfig};
use testsys_model::test_manager::TestManager;
use testsys_model::SecretName;

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

    /// The path to the `tests` directory
    #[clap(long, env = "TESTSYS_TESTS_DIR", parse(from_os_str))]
    tests_directory: PathBuf,

    /// The path to the EKS-A management cluster kubeconfig for vSphere K8s cluster creation
    #[clap(long, env = "TESTSYS_MGMT_CLUSTER_KUBECONFIG", parse(from_os_str))]
    mgmt_cluster_kubeconfig: Option<PathBuf>,

    /// Use this named repo infrastructure from Infra.toml for upgrade/downgrade testing.
    #[clap(long, env = "PUBLISH_REPO")]
    repo: Option<String>,

    /// The name of the vSphere data center in `Infra.toml` that should be used for testing
    /// If no data center is provided, the first one in `vmware.datacenters` will be used
    #[clap(long, env = "TESTSYS_DATACENTER")]
    datacenter: Option<String>,

    /// The name of the VMware OVA that should be used for testing
    #[clap(long, env = "BUILDSYS_OVA")]
    ova_name: Option<String>,

    /// The path to `amis.json`
    #[clap(long, env = "AMI_INPUT")]
    ami_input: Option<String>,

    /// Override for the region the tests should be run in. If none is provided the first region in
    /// Infra.toml will be used. This is the region that the aws client is created with for testing
    /// and resource agents.
    #[clap(long, env = "TESTSYS_TARGET_REGION")]
    target_region: Option<String>,

    #[clap(long, env = "BUILDSYS_VERSION_BUILD")]
    build_id: Option<String>,

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

    /// The template file that should be used for custom testing.
    #[clap(long = "template-file", short = 'f', parse(from_os_str))]
    custom_crd_template: Option<PathBuf>,
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

    /// The endpoint IP to reserve for the vSphere control plane VMs when creating a K8s cluster
    #[clap(long, env = "TESTSYS_CONTROL_PLANE_ENDPOINT")]
    pub control_plane_endpoint: Option<String>,

    /// Specify the path to the userdata that should be added for Bottlerocket launch
    #[clap(long, env = "TESTSYS_USERDATA")]
    pub userdata: Option<String>,

    /// A set of workloads that should be run for a workload test (--workload my-workload=<WORKLOAD-IMAGE>)
    #[clap(long = "workload", parse(try_from_str = parse_workloads), number_of_values = 1)]
    pub workloads: Vec<(String, String)>,
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
            control_plane_endpoint: val.control_plane_endpoint,
            userdata: val.userdata,
            dev: Default::default(),
            workloads: val.workloads.into_iter().collect(),
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

        let (variant_config, test_type) = test_config.reduced_config(
            &variant,
            &self.arch,
            Some(self.config.into()),
            &self.test_flavor.to_string(),
        );
        let resolved_test_type = TestType::from_str(&test_type)
            .expect("All unrecognized test type become `TestType::Custom`");

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
            test_opts.testsys_image_registry.map(|registry| {
                testsys_config::TestsysImages::new(registry, test_opts.testsys_image_tag)
            }),
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
                    ami_input: self.ami_input.context(error::InvalidSnafu {
                        what: "amis.json is required. You may need to run `cargo make ami`",
                    })?,
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
                    ami_input: self.ami_input.context(error::InvalidSnafu {
                        what: "amis.json is required. You may need to run `cargo make ami`",
                    })?,
                    migrate_starting_commit: self.migration_starting_commit,
                })
            }
            "vmware-k8s" => {
                debug!("Using family 'vmware-k8s'");
                let aws_config = infra_config.aws.unwrap_or_default();
                let region = aws_config
                    .regions
                    .front()
                    .map(String::to_string)
                    .unwrap_or_else(|| "us-west-2".to_string());
                let vmware_config = infra_config.vmware.unwrap_or_default();
                let dc_env = DatacenterBuilder::from_env();
                let dc_common = vmware_config.common.as_ref();
                let dc_config = self
                    .datacenter
                    .as_ref()
                    .or_else(|| vmware_config.datacenters.first())
                    .and_then(|datacenter| vmware_config.datacenter.get(datacenter));

                let datacenter: Datacenter = dc_env
                    .take_missing_from(dc_config)
                    .take_missing_from(dc_common)
                    .build()
                    .context(error::DatacenterBuildSnafu)?;

                let vsphere_secret = if !variant_config.secrets.contains_key("vsphereCredentials") {
                    info!("Creating vSphere secret, 'vspherecreds'");
                    let creds_env = DatacenterCredsBuilder::from_env();
                    let creds_file = if let Some(ref creds_file) = *VMWARE_CREDS_PATH {
                        if creds_file.exists() {
                            info!("Using vSphere credentials file at {}", creds_file.display());
                            DatacenterCredsConfig::from_path(creds_file)
                                .context(error::VmwareConfigSnafu)?
                        } else {
                            info!(
                            "vSphere credentials file not found, will attempt to use environment"
                        );
                            DatacenterCredsConfig::default()
                        }
                    } else {
                        info!("Unable to determine vSphere credentials file location, will attempt to use environment");
                        DatacenterCredsConfig::default()
                    };
                    let dc_creds = creds_file.datacenter.get(&datacenter.datacenter);
                    let creds: DatacenterCreds = creds_env
                        .take_missing_from(dc_creds)
                        .build()
                        .context(error::CredsBuildSnafu)?;

                    let secret_name =
                        SecretName::new("vspherecreds").context(error::SecretNameSnafu {
                            secret_name: "vspherecreds",
                        })?;
                    client
                        .create_secret(
                            &secret_name,
                            vec![
                                ("username".to_string(), creds.username),
                                ("password".to_string(), creds.password),
                            ],
                        )
                        .await?;
                    Some(("vsphereCredentials".to_string(), secret_name))
                } else {
                    None
                };

                let mgmt_cluster_kubeconfig =
                    self.mgmt_cluster_kubeconfig.context(error::InvalidSnafu {
                        what: "A management cluster kubeconfig is required for VMware testing",
                    })?;
                let encoded_kubeconfig = base64::encode(
                    read_to_string(&mgmt_cluster_kubeconfig).context(error::FileSnafu {
                        path: mgmt_cluster_kubeconfig,
                    })?,
                );

                Box::new(VmwareK8sCreator {
                    region,
                    ova_name: self.ova_name.context(error::InvalidSnafu {
                        what: "An OVA name is required for VMware testing.",
                    })?,
                    datacenter,
                    encoded_mgmt_cluster_kubeconfig: encoded_kubeconfig,
                    creds: vsphere_secret,
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
            build_id: self.build_id,
            config: variant_config,
            repo_config,
            starting_version: self.migration_starting_version,
            migrate_to_version: self.migration_target_version,
            starting_image_id: self.starting_image_id,
            test_type: resolved_test_type.clone(),
            test_flavor: self.test_flavor.to_string(),
            images,
            tests_directory: self.tests_directory,
        };

        let crds = match &resolved_test_type {
            TestType::Known(resolved_test_type) => {
                crd_creator
                    .create_crds(resolved_test_type, &crd_input)
                    .await?
            }
            TestType::Custom(resolved_test_type) => {
                crd_creator
                    .create_custom_crds(
                        resolved_test_type,
                        &crd_input,
                        self.custom_crd_template.to_owned(),
                    )
                    .await?
            }
        };

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

fn parse_workloads(s: &str) -> Result<(String, String)> {
    let mut iter = s.splitn(2, '=');
    let key = iter.next().context(error::InvalidSnafu {
        what: "Key is missing",
    })?;
    let value = iter.next().context(error::InvalidSnafu {
        what: "Value is missing",
    })?;
    Ok((key.to_string(), value.to_string()))
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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
    /// Workload testing is used to test specific workloads on a set of Bottlerocket nodes.
    Workload,
}

/// If a test type is one that is supported by TestSys it will be created as `Known(KnownTestType)`.
/// All other test types will be stored as `Custom(<TEST-TYPE>)`.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub(crate) enum TestType {
    Known(KnownTestType),
    Custom(String),
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

    /// vSphere cluster resource agent URI. If not provided the latest released resource agent will be used.
    #[clap(
        long = "vsphere-k8s-cluster-resource-agent-image",
        env = "TESTSYS_VSPHERE_K8S_CLUSTER_RESOURCE_AGENT_IMAGE"
    )]
    pub(crate) vsphere_k8s_cluster_resource: Option<String>,

    /// EC2 resource agent URI. If not provided the latest released resource agent will be used.
    #[clap(
        long = "ec2-resource-agent-image",
        env = "TESTSYS_EC2_RESOURCE_AGENT_IMAGE"
    )]
    pub(crate) ec2_resource: Option<String>,

    /// vSphere VM resource agent URI. If not provided the latest released resource agent will be used.
    #[clap(
        long = "vsphere-vm-resource-agent-image",
        env = "TESTSYS_VSPHERE_VM_RESOURCE_AGENT_IMAGE"
    )]
    pub(crate) vsphere_vm_resource: Option<String>,

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

    /// K8s workload agent URI. If not provided the latest released test agent will be used.
    #[clap(
        long = "k8s-workload-agent-image",
        env = "TESTSYS_K8S_WORKLOAD_AGENT_IMAGE"
    )]
    pub(crate) k8s_workload: Option<String>,

    /// TestSys controller URI. If not provided the latest released controller will be used.
    #[clap(long = "controller-image", env = "TESTSYS_CONTROLLER_IMAGE")]
    pub(crate) controller_uri: Option<String>,

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
            vsphere_k8s_cluster_resource_agent_image: val.vsphere_k8s_cluster_resource,
            ec2_resource_agent_image: val.ec2_resource,
            vsphere_vm_resource_agent_image: val.vsphere_vm_resource,
            sonobuoy_test_agent_image: val.sonobuoy_test,
            ecs_test_agent_image: val.ecs_test,
            migration_test_agent_image: val.migration_test,
            k8s_workload_agent_image: val.k8s_workload,
            controller_image: val.controller_uri,
            testsys_agent_pull_secret: val.secret,
        }
    }
}
