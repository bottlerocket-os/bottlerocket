use std::path::Path;

use modeled_types;
use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Snafu};

/// The result type for the [`api`] module.
pub(super) type Result<T> = std::result::Result<T, Error>;

/// Default Configuration Path
const DEFAULT_CONFIG_PATH: &str = "/etc/pluto.toml";

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct AwsK8sInfo {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) region: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) cluster_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) cluster_dns_ip: Option<modeled_types::KubernetesClusterDnsIp>,
}

#[derive(Debug, Snafu)]
pub(crate) enum Error {
    #[snafu(display("Failed to read configuration file: {}", source))]
    Read {
        #[snafu(source(from(std::io::Error, Box::new)))]
        source: Box<std::io::Error>,
    },
    #[snafu(display("Deserialization of configuration file failed: {}", source))]
    DeserializeToml {
        #[snafu(source(from(toml::de::Error, Box::new)))]
        source: Box<toml::de::Error>,
    },
}

/// Gets the info that we need to know about the EKS cluster from the configuration file.
/// Note that we cannot rely on the configuration file being generated as pluto gets called
/// before template files are fully rendered. (i.e. Pluto self resolves some of the values that
/// are used in the configuration file). To keep this behavior sane if the configuration file
/// does not exist, return None and auto populate the region off IMDS
pub(crate) async fn get_aws_k8s_info() -> Result<AwsK8sInfo> {
    let path = Path::new(DEFAULT_CONFIG_PATH);
    if path.exists() {
        let config = tokio::fs::read_to_string(DEFAULT_CONFIG_PATH)
            .await
            .context(ReadSnafu)?;
        toml::from_str(config.as_str()).context(DeserializeTomlSnafu)
    } else {
        // Attempt to fetch the region from imds since AWS_REGION is not set
        let mut imds = imdsclient::ImdsClient::new();
        let region = imds.fetch_region().await.unwrap();
        Ok(AwsK8sInfo {
            cluster_dns_ip: None,
            cluster_name: None,
            region,
        })
    }
}
