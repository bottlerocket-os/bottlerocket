use crate::error::Result;
use clap::Parser;
use log::{info, trace};
use testsys_model::test_manager::TestManager;

/// The uninstall subcommand is responsible for removing all of the components for testsys in
/// a k8s cluster. This is completed by removing the `testsys-bottlerocket-aws` namespace.
#[derive(Debug, Parser)]
pub(crate) struct Uninstall {}

impl Uninstall {
    pub(crate) async fn run(self, client: TestManager) -> Result<()> {
        trace!("Uninstalling testsys");

        client.uninstall().await?;

        info!("testsys components were successfully uninstalled.");

        Ok(())
    }
}
