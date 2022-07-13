/// This module contains utilities for querying IMDS about the AWS region in which this host is located.
use argh::FromArgs;
use imdsclient::ImdsClient;
use snafu::ResultExt;

use crate::error::{self, Result};

#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "is-partition")]
/// Fetch and return whether or not this host is in the given partition.
/// Accepts multiple partitions, returning `true` if the host is in any of the given partitions.
pub(crate) struct IsPartition {
    #[argh(option)]
    /// partition(s) to check current instance against
    partition: Vec<String>,
}

impl IsPartition {
    pub(crate) async fn run(self) -> Result<()> {
        let mut client = ImdsClient::new();

        let query_partitions = &self.partition;

        let instance_partition = client
            .fetch_partition()
            .await
            .context(error::ImdsClientSnafu)?;

        let query_result = query_partitions
            .iter()
            .any(|query_partition| Some(query_partition) == instance_partition.as_ref());

        println!("{}", query_result);
        Ok(())
    }
}
