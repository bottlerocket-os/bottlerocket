use crate::run::TestType;
use anyhow::{anyhow, Context, Result};
use bottlerocket_types::agent_config::{
    ClusterType, CreationPolicy, Ec2Config, EcsClusterConfig, EcsTestConfig, EksClusterConfig,
    K8sVersion, MigrationConfig, SonobuoyConfig, SonobuoyMode, TufRepoConfig,
};

use aws_sdk_ec2::model::{Filter, Image};
use aws_sdk_ec2::Region;
use bottlerocket_variant::Variant;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use k8s_openapi::serde_json::Value;
use log::debug;
use maplit::btreemap;
use model::constants::NAMESPACE;
use model::{
    Agent, Configuration, Crd, DestructionPolicy, Resource, ResourceSpec, SecretName, Test,
    TestSpec,
};
use std::collections::BTreeMap;
use testsys_config::{
    rendered_cluster_name, AwsEcsVariantConfig, AwsK8sVariantConfig, TestsysImages,
};

pub(crate) struct AwsK8s {
    pub(crate) arch: String,
    pub(crate) variant: String,
    pub(crate) region: String,
    pub(crate) ami: String,
    pub(crate) config: AwsK8sVariantConfig,
    pub(crate) tuf_repo: Option<TufRepoConfig>,
    pub(crate) starting_version: Option<String>,
    pub(crate) migrate_starting_commit: Option<String>,
    pub(crate) starting_image_id: Option<String>,
    pub(crate) migrate_to_version: Option<String>,
    pub(crate) capabilities: Option<Vec<String>>,
}

impl AwsK8s {
    /// Create the necessary test and resource crds for the specified test type.
    pub(crate) async fn create_crds(
        &self,
        test: TestType,
        testsys_images: &TestsysImages,
    ) -> Result<Vec<Crd>> {
        let mut crds = Vec::new();
        let target_cluster_names = if self.config.cluster_names.is_empty() {
            debug!("No cluster names were provided using default name");
            vec![self.default_cluster_name()]
        } else {
            self.config.cluster_names.clone()
        };
        for template_cluster_name in target_cluster_names {
            let cluster_name = &rendered_cluster_name(
                template_cluster_name,
                self.kube_arch(),
                self.kube_variant(),
            )?;
            crds.append(&mut match &test {
                TestType::Conformance => self.sonobuoy_test_crds(
                    testsys_images,
                    SonobuoyMode::CertifiedConformance,
                    cluster_name,
                )?,
                TestType::Quick => {
                    self.sonobuoy_test_crds(testsys_images, SonobuoyMode::Quick, cluster_name)?
                }
                TestType::Migration => {
                    self.migration_test_crds(cluster_name, testsys_images)
                        .await?
                }
            })
        }
        Ok(crds)
    }

    fn sonobuoy_test_crds(
        &self,
        testsys_images: &TestsysImages,
        sonobuoy_mode: SonobuoyMode,
        cluster_name: &str,
    ) -> Result<Vec<Crd>> {
        let crds = vec![
            self.eks_crd(cluster_name, testsys_images)?,
            self.ec2_crd(cluster_name, testsys_images, None)?,
            self.sonobuoy_crd("-test", cluster_name, sonobuoy_mode, None, testsys_images)?,
        ];
        Ok(crds)
    }

