use crate::run::{TestType, TestsysImages};
use anyhow::{anyhow, Context, Result};
use bottlerocket_types::agent_config::{
    ClusterType, CreationPolicy, Ec2Config, EksClusterConfig, K8sVersion, MigrationConfig,
    SonobuoyConfig, SonobuoyMode, TufRepoConfig,
};

use aws_sdk_ec2::model::{Filter, Image};
use aws_sdk_ec2::Region;
use bottlerocket_variant::Variant;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use k8s_openapi::serde_json::Value;
use maplit::btreemap;
use model::constants::NAMESPACE;
use model::{
    Agent, Configuration, Crd, DestructionPolicy, Resource, ResourceSpec, SecretName, Test,
    TestSpec,
};
use std::collections::BTreeMap;
use std::convert::identity;

pub(crate) struct AwsK8s {
    pub(crate) arch: String,
    pub(crate) variant: String,
    pub(crate) region: String,
    pub(crate) assume_role: Option<String>,
    pub(crate) instance_type: Option<String>,
    pub(crate) ami: String,
    pub(crate) secrets: Option<BTreeMap<String, SecretName>>,
    pub(crate) kube_conformance_image: Option<String>,
    pub(crate) target_cluster_name: Option<String>,
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
        match test {
            TestType::Conformance => {
                self.sonobuoy_test_crds(testsys_images, SonobuoyMode::CertifiedConformance)
            }
            TestType::Quick => self.sonobuoy_test_crds(testsys_images, SonobuoyMode::Quick),
            TestType::Migration => self.migration_test_crds(testsys_images).await,
        }
    }

    fn sonobuoy_test_crds(
        &self,
        testsys_images: &TestsysImages,
        sonobuoy_mode: SonobuoyMode,
    ) -> Result<Vec<Crd>> {
        let crds = vec![
            self.eks_crd(testsys_images)?,
            self.ec2_crd(testsys_images, None)?,
            self.sonobuoy_crd("-test", sonobuoy_mode, None, testsys_images)?,
        ];
        Ok(crds)
    }

    /// Creates `Test` crds for migration testing.
    async fn migration_test_crds(&self, testsys_images: &TestsysImages) -> Result<Vec<Crd>> {
        let ami = self
            .starting_image_id
            .as_ref()
            .unwrap_or(
                &get_ami_id(
                    format!(
                        "bottlerocket-{}-{}-{}-{}",
                        self.variant, self.arch, self.starting_version.as_ref().context("The starting version must be provided for migration testing")?, self.migrate_starting_commit.as_ref().context("The commit for the starting version must be provided if the starting image id is not")?
                    ), & self.arch,
                    self.region.to_string(),
                )
                .await?,
            )
            .to_string();
        let eks = self.eks_crd(testsys_images)?;
        let ec2 = self.ec2_crd(testsys_images, Some(ami))?;
        let mut depends_on = Vec::new();
        // Start with a `quick` test to make sure instances launched properly
        let initial = self.sonobuoy_crd("-1-initial", SonobuoyMode::Quick, None, testsys_images)?;
        depends_on.push(initial.name().context("Crd missing name")?);
        // Migrate instances to the target version
        let start_migrate = self.migration_crd(
            format!("{}-2-migrate", self.cluster_name()),
            MigrationVersion::Migrated,
            Some(depends_on.clone()),
            testsys_images,
        )?;
        // A `quick` test to validate the migration
        depends_on.push(start_migrate.name().context("Crd missing name")?);
        let migrated = self.sonobuoy_crd(
            "-3-migrated",
            SonobuoyMode::Quick,
            Some(depends_on.clone()),
            testsys_images,
        )?;
        // Migrate instances to the starting version
        depends_on.push(migrated.name().context("Crd missing name")?);
        let end_migrate = self.migration_crd(
            format!("{}-4-migrate", self.cluster_name()),
            MigrationVersion::Starting,
            Some(depends_on.clone()),
            testsys_images,
        )?;
        // A final quick test to validate the migration back to the starting version
        depends_on.push(end_migrate.name().context("Crd missing name")?);
        let last = self.sonobuoy_crd(
            "-5-final",
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
    fn cluster_name(&self) -> String {
        self.target_cluster_name
            .clone()
            .unwrap_or_else(|| format!("{}-{}", self.kube_arch(), self.kube_variant()))
    }

    fn eks_crd(&self, testsys_images: &TestsysImages) -> Result<Crd> {
        let cluster_version = K8sVersion::parse(
            Variant::new(&self.variant)
                .context("The provided variant cannot be interpreted.")?
                .version()
                .context("aws-k8s variant is missing k8s version")?,
        )
        .map_err(|e| anyhow!(e))?;
        let cluster_name = self.cluster_name();
        let eks_crd = Resource {
            metadata: ObjectMeta {
                name: Some(cluster_name.clone()),
                namespace: Some(NAMESPACE.into()),
                labels: Some(self.labels()),
                ..Default::default()
            },
            spec: ResourceSpec {
                depends_on: None,
                agent: Agent {
                    name: "eks-provider".to_string(),
                    image: testsys_images.eks_resource.clone(),
                    pull_secret: testsys_images.secret.clone(),
                    keep_running: false,
                    timeout: None,
                    configuration: Some(
                        EksClusterConfig {
                            cluster_name,
                            creation_policy: Some(CreationPolicy::IfNotExists),
                            region: Some(self.region.clone()),
                            zones: None,
                            version: Some(cluster_version),
                            assume_role: self.assume_role.clone(),
                        }
                        .into_map()
                        .context("Unable to convert eks config to map")?,
                    ),
                    secrets: self.secrets.clone(),
                    capabilities: None,
                },
                destruction_policy: DestructionPolicy::Never,
            },
            status: None,
        };
        Ok(Crd::Resource(eks_crd))
    }

    fn ec2_crd(&self, testsys_images: &TestsysImages, override_ami: Option<String>) -> Result<Crd> {
        let cluster_name = self.cluster_name();
        let mut ec2_config = Ec2Config {
            node_ami: override_ami.unwrap_or_else(|| self.ami.clone()),
            instance_count: Some(2),
            instance_type: self.instance_type.clone(),
            cluster_name: format!("${{{}.clusterName}}", cluster_name),
            region: format!("${{{}.region}}", cluster_name),
            instance_profile_arn: format!("${{{}.iamInstanceProfileArn}}", cluster_name),
            subnet_id: format!("${{{}.privateSubnetId}}", cluster_name),
            cluster_type: ClusterType::Eks,
            endpoint: Some(format!("${{{}.endpoint}}", cluster_name)),
            certificate: Some(format!("${{{}.certificate}}", cluster_name)),
            cluster_dns_ip: Some(format!("${{{}.clusterDnsIp}}", cluster_name)),
            security_groups: vec![],
            assume_role: self.assume_role.clone(),
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
                depends_on: Some(vec![cluster_name]),
                agent: Agent {
                    name: "ec2-provider".to_string(),
                    image: testsys_images.ec2_resource.clone(),
                    pull_secret: testsys_images.secret.clone(),
                    keep_running: false,
                    timeout: None,
                    configuration: Some(ec2_config),
                    secrets: self.secrets.clone(),
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
        sonobuoy_mode: SonobuoyMode,
        depends_on: Option<Vec<String>>,
        testsys_images: &TestsysImages,
    ) -> Result<Crd> {
        let cluster_name = self.cluster_name();
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
                    image: testsys_images.sonobuoy_test.clone(),
                    pull_secret: testsys_images.secret.clone(),
                    keep_running: true,
                    timeout: None,
                    configuration: Some(
                        SonobuoyConfig {
                            kubeconfig_base64: format!("${{{}.encodedKubeconfig}}", cluster_name),
                            plugin: "e2e".to_string(),
                            mode: sonobuoy_mode,
                            kubernetes_version: None,
                            kube_conformance_image: self.kube_conformance_image.clone(),
                            assume_role: self.assume_role.clone(),
                        }
                        .into_map()
                        .context("Unable to convert sonobuoy config to `Map`")?,
                    ),
                    secrets: self.secrets.clone(),
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
                .as_ref()
                .context("Tuf repo metadata is required for upgrade downgrade testing.")?
                .clone(),
            starting_version: self
                .starting_version
                .as_ref()
                .context("You must provide a starting version for upgrade downgrade testing.")?
                .clone(),
            migrate_to_version: self
                .migrate_to_version
                .as_ref()
                .context("You must provide a target version for upgrade downgrade testing.")?
                .clone(),
            region: self.region.to_string(),
            secrets: self.secrets.clone(),
            capabilities: self.capabilities.clone(),
            assume_role: self.assume_role.clone(),
        })
    }

    fn instance_provider(&self) -> String {
        let cluster_name = self.cluster_name();
        format!("{}-instances", cluster_name)
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

    /// Return the name of the instance provider that the migration agents should use to get the
    /// instance ids.
    fn instance_provider(&self) -> String;

    /// Create a migration test for a given arch/variant.
    fn migration_crd(
        &self,
        test_name: String,
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
            Value::String(format!("${{{}.ids}}", self.instance_provider())),
        );
        Ok(Crd::Test(Test {
            metadata: ObjectMeta {
                name: Some(test_name),
                namespace: Some(NAMESPACE.into()),
                labels: Some(self.migration_labels()),
                ..Default::default()
            },
            spec: TestSpec {
                resources: vec![self.instance_provider()],
                depends_on,
                retries: None,
                agent: Agent {
                    name: "migration-test-agent".to_string(),
                    image: testsys_images.migration_test.to_string(),
                    pull_secret: testsys_images.secret.clone(),
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
    let images: Vec<&Image> = describe_images
        .iter()
        .flat_map(|image| identity(image))
        .collect();
    if images.len() > 1 {
        return Err(anyhow!("Multiple images were found"));
    };
    if let Some(image) = images.last().as_ref() {
        Ok(image.image_id().context("No image id for AMI")?.to_string())
    } else {
        Err(anyhow!("No images were found"))
    }
}
