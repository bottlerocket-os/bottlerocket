use crate::error::Result;
use clap::Parser;
use log::{info, trace};
use model::test_manager::{ImageConfig, TestManager};

/// The install subcommand is responsible for putting all of the necessary components for testsys in
/// a k8s cluster.
#[derive(Debug, Parser)]
pub(crate) struct Install {
    /// Controller image pull secret. This is the name of a Kubernetes secret that will be used to
    /// pull the container image from a private registry. For example, if you created a pull secret
    /// with `kubectl create secret docker-registry regcred` then you would pass
    /// `--controller-pull-secret regcred`.
    #[clap(
        long = "controller-pull-secret",
        env = "TESTSYS_CONTROLLER_PULL_SECRET"
    )]
    secret: Option<String>,

    /// Controller image uri. If not provided the latest released controller image will be used.
    #[clap(
        long = "controller-uri",
        env = "TESTSYS_CONTROLLER_IMAGE",
        default_value = "public.ecr.aws/bottlerocket-test-system/controller:v0.0.4"
    )]
    controller_uri: String,
}

impl Install {
    pub(crate) async fn run(self, client: TestManager) -> Result<()> {
        trace!(
            "Installing testsys using controller image '{}'",
            &self.controller_uri
        );
        let controller_image = match (self.secret, self.controller_uri) {
            (Some(secret), image) => ImageConfig::WithCreds { secret, image },
            (None, image) => ImageConfig::Image(image),
        };
        client.install(controller_image).await?;

        info!("testsys components were successfully installed.");

        Ok(())
    }
}
