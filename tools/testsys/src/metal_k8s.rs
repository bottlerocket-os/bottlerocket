use crate::crds::{
    BottlerocketInput, ClusterInput, CrdCreator, CrdInput, CreateCrdOutput, MigrationInput,
    TestInput,
};
use crate::error::{self, Result};
use crate::migration::migration_crd;
use crate::sonobuoy::{sonobuoy_crd, workload_crd};
use bottlerocket_types::agent_config::MetalK8sClusterConfig;
use maplit::btreemap;
use serde::Deserialize;
use snafu::{OptionExt, ResultExt};
use std::collections::BTreeMap;
use testsys_model::{Crd, DestructionPolicy};
use url::Url;

/// A `CrdCreator` responsible for creating crd related to `metal-k8s` variants.
pub(crate) struct MetalK8sCreator {
    pub(crate) region: String,
    pub(crate) encoded_mgmt_cluster_kubeconfig: String,
    pub(crate) image_name: String,
}

#[async_trait::async_trait]
impl CrdCreator for MetalK8sCreator {
    /// Use the provided image name with the `os_image_dir` from `Test.toml` for the image id.
    async fn image_id(&self, crd_input: &CrdInput) -> Result<String> {
        image_url(
            crd_input
                .config
                .os_image_dir
                .as_ref()
                .context(error::InvalidSnafu {
                    what: "An os image directory is required for metal testing",
                })?,
            &self.image_name,
        )
    }

    /// Use standard naming conventions to predict the starting image name.
    async fn starting_image_id(&self, crd_input: &CrdInput) -> Result<String> {
        let filename = format!(
            "bottlerocket-{}-{}-{}.img.gz",
            crd_input.variant,
            crd_input.arch,
            crd_input
                .starting_version
                .as_ref()
                .context(error::InvalidSnafu {
                    what: "The starting version must be provided for migration testing"
                })?
        );
        image_url(crd_input.config.os_image_dir.as_ref().context(error::InvalidSnafu {
            what: "An os image directory is required for metal testing if a starting image id not used",
        })?, &filename)
    }

