use anyhow::{Context, Result};
use clap::Parser;
use model::test_manager::TestManager;

/// Restart a test. This will delete the test object from the testsys cluster and replace it with
/// a new, identical test object with a clean state.
#[derive(Debug, Parser)]
pub(crate) struct RestartTest {
    /// The name of the test to be restarted.
    #[clap()]
    test_name: String,
}

impl RestartTest {
    pub(crate) async fn run(self, client: TestManager) -> Result<()> {
        client
            .restart_test(&self.test_name)
            .await
            .context(format!("Unable to restart test '{}'", self.test_name))
    }
}
