use crate::error::{self, Result};
use clap::Parser;
use log::{debug, info};
use model::test_manager::{SelectionParams, TestManager};
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
}

impl Status {
    pub(crate) async fn run(self, client: TestManager) -> Result<()> {
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
