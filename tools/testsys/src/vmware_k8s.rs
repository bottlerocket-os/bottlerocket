use crate::crds::{
    BottlerocketInput, ClusterInput, CrdCreator, CrdInput, CreateCrdOutput, MigrationInput,
    TestInput,
};
use crate::error::{self, Result};
use crate::migration::migration_crd;
use crate::sonobuoy::{sonobuoy_crd, workload_crd};
use bottlerocket_types::agent_config::{
    CreationPolicy, CustomUserData, K8sVersion, VSphereK8sClusterConfig, VSphereK8sClusterInfo,
    VSphereVmConfig,
};
use maplit::btreemap;
use pubsys_config::vmware::Datacenter;
use snafu::{OptionExt, ResultExt};
use std::collections::BTreeMap;
use std::iter::repeat_with;
use std::str::FromStr;
use testsys_model::{Crd, DestructionPolicy, SecretName};

/// A `CrdCreator` responsible for creating crd related to `vmware-k8s` variants.
pub(crate) struct VmwareK8sCreator {
    pub(crate) region: String,
    pub(crate) datacenter: Datacenter,
    pub(crate) creds: Option<(String, SecretName)>,
    pub(crate) ova_name: String,
    pub(crate) encoded_mgmt_cluster_kubeconfig: String,
}

#[async_trait::async_trait]
impl CrdCreator for VmwareK8sCreator {
    /// Use the provided OVA name for the image id.
    async fn image_id(&self, _: &CrdInput) -> Result<String> {
        Ok(self.ova_name.to_string())
    }

    /// Use standard naming conventions to predict the starting OVA.
    async fn starting_image_id(&self, crd_input: &CrdInput) -> Result<String> {
        Ok(format!(
            "bottlerocket-{}-{}-{}.ova",
            crd_input.variant,
            crd_input.arch,
            crd_input
                .starting_version
                .as_ref()
                .context(error::InvalidSnafu {
                    what: "The starting version must be provided for migration testing"
                })?
        ))
    }

