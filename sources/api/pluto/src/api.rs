pub(super) use inner::{get_aws_k8s_info, Error};

/// The result type for the [`api`] module.
pub(super) type Result<T> = std::result::Result<T, Error>;

#[derive(Default)]
pub(crate) struct AwsK8sInfo {
    pub(crate) region: String,
    pub(crate) cluster_name: String,
}

/// This code is the 'actual' implementation compiled when the `sources` workspace is being compiled
/// for `aws-k8s-*` variants.
// TODO - find a better way https://github.com/bottlerocket-os/bottlerocket/issues/1260
#[cfg(aws_k8s_variant)]
mod inner {
    use super::*;
    use snafu::{OptionExt, ResultExt, Snafu};
    use constants;

    #[derive(Debug, Snafu)]
    pub(crate) enum Error {
        #[snafu(display("Error calling Bottlerocket API: {}", source))]
        ApiClient {
            source: apiclient::Error,
            uri: String,
        },

        #[snafu(display("The '{}' setting is missing", setting))]
        Missing { setting: String },

        #[snafu(display("Unable to deserialize Bottlerocket settings: {}", source))]
        SettingsJson { source: serde_json::Error },
    }

    /// Gets the Bottlerocket settings from the API and deserializes them into a struct.
    async fn get_settings() -> Result<model::Settings> {
        let uri = constants::API_SETTINGS_URI;
        let (_status, response_body) =
            apiclient::raw_request(constants::API_SOCKET, uri, "GET", None)
                .await
                .context(ApiClient { uri })?;

        serde_json::from_str(&response_body).context(SettingsJson)
    }

    /// Gets the info that we need to know about the EKS cluster from the Bottlerocket API.
    pub(crate) async fn get_aws_k8s_info() -> Result<AwsK8sInfo> {
        let settings = get_settings().await?;
        Ok(AwsK8sInfo {
            region: settings
                .aws
                .context(Missing { setting: "aws" })?
                .region
                .context(Missing { setting: "region" })?
                .into(),
            cluster_name: settings
                .kubernetes
                .context(Missing {
                    setting: "kubernetes",
                })?
                .cluster_name
                .context(Missing {
                    setting: "cluster-name",
                })?
                .into(),
        })
    }
}

/// This dummy code is compiled when the `sources` workspace is being compiled for non `aws-k8s-*`
/// variants.
// TODO - find a better way https://github.com/bottlerocket-os/bottlerocket/issues/1260
#[cfg(not(aws_k8s_variant))]
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
        WrongVariant.fail()
    }
}
