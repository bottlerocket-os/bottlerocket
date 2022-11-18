use crate::error::Result;
use clap::Parser;
use futures::TryStreamExt;
use log::info;
use model::test_manager::{DeleteEvent, TestManager};

/// Delete all tests and resources from a testsys cluster.
#[derive(Debug, Parser)]
pub(crate) struct Delete {}

impl Delete {
    pub(crate) async fn run(self, client: TestManager) -> Result<()> {
        let mut stream = client.delete_all().await?;

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
