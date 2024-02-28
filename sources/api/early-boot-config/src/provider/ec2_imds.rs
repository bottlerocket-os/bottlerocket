/// EC2 Instance Metadata Service
use async_trait::async_trait;
use imdsclient::ImdsClient;
use snafu::ResultExt;
use user_data_provider::compression::expand_slice_maybe;
use user_data_provider::provider::UserDataProvider;
use user_data_provider::settings::SettingsJson;

pub struct Ec2Imds;

#[async_trait]
impl UserDataProvider for Ec2Imds {
    async fn user_data(
        &self,
    ) -> std::result::Result<Option<SettingsJson>, Box<dyn std::error::Error>> {
        let mut client = ImdsClient::new();

        info!("Fetching user data from IMDS");
        let user_data_raw = match client
            .fetch_userdata()
            .await
            .context(error::ImdsRequestSnafu)?
        {
            Some(user_data_raw) => user_data_raw,
            None => return Ok(None),
        };

        let user_data_str = expand_slice_maybe(&user_data_raw)
            .context(error::DecompressionSnafu { what: "user data" })?;

        if user_data_str.trim().is_empty() {
            warn!("No user data found via IMDS");
            return Ok(None);
        }

        trace!("Received user data: {}", user_data_str);
        let json = SettingsJson::from_toml_str(&user_data_str, "EC2 IMDS").context(
            error::SettingsToJSONSnafu {
                from: "instance user data",
            },
        )?;
        Ok(Some(json))
    }
}

mod error {
    use snafu::Snafu;
    use std::io;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub(crate) enum Error {
        #[snafu(display("Failed to decompress {}: {}", what, source))]
        Decompression { what: String, source: io::Error },

        #[snafu(display("IMDS request failed: {}", source))]
        ImdsRequest { source: imdsclient::Error },

        #[snafu(display("Unable to serialize settings from {}: {}", from, source))]
        SettingsToJSON {
            from: String,
            source: user_data_provider::settings::Error,
        },
    }
}
