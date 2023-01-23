pub(super) use inner::{get_aws_k8s_info, Error};

/// The result type for the [`api`] module.
pub(super) type Result<T> = std::result::Result<T, Error>;

pub(crate) struct AwsK8sInfo {
    pub(crate) region: Option<String>,
    pub(crate) cluster_name: Option<String>,
    pub(crate) cluster_dns_ip: Option<model::modeled_types::KubernetesClusterDnsIp>,
}

/// This code is the 'actual' implementation compiled when the `sources` workspace is being compiled
/// for `aws-k8s-*` variants.
#[cfg(variant_family = "aws-k8s")]
mod inner {
    use super::*;
    use snafu::{ResultExt, Snafu};

    #[derive(Debug, Snafu)]
    pub(crate) enum Error {
        #[snafu(display("Error calling Bottlerocket API: {}", source))]
        ApiClient {
            #[snafu(source(from(apiclient::Error, Box::new)))]
            source: Box<apiclient::Error>,
            uri: String,
        },

        #[snafu(display("Unable to deserialize Bottlerocket settings: {}", source))]
        SettingsJson { source: serde_json::Error },
    }

    /// Gets the Bottlerocket settings from the API and deserializes them into a struct.
    async fn get_settings() -> Result<model::Settings> {
        let uri = constants::API_SETTINGS_URI;
        let (_status, response_body) =
            apiclient::raw_request(constants::API_SOCKET, uri, "GET", None)
                .await
                .context(ApiClientSnafu { uri })?;

        serde_json::from_str(&response_body).context(SettingsJsonSnafu)
    }

    /// Gets the info that we need to know about the EKS cluster from the Bottlerocket API.
    pub(crate) async fn get_aws_k8s_info() -> Result<AwsK8sInfo> {
        let settings = get_settings().await?;
        Ok(AwsK8sInfo {
            region: settings.aws.and_then(|a| a.region).map(|s| s.into()),
            cluster_name: settings
                .kubernetes
                .as_ref()
                .and_then(|k| k.cluster_name.clone())
                .map(|s| s.into()),
            cluster_dns_ip: settings.kubernetes.and_then(|k| k.cluster_dns_ip),
        })
    }
}

/// This dummy code is compiled when the `sources` workspace is being compiled for non `aws-k8s-*`
/// variants.
#[cfg(not(variant_family = "aws-k8s"))]
mod inner {
    use super::*;
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    pub(crate) enum Error {
        #[snafu(display(
            "The get_aws_k8s_info function is only compatible with aws-k8s variants"
        ))]
        WrongVariant,
    }

    pub(crate) async fn get_aws_k8s_info() -> Result<AwsK8sInfo> {
        WrongVariantSnafu.fail()
    }
}