    /// Creates `Test` crds for migration testing.
    async fn migration_test_crds(
        &self,
        cluster_name: &str,
        testsys_images: &TestsysImages,
    ) -> Result<Vec<Crd>> {
        let ami = if let Some(ami) = self.starting_image_id.to_owned() {
            ami
        } else {
            get_ami_id(
                    format!(
                        "bottlerocket-{}-{}-{}-{}",
                        self.variant, self.arch, self.starting_version.as_ref().context("The starting version must be provided for migration testing")?, self.migrate_starting_commit.as_ref().context("The commit for the starting version must be provided if the starting image id is not")?
                    ), & self.arch,
                    self.region.to_string(),
                )
                .await?
        };
        let eks = self.eks_crd(cluster_name, testsys_images)?;
        let ec2 = self.ec2_crd(cluster_name, testsys_images, Some(ami))?;
        let instance_provider = ec2
            .name()
            .expect("The EC2 instance provider crd is missing a name.");
        let mut depends_on = Vec::new();
        // Start with a `quick` test to make sure instances launched properly
        let initial = self.sonobuoy_crd(
            "-1-initial",
            cluster_name,
            SonobuoyMode::Quick,
            None,
            testsys_images,
        )?;
        depends_on.push(initial.name().context("Crd missing name")?);
        // Migrate instances to the target version
        let start_migrate = self.migration_crd(
            format!("{}-2-migrate", cluster_name),
            instance_provider.clone(),
            MigrationVersion::Migrated,
            Some(depends_on.clone()),
            testsys_images,
        )?;
        // A `quick` test to validate the migration
        depends_on.push(start_migrate.name().context("Crd missing name")?);
        let migrated = self.sonobuoy_crd(
            "-3-migrated",
            cluster_name,
            SonobuoyMode::Quick,
            Some(depends_on.clone()),
            testsys_images,
        )?;
        // Migrate instances to the starting version
        depends_on.push(migrated.name().context("Crd missing name")?);
        let end_migrate = self.migration_crd(
            format!("{}-4-migrate", cluster_name),
            instance_provider,
            MigrationVersion::Starting,
            Some(depends_on.clone()),
            testsys_images,
        )?;
        // A final quick test to validate the migration back to the starting version
        depends_on.push(end_migrate.name().context("Crd missing name")?);
        let last = self.sonobuoy_crd(
            "-5-final",
            cluster_name,
            SonobuoyMode::Quick,
            Some(depends_on.clone()),
            testsys_images,
        )?;
        Ok(vec![
            eks,
            ec2,
            initial,
            start_migrate,
            migrated,
            end_migrate,
            last,
        ])
    }

    /// Labels help filter test results with `testsys status`.
    fn labels(&self) -> BTreeMap<String, String> {
        btreemap! {
            "testsys/arch".to_string() => self.arch.to_string(),
            "testsys/variant".to_string() => self.variant.to_string(),
        }
    }

    fn kube_arch(&self) -> String {
        self.arch.replace('_', "-")
    }

    fn kube_variant(&self) -> String {
        self.variant.replace('.', "")
    }

    /// Bottlerocket cluster naming convention.
    fn default_cluster_name(&self) -> String {
        format!("{}-{}", self.kube_arch(), self.kube_variant())
    }

    fn eks_crd(&self, cluster_name: &str, testsys_images: &TestsysImages) -> Result<Crd> {
        let cluster_version = K8sVersion::parse(
            Variant::new(&self.variant)
                .context("The provided variant cannot be interpreted.")?
                .version()
                .context("aws-k8s variant is missing k8s version")?,
        )
        .map_err(|e| anyhow!(e))?;
        let eks_crd = Resource {
            metadata: ObjectMeta {
                name: Some(cluster_name.to_string()),
                namespace: Some(NAMESPACE.into()),
                labels: Some(self.labels()),
                ..Default::default()
            },
            spec: ResourceSpec {
                depends_on: None,
                conflicts_with: None,
                agent: Agent {
                    name: "eks-provider".to_string(),
                    image: testsys_images
                        .eks_resource_agent_image
                        .to_owned()
                        .expect("Missing default image for EKS resource agent"),
                    pull_secret: testsys_images.testsys_agent_pull_secret.clone(),
                    keep_running: false,
                    timeout: None,
                    configuration: Some(
                        EksClusterConfig {
                            cluster_name: cluster_name.to_string(),
                            creation_policy: Some(CreationPolicy::IfNotExists),
                            region: Some(self.region.clone()),
                            zones: None,
                            version: Some(cluster_version),
                            assume_role: self.config.assume_role.clone(),
                        }
                        .into_map()
                        .context("Unable to convert eks config to map")?,
                    ),
                    secrets: Some(self.config.secrets.clone()),
                    capabilities: None,
                },
                destruction_policy: DestructionPolicy::Never,
            },
            status: None,
        };
        Ok(Crd::Resource(eks_crd))
    }

