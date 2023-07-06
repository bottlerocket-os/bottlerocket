use crate::error::Result;
use crate::run::TestsysImages;
use clap::Parser;
use log::{info, trace};
use std::path::PathBuf;
use testsys_config::TestConfig;
use testsys_model::test_manager::{ImageConfig, TestManager};

/// The install subcommand is responsible for putting all of the necessary components for testsys in
/// a k8s cluster.
#[derive(Debug, Parser)]
pub(crate) struct Install {
    /// The path to `Test.toml`
    #[arg(long, env = "TESTSYS_TEST_CONFIG_PATH")]
    test_config_path: PathBuf,

    #[command(flatten)]
    agent_images: TestsysImages,
}

impl Install {
    pub(crate) async fn run(self, client: TestManager) -> Result<()> {
        // Use Test.toml or default
        let test_config = TestConfig::from_path_or_default(&self.test_config_path)?;

        let test_opts = test_config.test.to_owned().unwrap_or_default();

        let images = vec![
            Some(self.agent_images.into()),
            Some(test_opts.testsys_images),
            test_opts.testsys_image_registry.map(|registry| {
                testsys_config::TestsysImages::new(registry, test_opts.testsys_image_tag)
            }),
            Some(testsys_config::TestsysImages::public_images()),
        ]
        .into_iter()
        .flatten()
        .fold(Default::default(), testsys_config::TestsysImages::merge);

        let controller_uri = images
            .controller_image
            .expect("The default controller image is missing.");

        trace!(
            "Installing testsys using controller image '{}'",
            controller_uri
        );

        let controller_image = match images.testsys_agent_pull_secret {
            Some(secret) => ImageConfig::WithCreds {
                secret,
                image: controller_uri,
            },
            None => ImageConfig::Image(controller_uri),
        };
        client.install(controller_image).await?;

        info!("testsys components were successfully installed.");

        Ok(())
    }
}
