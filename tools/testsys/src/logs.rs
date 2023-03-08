use crate::error::{self, Result};
use clap::Parser;
use futures::TryStreamExt;
use snafu::OptionExt;
use testsys_model::test_manager::{ResourceState, TestManager};
use unescape::unescape;

/// Stream the logs of an object from a testsys cluster.
#[derive(Debug, Parser)]
pub(crate) struct Logs {
    /// The name of the test we want logs from.
    #[clap(long, conflicts_with = "resource")]
    test: Option<String>,

    /// The name of the resource we want logs from.
    #[clap(long, conflicts_with = "test", requires = "state")]
    resource: Option<String>,

    /// The resource state we want logs for (Creation, Destruction).
    #[clap(long = "state", conflicts_with = "test")]
    resource_state: Option<ResourceState>,

    /// Follow logs
    #[clap(long, short)]
    follow: bool,
}

impl Logs {
    pub(crate) async fn run(self, client: TestManager) -> Result<()> {
        match (self.test, self.resource, self.resource_state) {
            (Some(test), None, None) => {
                let mut logs = client.test_logs(test, self.follow).await?;
                while let Some(line) = logs.try_next().await? {
                    println!("{}", unescape(&String::from_utf8_lossy(&line)).context(error::InvalidSnafu{what: "Unable to unescape log string"})?);
                }
            }
            (None, Some(resource), Some(state)) => {
                let mut logs = client.resource_logs(resource, state, self.follow).await?;
                while let Some(line) = logs.try_next().await? {
                    println!("{}", unescape(&String::from_utf8_lossy(&line)).context(error::InvalidSnafu{what: "Unable to unescape log string"})?);
                }
            }
            _ => return Err(error::Error::Invalid{what: "Invalid arguments were provided. Exactly one of `--test` or `--resource` must be given.".to_string()}),
        };
        Ok(())
    }
}