    fn ec2_crd(
        &self,
        cluster_name: &str,
        testsys_images: &TestsysImages,
        override_ami: Option<String>,
    ) -> Result<Crd> {
        let mut ec2_config = Ec2Config {
            node_ami: override_ami.unwrap_or_else(|| self.ami.clone()),
            instance_count: Some(2),
            instance_type: self.config.instance_type.clone(),
            cluster_name: format!("${{{}.clusterName}}", cluster_name),
            region: format!("${{{}.region}}", cluster_name),
            instance_profile_arn: format!("${{{}.iamInstanceProfileArn}}", cluster_name),
            subnet_id: format!("${{{}.privateSubnetId}}", cluster_name),
            cluster_type: ClusterType::Eks,
            endpoint: Some(format!("${{{}.endpoint}}", cluster_name)),
            certificate: Some(format!("${{{}.certificate}}", cluster_name)),
            cluster_dns_ip: Some(format!("${{{}.clusterDnsIp}}", cluster_name)),
            security_groups: vec![],
            assume_role: self.config.assume_role.clone(),
        }
        .into_map()
        .context("Unable to create ec2 config")?;

        // TODO - we have change the raw map to reference/template a non string field.
        ec2_config.insert(
            "securityGroups".to_owned(),
            Value::String(format!("${{{}.securityGroups}}", cluster_name)),
        );

        let ec2_resource = Resource {
            metadata: ObjectMeta {
                name: Some(format!("{}-instances", cluster_name)),
                namespace: Some(NAMESPACE.into()),
                labels: Some(self.labels()),
                ..Default::default()
            },
            spec: ResourceSpec {
                depends_on: Some(vec![cluster_name.to_string()]),
                conflicts_with: None,
                agent: Agent {
                    name: "ec2-provider".to_string(),
                    image: testsys_images
                        .ec2_resource_agent_image
                        .to_owned()
                        .expect("Missing default image for EC2 resource agent"),
                    pull_secret: testsys_images.testsys_agent_pull_secret.clone(),
                    keep_running: false,
                    timeout: None,
                    configuration: Some(ec2_config),
                    secrets: Some(self.config.secrets.clone()),
                    capabilities: None,
                },
                destruction_policy: DestructionPolicy::OnDeletion,
            },
            status: None,
        };
        Ok(Crd::Resource(ec2_resource))
    }

    fn sonobuoy_crd(
        &self,
        test_name_suffix: &str,
        cluster_name: &str,
        sonobuoy_mode: SonobuoyMode,
        depends_on: Option<Vec<String>>,
        testsys_images: &TestsysImages,
    ) -> Result<Crd> {
        let ec2_resource_name = format!("{}-instances", cluster_name);
        let test_name = format!("{}{}", cluster_name, test_name_suffix);
        let sonobuoy = Test {
            metadata: ObjectMeta {
                name: Some(test_name),
                namespace: Some(NAMESPACE.into()),
                labels: Some(self.labels()),
                ..Default::default()
            },
            spec: TestSpec {
                resources: vec![ec2_resource_name, cluster_name.to_string()],
                depends_on,
                retries: Some(5),
                agent: Agent {
                    name: "sonobuoy-test-agent".to_string(),
                    image: testsys_images
                        .sonobuoy_test_agent_image
                        .to_owned()
                        .expect("Missing default image for Sonobuoy test agent"),
                    pull_secret: testsys_images.testsys_agent_pull_secret.clone(),
                    keep_running: true,
                    timeout: None,
                    configuration: Some(
                        SonobuoyConfig {
                            kubeconfig_base64: format!("${{{}.encodedKubeconfig}}", cluster_name),
                            plugin: "e2e".to_string(),
                            mode: sonobuoy_mode,
                            kubernetes_version: None,
                            kube_conformance_image: self.config.kube_conformance_image.clone(),
                            e2e_repo_config_base64: self.config.e2e_repo_registry.as_ref().map(
                                |e2e_registry| {
                                    base64::encode(format!(
                                        r#"buildImageRegistry: {e2e_registry}
dockerGluster: {e2e_registry}
dockerLibraryRegistry: {e2e_registry}
e2eRegistry: {e2e_registry}
e2eVolumeRegistry: {e2e_registry}
gcRegistry: {e2e_registry}
gcEtcdRegistry: {e2e_registry}
promoterE2eRegistry: {e2e_registry}
sigStorageRegistry: {e2e_registry}"#
                                    ))
                                },
                            ),
                            assume_role: self.config.assume_role.clone(),
                        }
                        .into_map()
                        .context("Unable to convert sonobuoy config to `Map`")?,
                    ),
                    secrets: Some(self.config.secrets.clone()),
                    capabilities: None,
                },
            },
            status: None,
        };

        Ok(Crd::Test(sonobuoy))
    }
}

