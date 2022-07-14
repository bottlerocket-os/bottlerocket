use crate::run::{TestType, TestsysImages};
use anyhow::{anyhow, Context, Result};
use bottlerocket_types::agent_config::{
    ClusterType, CreationPolicy, Ec2Config, EksClusterConfig, K8sVersion, SonobuoyConfig,
    SonobuoyMode,
};

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
}

impl AwsK8s {
    /// Create the necessary test and resource crds for the specified test type.
    pub(crate) fn create_crds(
        &self,
        test: TestType,
        testsys_images: &TestsysImages,
    ) -> Result<Vec<Crd>> {
        match test {
            TestType::Conformance => {
                self.sonobuoy_test_crds(testsys_images, SonobuoyMode::CertifiedConformance)
            }
            TestType::Quick => self.sonobuoy_test_crds(testsys_images, SonobuoyMode::Quick),
        }
    }

    fn sonobuoy_test_crds(
        &self,
        testsys_images: &TestsysImages,
        sonobuoy_mode: SonobuoyMode,
    ) -> Result<Vec<Crd>> {
        let crds = vec![
            self.eks_crd("", testsys_images)?,
            self.ec2_crd("", testsys_images)?,
            self.sonobuoy_crd("", "-test", sonobuoy_mode, None, testsys_images)?,
        ];
        Ok(crds)
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
    fn cluster_name(&self, suffix: &str) -> String {
        self.target_cluster_name
            .clone()
            .unwrap_or_else(|| format!("{}-{}{}", self.kube_arch(), self.kube_variant(), suffix))
    }

    fn eks_crd(&self, cluster_suffix: &str, testsys_images: &TestsysImages) -> Result<Crd> {
        let cluster_version = K8sVersion::parse(
            Variant::new(&self.variant)
                .context("The provided variant cannot be interpreted.")?
                .version()
                .context("aws-k8s variant is missing k8s version")?,
        )
        .map_err(|e| anyhow!(e))?;
        let cluster_name = self.cluster_name(cluster_suffix);
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

    fn ec2_crd(&self, cluster_suffix: &str, testsys_images: &TestsysImages) -> Result<Crd> {
        let cluster_name = self.cluster_name(cluster_suffix);
        let mut ec2_config = Ec2Config {
            node_ami: self.ami.clone(),
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
        cluster_suffix: &str,
        test_name_suffix: &str,
        sonobuoy_mode: SonobuoyMode,
        depends_on: Option<Vec<String>>,
        testsys_images: &TestsysImages,
    ) -> Result<Crd> {
        let cluster_name = self.cluster_name(cluster_suffix);
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