    /// Creates a vSphere K8s cluster CRD with the `cluster_name` in `cluster_input`.
    async fn cluster_crd<'a>(&self, cluster_input: ClusterInput<'a>) -> Result<CreateCrdOutput> {
        let control_plane_endpoint = cluster_input
            .crd_input
            .config
            .control_plane_endpoint
            .as_ref()
            .context(error::InvalidSnafu {
                what: "The control plane endpoint is required for VMware cluster creation.",
            })?;
        let labels = cluster_input.crd_input.labels(btreemap! {
            "testsys/type".to_string() => "cluster".to_string(),
            "testsys/cluster".to_string() => cluster_input.cluster_name.to_string(),
            "testsys/controlPlaneEndpoint".to_string() => control_plane_endpoint.to_string(),
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

        let vsphere_k8s_crd = VSphereK8sClusterConfig::builder()
            .name(cluster_input.cluster_name)
            .set_labels(Some(labels))
            .control_plane_endpoint_ip(control_plane_endpoint)
            .creation_policy(CreationPolicy::IfNotExists)
            .version(cluster_version)
            .ova_name(self.image_id(cluster_input.crd_input).await?)
            .tuf_repo(
                cluster_input
                    .crd_input
                    .tuf_repo_config()
                    .context(error::InvalidSnafu {
                        what: "TUF repo information is required for VMware cluster creation.",
                    })?,
            )
            .vcenter_host_url(&self.datacenter.vsphere_url)
            .vcenter_datacenter(&self.datacenter.datacenter)
            .vcenter_datastore(&self.datacenter.datastore)
            .vcenter_network(&self.datacenter.network)
            .vcenter_resource_pool(&self.datacenter.resource_pool)
            .vcenter_workload_folder(&self.datacenter.folder)
            .mgmt_cluster_kubeconfig_base64(&self.encoded_mgmt_cluster_kubeconfig)
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
                    .vsphere_k8s_cluster_resource_agent_image
                    .as_ref()
                    .expect(
                        "The default vSphere K8s cluster resource provider image URI is missing.",
                    ),
            )
            .set_image_pull_secret(
                cluster_input
                    .crd_input
                    .images
                    .testsys_agent_pull_secret
                    .to_owned(),
            )
            .set_secrets(Some(
                cluster_input
                    .crd_input
                    .config
                    .secrets
                    .clone()
                    .into_iter()
                    .chain(self.creds.clone())
                    .collect(),
            ))
            .privileged(true)
            .build(cluster_input.cluster_name)
            .context(error::BuildSnafu {
                what: "vSphere K8s cluster CRD",
            })?;
        Ok(CreateCrdOutput::NewCrd(Box::new(Crd::Resource(
            vsphere_k8s_crd,
        ))))
    }

    /// Create a vSphere VM provider CRD to launch Bottlerocket VMs on the cluster created by
    /// `cluster_crd`.
    async fn bottlerocket_crd<'a>(
        &self,
        bottlerocket_input: BottlerocketInput<'a>,
    ) -> Result<CreateCrdOutput> {
        let cluster_name = bottlerocket_input
            .cluster_crd_name
            .as_ref()
            .expect("A vSphere K8s cluster provider is required");
        let labels = bottlerocket_input.crd_input.labels(btreemap! {
            "testsys/type".to_string() => "vms".to_string(),
            "testsys/cluster".to_string() => cluster_name.to_string(),
        });

        // Check if other VMs are using this cluster
        let existing_clusters = bottlerocket_input
            .crd_input
            .existing_crds(&labels, &["testsys/type", "testsys/cluster"])
            .await?;

        let suffix: String = repeat_with(fastrand::lowercase).take(4).collect();
        let vsphere_vm_crd = VSphereVmConfig::builder()
            .ova_name(bottlerocket_input.image_id)
            .tuf_repo(bottlerocket_input.crd_input.tuf_repo_config().context(
                error::InvalidSnafu {
                    what: "TUF repo information is required for Bottlerocket vSphere VM creation.",
                },
            )?)
            .vcenter_host_url(&self.datacenter.vsphere_url)
            .vcenter_datacenter(&self.datacenter.datacenter)
            .vcenter_datastore(&self.datacenter.datastore)
            .vcenter_network(&self.datacenter.network)
            .vcenter_resource_pool(&self.datacenter.resource_pool)
            .vcenter_workload_folder(&self.datacenter.folder)
            .cluster(VSphereK8sClusterInfo {
                name: format!("${{{}.clusterName}}", cluster_name),
                control_plane_endpoint_ip: format!("${{{}.endpoint}}", cluster_name),
                kubeconfig_base64: format!("${{{}.encodedKubeconfig}}", cluster_name),
            })
            .custom_user_data(
                bottlerocket_input
                    .crd_input
                    .encoded_userdata()?
                    .map(|encoded_userdata| CustomUserData::Merge { encoded_userdata }),
            )
            .assume_role(bottlerocket_input.crd_input.config.agent_role.clone())
            .set_labels(Some(labels))
            .set_conflicts_with(Some(existing_clusters))
            .destruction_policy(
                bottlerocket_input
                    .crd_input
                    .config
                    .dev
                    .bottlerocket_destruction_policy
                    .to_owned()
                    .unwrap_or(DestructionPolicy::OnTestSuccess),
            )
            .image(
                bottlerocket_input
                    .crd_input
                    .images
                    .vsphere_vm_resource_agent_image
                    .as_ref()
                    .expect("The default vSphere VM resource provider image URI is missing."),
            )
            .set_image_pull_secret(
                bottlerocket_input
                    .crd_input
                    .images
                    .testsys_agent_pull_secret
                    .to_owned(),
            )
            .set_secrets(Some(
                bottlerocket_input
                    .crd_input
                    .config
                    .secrets
                    .clone()
                    .into_iter()
                    .chain(self.creds.clone())
                    .collect(),
            ))
            .depends_on(cluster_name)
            .build(format!("{}-vms-{}", cluster_name, suffix))
            .context(error::BuildSnafu {
                what: "vSphere VM CRD",
            })?;
        Ok(CreateCrdOutput::NewCrd(Box::new(Crd::Resource(
            vsphere_vm_crd,
        ))))
    }

    async fn migration_crd<'a>(
        &self,
        migration_input: MigrationInput<'a>,
    ) -> Result<CreateCrdOutput> {
        Ok(CreateCrdOutput::NewCrd(Box::new(Crd::Test(migration_crd(
            migration_input,
            // Let the migration test's SSM RunDocuments and RunCommand invocations happen in 'us-west-2'
            // FIXME: Do we need to allow this to be configurable?
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