/// In order to easily create migration tests for `aws-k8s` variants we need to implement
/// `Migration` for it.
impl Migration for AwsK8s {
    fn migration_config(&self) -> Result<MigrationsConfig> {
        Ok(MigrationsConfig {
            tuf_repo: self
                .tuf_repo
                .to_owned()
                .context("Tuf repo metadata is required for upgrade downgrade testing.")?,
            starting_version: self
                .starting_version
                .to_owned()
                .context("You must provide a starting version for upgrade downgrade testing.")?,
            migrate_to_version: self
                .migrate_to_version
                .to_owned()
                .context("You must provide a target version for upgrade downgrade testing.")?,
            region: self.region.to_string(),
            secrets: Some(self.config.secrets.clone()),
            capabilities: self.capabilities.clone(),
            assume_role: self.config.assume_role.clone(),
        })
    }

    fn migration_labels(&self) -> BTreeMap<String, String> {
        btreemap! {
            "testsys/arch".to_string() => self.arch.to_string(),
            "testsys/variant".to_string() => self.variant.to_string(),
            "testsys/flavor".to_string() => "updown".to_string(),
        }
    }
}

/// All information required to test ECS variants of Bottlerocket are captured in the `AwsEcs`
/// struct for migration testing, either `starting_version` and `migration_starting_commit`, or
/// `starting_image_id` must be set. TestSys supports `quick` and `migration` testing on ECS
/// variants.
pub(crate) struct AwsEcs {
    /// The architecture to test (`x86_64`,`aarch64')
    pub(crate) arch: String,
    /// The variant to test (`aws-ecs-1`)
    pub(crate) variant: String,
    /// The region testing should be performed in
    pub(crate) region: String,
    /// Configuration for the variant
    pub(crate) config: AwsEcsVariantConfig,
    /// The ami that should be used for quick testing
    pub(crate) ami: String,

    // Migrations
    /// The TUF repos for migration testing. If no TUF repos are used, the default Bottlerocket
    /// repos will be used
    pub(crate) tuf_repo: Option<TufRepoConfig>,
    /// The starting version for migration testing
    pub(crate) starting_version: Option<String>,
    /// The AMI id of the starting version for migration testing
    pub(crate) starting_image_id: Option<String>,
    /// The short commit SHA of the starting version
    pub(crate) migrate_starting_commit: Option<String>,
    /// The target version for Bottlerocket migrations
    pub(crate) migrate_to_version: Option<String>,
    /// Additional capabilities that need to be enabled on the agent's pods
    pub(crate) capabilities: Option<Vec<String>>,
}

impl AwsEcs {
    /// Create the necessary test and resource crds for the specified test type.
    pub(crate) async fn create_crds(
        &self,
        test: TestType,
        testsys_images: &TestsysImages,
    ) -> Result<Vec<Crd>> {
        let mut crds = Vec::new();
        let target_cluster_names = if self.config.cluster_names.is_empty() {
            debug!("No cluster names were provided using default name");
            vec![self.default_cluster_name()]
        } else {
            self.config.cluster_names.clone()
        };
        for template_cluster_name in target_cluster_names {
            let cluster_name = &rendered_cluster_name(
                template_cluster_name,
                self.kube_arch(),
                self.kube_variant(),
            )?;
            crds.append(&mut match test {
                TestType::Conformance => {
                    return Err(anyhow!(
                        "Conformance testing for ECS variants is not supported."
                    ))
                }
                TestType::Quick => self.ecs_test_crds(cluster_name, testsys_images)?,
                TestType::Migration => {
                    self.migration_test_crds(cluster_name, testsys_images)
                        .await?
                }
            });
        }

        Ok(crds)
    }

    fn ecs_test_crds(
        &self,
        cluster_name: &str,
        testsys_images: &TestsysImages,
    ) -> Result<Vec<Crd>> {
        let crds = vec![
            self.ecs_crd(cluster_name, testsys_images)?,
            self.ec2_crd(cluster_name, testsys_images, None)?,
            self.ecs_test_crd(cluster_name, "-test", None, testsys_images)?,
        ];
        Ok(crds)
    }

