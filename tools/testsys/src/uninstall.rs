use anyhow::{Context, Result};
use clap::Parser;
use log::{info, trace};
use model::test_manager::TestManager;

/// The uninstall subcommand is responsible for removing all of the components for testsys in
/// a k8s cluster. This is completed by removing the `testsys-bottlerocket-aws` namespace.
#[derive(Debug, Parser)]
pub(crate) struct Uninstall {}

impl Uninstall {
    pub(crate) async fn run(self, client: TestManager) -> Result<()> {
        trace!("Uninstalling testsys");

        client.uninstall().await.context(
            "Unable to uninstall testsys from the cluster. (Some artifacts may be left behind)",
        )?;

        info!("testsys components were successfully uninstalled.");

        Ok(())
    }
}
