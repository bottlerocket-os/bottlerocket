use aws_sdk_ec2::error::DescribeImagesError;
use aws_sdk_ec2::types::SdkError;
use snafu::Snafu;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(super)))]
pub enum Error {
    // `error` must be used instead of `source` because the build function returns
    // `std::error::Error` but not `std::error::Error + Sync + Send`.
    #[snafu(display("Unable to build '{}': {}", what, error))]
    Build { what: String, error: String },

    #[snafu(context(false), display("{}", source))]
    DescribeImages {
        source: SdkError<DescribeImagesError>,
    },

    #[snafu(display("Unable to create map from {}: {}", what, source))]
    IntoMap { what: String, source: model::Error },

    #[snafu(display("{}", what))]
    Invalid { what: String },

    #[snafu(display("{}: {}", what, source))]
    IO {
        what: String,
        source: std::io::Error,
    },

    #[snafu(display("Unable to parse K8s version '{}'", version))]
    K8sVersion { version: String },

    #[snafu(display("{}", source))]
    KubeClient { source: kube_client::error::Error },

    #[snafu(display("{} was missing from {}", item, what))]
    Missing { item: String, what: String },

    #[snafu(context(false), display("{}", source))]
    PubsysConfig { source: pubsys_config::Error },

    #[snafu(display("Unable to create secret name for '{}': {}", secret_name, source))]
    SecretName {
        secret_name: String,
        source: model::Error,
    },

    #[snafu(display("{}: {}", what, source))]
    SerdeJson {
        what: String,
        source: serde_json::Error,
    },

    #[snafu(context(false), display("{}", source))]
    TestManager { source: model::test_manager::Error },

    #[snafu(context(false), display("{}", source))]
    TestsysConfig { source: testsys_config::Error },

    #[snafu(display("{} is not supported.", what))]
    Unsupported { what: String },

    #[snafu(display("Unable to create `Variant` from `{}`: {}", variant, source))]
    Variant {
        variant: String,
        source: bottlerocket_variant::error::Error,
    },
}