    async fn migration_test_crds(
        &self,
        cluster_name: &str,
        testsys_images: &TestsysImages,
    ) -> Result<Vec<Crd>> {
        let ami = self
            .starting_image_id
            .as_ref()
            .unwrap_or(
                &get_ami_id(
                    format!(
                        "bottlerocket-{}-{}-{}-{}",
                        self.variant,
                        self.arch,
                        self.starting_version.as_ref().context("The starting version must be provided for migration testing")?, 
                        self.migrate_starting_commit.as_ref().context("The commit for the starting version must be provided if the starting image id is not")?
                    ), & self.arch,
                    self.region.to_string(),
                )
                .await?,
            )
            .to_string();
        let ecs = self.ecs_crd(cluster_name, testsys_images)?;
        let ec2 = self.ec2_crd(cluster_name, testsys_images, Some(ami))?;
        let instance_provider = ec2
            .name()
            .expect("The EC2 instance provider crd is missing a name.");
        let mut depends_on = Vec::new();
        let initial = self.ecs_test_crd(cluster_name, "-1-initial", None, testsys_images)?;
        depends_on.push(initial.name().context("Crd missing name")?);
        let start_migrate = self.migration_crd(
            format!("{}-2-migrate", cluster_name),
            instance_provider.clone(),
            MigrationVersion::Migrated,
            Some(depends_on.clone()),
            testsys_images,
        )?;
        depends_on.push(start_migrate.name().context("Crd missing name")?);
        let migrated = self.ecs_test_crd(
            cluster_name,
            "-3-migrated",
            Some(depends_on.clone()),
            testsys_images,
        )?;
        depends_on.push(migrated.name().context("Crd missing name")?);
        let end_migrate = self.migration_crd(
            format!("{}-4-migrate", cluster_name),
            instance_provider,
            MigrationVersion::Starting,
            Some(depends_on.clone()),
            testsys_images,
        )?;
        depends_on.push(end_migrate.name().context("Crd missing name")?);
        let last = self.ecs_test_crd(
            cluster_name,
            "-5-final",
            Some(depends_on.clone()),
            testsys_images,
        )?;
        Ok(vec![
            ecs,
            ec2,
            initial,
            start_migrate,
            migrated,
            end_migrate,
            last,
        ])
    }

    /// Labels help filter test results with `testsys status`.
    fn labels(&self) -> BTreeMap<String, String> {
        btreemap! {
            "testsys/arch".to_string() => self.arch.to_string(),
            "testsys/variant".to_string() => self.variant.to_string(),
        }
    }

    fn kube_arch(&self) -> String {
        self.arch.replace('_', "-")
    }

    fn kube_variant(&self) -> String {
        self.variant.replace('.', "")
    }

    /// Bottlerocket cluster naming convention.
    fn default_cluster_name(&self) -> String {
        format!("{}-{}", self.kube_arch(), self.kube_variant())
    }

    fn ecs_crd(&self, cluster_name: &str, testsys_images: &TestsysImages) -> Result<Crd> {
        let ecs_crd = Resource {
            metadata: ObjectMeta {
                name: Some(cluster_name.to_string()),
                namespace: Some(NAMESPACE.into()),
                labels: Some(self.labels()),
                ..Default::default()
            },
            spec: ResourceSpec {
                depends_on: None,
                conflicts_with: None,
                agent: Agent {
                    name: "ecs-provider".to_string(),
                    image: testsys_images
                        .ecs_resource_agent_image
                        .to_owned()
                        .expect("Missing default image for ECS resource agent"),
                    pull_secret: testsys_images.testsys_agent_pull_secret.clone(),
                    keep_running: false,
                    timeout: None,
                    configuration: Some(
                        EcsClusterConfig {
                            cluster_name: cluster_name.to_string(),
                            region: Some(self.region.clone()),
                            assume_role: self.config.assume_role.clone(),
                            vpc: None,
                        }
                        .into_map()
                        .context("Unable to convert ECS config to map")?,
                    ),
                    secrets: Some(self.config.secrets.clone()),
                    capabilities: None,
                },
                destruction_policy: DestructionPolicy::Never,
            },
            status: None,
        };
        Ok(Crd::Resource(ecs_crd))
    }