    /// Creates a metal K8s cluster CRD with the `cluster_name` in `cluster_input`.
    async fn cluster_crd<'a>(&self, cluster_input: ClusterInput<'a>) -> Result<CreateCrdOutput> {
        let (cluster_name, control_plane_endpoint_ip, k8s_version) = cluster_data(
            cluster_input
                .cluster_config
                .as_ref()
                .context(error::InvalidSnafu {
                    what: "A cluster config is required for Bare Metal cluster provisioning.",
                })?,
        )?;

        let labels = cluster_input.crd_input.labels(btreemap! {
            "testsys/type".to_string() => "cluster".to_string(),
            "testsys/cluster".to_string() => cluster_name.clone(),
            "testsys/controlPlaneEndpoint".to_string() => control_plane_endpoint_ip,
            "testsys/k8sVersion".to_string() => k8s_version
        });

        // Check if the cluster already has a CRD
        if let Some(cluster_crd) = cluster_input
            .crd_input
            .existing_crds(
                &labels,
                &[
                    "testsys/cluster",
                    "testsys/type",
                    "testsys/controlPlaneEndpoint",
                    "testsys/k8sVersion",
                ],
            )
            .await?
            .pop()
        {
            return Ok(CreateCrdOutput::ExistingCrd(cluster_crd));
        }

        // Check if an existing cluster is using this endpoint
        let existing_clusters = cluster_input
            .crd_input
            .existing_crds(&labels, &["testsys/type", "testsys/controlPlaneEndpoint"])
            .await?;

        let metal_k8s_crd = MetalK8sClusterConfig::builder()
            .set_labels(Some(labels))
            .mgmt_cluster_kubeconfig_base64(&self.encoded_mgmt_cluster_kubeconfig)
            .hardware_csv_base64(base64::encode(
                cluster_input
                    .hardware_csv
                    .as_ref()
                    .context(error::InvalidSnafu {
                        what: "A hardware CSV is required for Bare Metal testing",
                    })?,
            ))
            .cluster_config_base64(base64::encode(
                cluster_input
                    .cluster_config
                    .as_ref()
                    .context(error::InvalidSnafu {
                        what: "A cluster config is required for Bare Metal testing",
                    })?,
            ))
            .set_conflicts_with(Some(existing_clusters))
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
                    .metal_k8s_cluster_resource_agent_image
                    .as_ref()
                    .expect(
                        "The default metal K8s cluster resource provider image URI is missing.",
                    ),
            )
            .set_image_pull_secret(
                cluster_input
                    .crd_input
                    .images
                    .testsys_agent_pull_secret
                    .to_owned(),
            )
            .privileged(true)
            .build(cluster_name)
            .context(error::BuildSnafu {
                what: "metal K8s cluster CRD",
            })?;

        Ok(CreateCrdOutput::NewCrd(Box::new(Crd::Resource(
            metal_k8s_crd,
        ))))
    }

    /// Machines are provisioned during cluster creation, so there is nothing to do here.
    async fn bottlerocket_crd<'a>(
        &self,
        _bottlerocket_input: BottlerocketInput<'a>,
    ) -> Result<CreateCrdOutput> {
        Ok(CreateCrdOutput::None)
    }

    async fn migration_crd<'a>(
        &self,
        migration_input: MigrationInput<'a>,
    ) -> Result<CreateCrdOutput> {
        Ok(CreateCrdOutput::NewCrd(Box::new(Crd::Test(migration_crd(
            migration_input,
            Some("us-west-2".to_string()),
            "instanceIds",
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

/// Determine the (cluster name, control plane endpoint ip, K8s version) from an EKS Anywhere cluster manifest
fn cluster_data(config: &str) -> Result<(String, String, String)> {
    let cluster_manifest = serde_yaml::Deserializer::from_str(config)
        .map(|config| {
            serde_yaml::Value::deserialize(config).context(error::SerdeYamlSnafu {
                what: "Unable to deserialize cluster config",
            })
        })
        // Make sure all of the configs were deserializable
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        // Find the `Cluster` config
        .find(|config| {
            config.get("kind") == Some(&serde_yaml::Value::String("Cluster".to_string()))
        });
    let cluster_name = cluster_manifest
        .as_ref()
        // Get the name from the metadata field in the `Cluster` config
        .and_then(|config| config.get("metadata"))
        .and_then(|config| config.get("name"))
        .and_then(|name| name.as_str())
        .context(error::MissingSnafu {
            item: "name",
            what: "EKS Anywhere config metadata",
        })?
        .to_string();

    let control_plane_endpoint_ip = cluster_manifest
        .as_ref()
        // Get the name from the metadata field in the `Cluster` config
        .and_then(|config| config.get("spec"))
        .and_then(|config| config.get("controlPlaneConfiguration"))
        .and_then(|config| config.get("endpoint"))
        .and_then(|config| config.get("host"))
        .and_then(|name| name.as_str())
        .context(error::MissingSnafu {
            item: "control plane endpoint",
            what: "EKS Anywhere config metadata",
        })?
        .to_string();

    let k8s_version = cluster_manifest
        .as_ref()
        // Get the name from the metadata field in the `Cluster` config
        .and_then(|config| config.get("spec"))
        .and_then(|config| config.get("kubernetesVersion"))
        .and_then(|name| name.as_str())
        .context(error::MissingSnafu {
            item: "control plane endpoint",
            what: "EKS Anywhere config metadata",
        })?
        .to_string();

    Ok((cluster_name, control_plane_endpoint_ip, k8s_version))
}

fn image_url(image_dir: &str, filename: &str) -> Result<String> {
    let image_url = Url::parse(image_dir)
        .and_then(|base_url| base_url.join(filename))
        .context(error::UrlParseSnafu { url: image_dir })?;
    Ok(image_url.to_string())
}
