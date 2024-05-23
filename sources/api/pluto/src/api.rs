use serde::{Deserialize, Serialize};
use snafu::{ensure, ResultExt, Snafu};
use std::ffi::OsStr;
use tokio::process::Command;

/// The result type for the [`api`] module.
pub(super) type Result<T> = std::result::Result<T, Error>;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct AwsK8sInfo {
    #[serde(skip)]
    pub(crate) region: Option<String>,
    #[serde(skip)]
    pub(crate) https_proxy: Option<String>,
    #[serde(skip)]
    pub(crate) no_proxy: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) cluster_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) cluster_dns_ip: Option<modeled_types::KubernetesClusterDnsIp>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) node_ip: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) max_pods: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) provider_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) hostname_override: Option<String>,
    #[serde(skip)]
    pub(crate) variant_id: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct AwsInfo {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) region: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Kubernetes {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) cluster_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) cluster_dns_ip: Option<modeled_types::KubernetesClusterDnsIp>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) node_ip: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) max_pods: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) provider_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) hostname_override: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct Os {
    variant_id: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Network {
    https_proxy: Option<String>,
    no_proxy: Option<String>,
}

#[derive(Deserialize)]
struct View {
    pub aws: Option<AwsInfo>,
    pub network: Option<Network>,
    pub kubernetes: Option<Kubernetes>,
}

#[derive(Deserialize)]
struct SettingsView {
    pub os: Os,
    pub settings: View,
}

#[derive(Debug, Snafu)]
pub(crate) enum Error {
    #[snafu(display("Failed to call apiclient: {}", source))]
    CommandFailure { source: std::io::Error },
    #[snafu(display("apiclient execution failed: {}", reason))]
    ExecutionFailure { reason: String },
    #[snafu(display("Deserialization of configuration file failed: {}", source))]
    Deserialize {
        #[snafu(source(from(serde_json::Error, Box::new)))]
        source: Box<serde_json::Error>,
    },
}

pub(crate) async fn client_command<I, S>(args: I) -> Result<Vec<u8>>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let result = Command::new("/usr/bin/apiclient")
        .args(args)
        .output()
        .await
        .context(CommandFailureSnafu)?;

    ensure!(
        result.status.success(),
        ExecutionFailureSnafu {
            reason: String::from_utf8_lossy(&result.stderr)
        }
    );

    Ok(result.stdout)
}

/// Gets the info that we need to know about the EKS cluster from the Bottlerocket API.
pub(crate) async fn get_aws_k8s_info() -> Result<AwsK8sInfo> {
    let view_str = client_command(&[
        "get",
        "os.variant_id",
        "settings.aws.region",
        "settings.network.http-proxy",
        "settings.network.no-proxy",
        "settings.kubernetes.cluster-name",
        "settings.kubernetes.cluster-dns-ip",
        "settings.kubernetes.node-ip",
        "settings.kubernetes.max-pods",
        "settings.kubernetes.provider-id",
        "settings.kubernetes.hostname-override",
    ])
    .await?;
    let view: SettingsView =
        serde_json::from_slice(view_str.as_slice()).context(DeserializeSnafu)?;

    Ok(AwsK8sInfo {
        variant_id: view.os.variant_id,
        region: view.settings.aws.and_then(|a| a.region),
        https_proxy: view
            .settings
            .network
            .as_ref()
            .and_then(|n| n.https_proxy.clone()),
        no_proxy: view
            .settings
            .network
            .as_ref()
            .and_then(|n| n.no_proxy.clone()),
        cluster_name: view
            .settings
            .kubernetes
            .as_ref()
            .and_then(|k| k.cluster_name.clone()),
        cluster_dns_ip: view
            .settings
            .kubernetes
            .as_ref()
            .and_then(|k| k.cluster_dns_ip.clone()),
        node_ip: view
            .settings
            .kubernetes
            .as_ref()
            .and_then(|k| k.node_ip.clone()),
        max_pods: view.settings.kubernetes.as_ref().and_then(|k| k.max_pods),
        provider_id: view
            .settings
            .kubernetes
            .as_ref()
            .and_then(|k| k.provider_id.clone()),
        hostname_override: view
            .settings
            .kubernetes
            .as_ref()
            .and_then(|k| k.hostname_override.clone()),
    })
}