    fn ec2_crd(
        &self,
        cluster_name: &str,
        testsys_images: &TestsysImages,
        override_ami: Option<String>,
    ) -> Result<Crd> {
        let ec2_config = Ec2Config {
            node_ami: override_ami.unwrap_or_else(|| self.ami.clone()),
            instance_count: Some(2),
            instance_type: self.config.instance_type.clone(),
            cluster_name: format!("${{{}.clusterName}}", cluster_name),
            region: format!("${{{}.region}}", cluster_name),
            instance_profile_arn: format!("${{{}.iamInstanceProfileArn}}", cluster_name),
            subnet_id: format!("${{{}.publicSubnetId}}", cluster_name),
            cluster_type: ClusterType::Ecs,
            endpoint: None,
            certificate: None,
            cluster_dns_ip: None,
            security_groups: vec![],
            assume_role: self.config.assume_role.clone(),
        }
        .into_map()
        .context("Unable to create EC2 config")?;

        let ec2_resource = Resource {
            metadata: ObjectMeta {
                name: Some(format!("{}-instances", cluster_name)),
                namespace: Some(NAMESPACE.into()),
                labels: Some(self.labels()),
                ..Default::default()
            },
            spec: ResourceSpec {
                depends_on: Some(vec![cluster_name.to_string()]),
                conflicts_with: None,
                agent: Agent {
                    name: "ec2-provider".to_string(),
                    image: testsys_images
                        .ec2_resource_agent_image
                        .to_owned()
                        .expect("Missing default image for EC2 resource agent"),
                    pull_secret: testsys_images.testsys_agent_pull_secret.clone(),
                    keep_running: false,
                    timeout: None,
                    configuration: Some(ec2_config),
                    secrets: Some(self.config.secrets.clone()),
                    capabilities: None,
                },
                destruction_policy: DestructionPolicy::OnDeletion,
            },
            status: None,
        };
        Ok(Crd::Resource(ec2_resource))
    }

    fn ecs_test_crd(
        &self,
        cluster_name: &str,
        test_name_suffix: &str,
        depends_on: Option<Vec<String>>,
        testsys_images: &TestsysImages,
    ) -> Result<Crd> {
        let ec2_resource_name = format!("{}-instances", cluster_name);
        let test_name = format!("{}{}", cluster_name, test_name_suffix);
        let ecs_test = Test {
            metadata: ObjectMeta {
                name: Some(test_name),
                namespace: Some(NAMESPACE.into()),
                labels: Some(self.labels()),
                ..Default::default()
            },
            spec: TestSpec {
                resources: vec![ec2_resource_name, cluster_name.to_string()],
                depends_on,
                retries: Some(5),
                agent: Agent {
                    name: "ecs-test-agent".to_string(),
                    image: testsys_images
                        .ecs_test_agent_image
                        .to_owned()
                        .expect("Missing default image for ECS test agent"),
                    pull_secret: testsys_images.testsys_agent_pull_secret.clone(),
                    keep_running: true,
                    timeout: None,
                    configuration: Some(
                        EcsTestConfig {
                            assume_role: self.config.assume_role.clone(),
                            region: Some(self.region.clone()),
                            cluster_name: cluster_name.to_string(),
                            task_count: 1,
                            subnet: format!("${{{}.publicSubnetId}}", cluster_name),
                            task_definition_name_and_revision: None,
                        }
                        .into_map()
                        .context("Unable to convert sonobuoy config to `Map`")?,
                    ),
                    secrets: Some(self.config.secrets.clone()),
                    capabilities: None,
                },
            },
            status: None,
        };

        Ok(Crd::Test(ecs_test))
    }
}

/// In order to easily create migration tests for `aws-ecs` variants we need to implement
/// `Migration` for it.
impl Migration for AwsEcs {
    fn migration_config(&self) -> Result<MigrationsConfig> {
        Ok(MigrationsConfig {
            tuf_repo: self
                .tuf_repo
                .to_owned()
                .context("Tuf repo metadata is required for upgrade downgrade testing.")?,
            starting_version: self
                .starting_version
                .to_owned()
                .context("You must provide a starting version for upgrade downgrade testing.")?,
            migrate_to_version: self
                .migrate_to_version
                .to_owned()
                .context("You must provide a target version for upgrade downgrade testing.")?,
            region: self.region.to_string(),
            secrets: Some(self.config.secrets.clone()),
            capabilities: self.capabilities.clone(),
            assume_role: self.config.assume_role.clone(),
        })
    }

