use crate::error::Result;
use clap::Parser;
use futures::TryStreamExt;
use log::info;
use testsys_model::test_manager::{CrdState, CrdType, DeleteEvent, SelectionParams, TestManager};

/// Delete all tests and resources from a testsys cluster.
#[derive(Debug, Parser)]
pub(crate) struct Delete {
    /// Only delete tests
    #[clap(long)]
    test: bool,

    /// Focus status on a particular arch
    #[clap(long)]
    arch: Option<String>,

    /// Focus status on a particular variant
    #[clap(long)]
    variant: Option<String>,

    /// Only delete passed tests
    #[clap(long, conflicts_with_all=&["failed", "running"])]
    passed: bool,

    /// Only delete failed tests
    #[clap(long, conflicts_with_all=&["passed", "running"])]
    failed: bool,

    /// Only CRD's that haven't finished
    #[clap(long, conflicts_with_all=&["passed", "failed"])]
    running: bool,
}

impl Delete {
    pub(crate) async fn run(self, client: TestManager) -> Result<()> {
        let state = if self.running {
            info!("Deleting all running tests and resources");
            Some(CrdState::NotFinished)
        } else if self.passed {
            info!("Deleting all passed tests");
            Some(CrdState::Passed)
        } else if self.failed {
            info!("Deleting all failed tests");
            Some(CrdState::Failed)
        } else {
            info!("Deleting all tests and resources");
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
        let mut stream = client
            .delete(
                &SelectionParams {
                    labels: Some(labels.join(",")),
                    state,
                    crd_type,
                    ..Default::default()
                },
                false,
            )
            .await?;

        while let Some(delete) = stream.try_next().await? {
            match delete {
                DeleteEvent::Starting(crd) => println!("Starting delete for {}", crd.name()),
                DeleteEvent::Deleted(crd) => println!("Delete finished for {}", crd.name()),
                DeleteEvent::Failed(crd) => println!("Delete failed for {}", crd.name()),
            }
        }
        info!("Delete finished");
        Ok(())
    }
}
