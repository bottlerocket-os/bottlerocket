use crate::error::{self, Result};
use clap::Parser;
use log::{debug, info};
use model::test_manager::{CrdState, CrdType, SelectionParams, TestManager};
use snafu::ResultExt;

/// Check the status of testsys objects.
#[derive(Debug, Parser)]
pub(crate) struct Status {
    /// Output the results in JSON format.
    #[clap(long = "json")]
    json: bool,

    /// Check the status of the testsys controller
    #[clap(long, short = 'c')]
    controller: bool,

    /// Focus status on a particular arch
    #[clap(long)]
    arch: Option<String>,

    /// Focus status on a particular variant
    #[clap(long)]
    variant: Option<String>,

    /// Only show tests
    #[clap(long)]
    test: bool,

    /// Only show passed tests
    #[clap(long, conflicts_with_all=&["failed", "running"])]
    passed: bool,

    /// Only show failed tests
    #[clap(long, conflicts_with_all=&["passed", "running"])]
    failed: bool,

    /// Only CRD's that haven't finished
    #[clap(long, conflicts_with_all=&["passed", "failed"])]
    running: bool,
}

impl Status {
    pub(crate) async fn run(self, client: TestManager) -> Result<()> {
        let state = if self.running {
            Some(CrdState::NotFinished)
        } else if self.passed {
            Some(CrdState::Passed)
        } else if self.failed {
            Some(CrdState::Failed)
        } else {
            None
        };
        let crd_type = self.test.then_some(CrdType::Test);
        let mut labels = Vec::new();
        if let Some(arch) = self.arch {
            labels.push(format!("testsys/arch={}", arch))
        };
        if let Some(variant) = self.variant {
            labels.push(format!("testsys/variant={}", variant))
        };
        let mut status = client
            .status(
                &SelectionParams {
                    labels: Some(labels.join(",")),
                    state,
                    crd_type,
                    ..Default::default()
                },
                self.controller,
            )
            .await?;
        status.new_column("BUILD ID", |crd| {
            crd.labels().get("testsys/build-id").cloned()
        });

        if self.json {
            info!(
                "{}",
                serde_json::to_string_pretty(&status).context(error::SerdeJsonSnafu {
                    what: "Could not create string from status."
                })?
            );
        } else {
            let (width, _) = term_size::dimensions().unwrap_or((80, 0));
            debug!("Window width '{}'", width);
            println!("{:width$}", status);
        }
        Ok(())
    }
}