    fn migration_labels(&self) -> BTreeMap<String, String> {
        btreemap! {
            "testsys/arch".to_string() => self.arch.to_string(),
            "testsys/variant".to_string() => self.variant.to_string(),
            "testsys/flavor".to_string() => "updown".to_string(),
        }
    }
}

/// An enum to differentiate between upgrade and downgrade tests.
enum MigrationVersion {
    ///`MigrationVersion::Starting` will create a migration to the starting version.
    Starting,
    ///`MigrationVersion::Migrated` will create a migration to the target version.
    Migrated,
}

/// A configuration containing all information needed to create a migration test for a given
/// variant.
struct MigrationsConfig {
    tuf_repo: TufRepoConfig,
    starting_version: String,
    migrate_to_version: String,
    region: String,
    secrets: Option<BTreeMap<String, SecretName>>,
    capabilities: Option<Vec<String>>,
    assume_role: Option<String>,
}

/// Migration is a trait that should be implemented for all traits that use upgrade/downgrade
/// testing. It provides the infrastructure to easily create migration tests.
trait Migration {
    /// Create a migration config that is used to create migration tests.
    fn migration_config(&self) -> Result<MigrationsConfig>;

    /// Create the labels that should be used for the migration tests.
    fn migration_labels(&self) -> BTreeMap<String, String>;

    /// Create a migration test for a given arch/variant.
    fn migration_crd(
        &self,
        test_name: String,
        instance_provider: String,
        migration_version: MigrationVersion,
        depends_on: Option<Vec<String>>,
        testsys_images: &TestsysImages,
    ) -> Result<Crd> {
        // Get the migration configuration for the given type.
        let migration = self.migration_config()?;

        // Determine which version we are migrating to.
        let version = match migration_version {
            MigrationVersion::Starting => migration.starting_version,
            MigrationVersion::Migrated => migration.migrate_to_version,
        };

        // Create the migration test crd.
        let mut migration_config = MigrationConfig {
            aws_region: migration.region,
            instance_ids: Default::default(),
            migrate_to_version: version,
            tuf_repo: Some(migration.tuf_repo.clone()),
            assume_role: migration.assume_role.clone(),
        }
        .into_map()
        .context("Unable to convert migration config to map")?;
        migration_config.insert(
            "instanceIds".to_string(),
            Value::String(format!("${{{}.ids}}", instance_provider)),
        );
        Ok(Crd::Test(Test {
            metadata: ObjectMeta {
                name: Some(test_name),
                namespace: Some(NAMESPACE.into()),
                labels: Some(self.migration_labels()),
                ..Default::default()
            },
            spec: TestSpec {
                resources: vec![instance_provider],
                depends_on,
                retries: None,
                agent: Agent {
                    name: "migration-test-agent".to_string(),
                    image: testsys_images
                        .migration_test_agent_image
                        .to_owned()
                        .expect("Missing default image for migration test agent"),
                    pull_secret: testsys_images.testsys_agent_pull_secret.clone(),
                    keep_running: true,
                    timeout: None,
                    configuration: Some(migration_config),
                    secrets: migration.secrets.clone(),
                    capabilities: migration.capabilities,
                },
            },
            status: None,
        }))
    }
}

/// Queries EC2 for the given AMI name. If found, returns Ok(Some(id)), if not returns Ok(None).
pub(crate) async fn get_ami_id<S1, S2, S3>(name: S1, arch: S2, region: S3) -> Result<String>
where
    S1: Into<String>,
    S2: Into<String>,
    S3: Into<String>,
{
    let config = aws_config::from_env()
        .region(Region::new(region.into()))
        .load()
        .await;
    let ec2_client = aws_sdk_ec2::Client::new(&config);
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
    if images.len() > 1 {
        return Err(anyhow!("Multiple images were found"));
    };
    if let Some(image) = images.last().as_ref() {
        Ok(image.image_id().context("No image id for AMI")?.to_string())
    } else {
        Err(anyhow!("No images were found"))
    }
}
