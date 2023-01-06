use crate::error::{self, Result};
use clap::Parser;
use log::{debug, info};
use model::test_manager::{CrdState, CrdType, SelectionParams, StatusProgress, TestManager};
use serde::Deserialize;
use serde_plain::derive_fromstr_from_deserialize;
use snafu::ResultExt;

/// Check the status of testsys objects.
#[derive(Debug, Parser)]
pub(crate) struct Status {
    /// Configure the output of the command (json, narrow, wide).
    #[clap(long, short = 'o')]
    output: Option<StatusOutput>,

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

        match self.output {
            Some(StatusOutput::Json) => {
                info!(
                    "{}",
                    serde_json::to_string_pretty(&status).context(error::SerdeJsonSnafu {
                        what: "Could not create string from status."
                    })?
                );
                return Ok(());
            }
            Some(StatusOutput::Narrow) => (),
            None => {
                status.new_column("BUILD ID", |crd| {
                    crd.labels().get("testsys/build-id").cloned()
                });
                status.with_time();
            }
            Some(StatusOutput::Wide) => {
                status.new_column("BUILD ID", |crd| {
                    crd.labels().get("testsys/build-id").cloned()
                });
                status.with_time();
                status.with_progress(StatusProgress::WithTests);
            }
        };

        let (width, _) = term_size::dimensions().unwrap_or((80, 0));
        debug!("Window width '{}'", width);
        println!("{:width$}", status);

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum StatusOutput {
    /// Output the status in json
    Json,
    /// Show minimal columns in the status table
    Narrow,
    /// Show all columns in the status table
    Wide,
}

derive_fromstr_from_deserialize!(StatusOutput);
