/// This module contains utilities for querying IMDS about the AWS domain in which this host resides.
use argh::FromArgs;
use imdsclient::ImdsClient;
use snafu::ResultExt;

use crate::error::{self, Result};

#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "fetch-domain")]
/// Fetch and return the domain from IMDS meta-data.
pub(crate) struct FetchDomain {}

impl FetchDomain {
    const FALLBACK_DOMAIN: &'static str = "amazonaws.com";
    pub(crate) async fn run(self) -> Result<()> {
        let mut client = ImdsClient::new();

        let instance_domain = client
            .fetch_domain()
            .await
            .context(error::ImdsClientSnafu)?
            .unwrap_or_else(|| Self::FALLBACK_DOMAIN.to_owned());

        log::info!("Outputting domain");
        // sundog expects JSON-serialized output so that many types can be represented, allowing the
        // API model to use more accurate types.
        let output = serde_json::to_string(&instance_domain).context(error::SerializeJsonSnafu)?;

        println!("{}", output);
        Ok(())
    }
}
