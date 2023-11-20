/// EC2 Identity Document
use super::UserDataProvider;
use crate::settings::SettingsJson;
use async_trait::async_trait;
use imdsclient::ImdsClient;
use serde_json::json;
use snafu::{OptionExt, ResultExt};
use std::{fs, path::Path};

const IDENTITY_DOCUMENT_FILE: &str = "/etc/early-boot-config/identity-document";
const FALLBACK_REGION: &str = "us-east-1";

pub struct Ec2IdentityDoc;

impl Ec2IdentityDoc {
    async fn fetch_region() -> Result<String> {
        let region = if Path::new(IDENTITY_DOCUMENT_FILE).exists() {
            info!("'{}' exists, using it", IDENTITY_DOCUMENT_FILE);
            let data =
                fs::read_to_string(IDENTITY_DOCUMENT_FILE).context(error::InputFileReadSnafu {
                    path: IDENTITY_DOCUMENT_FILE,
                })?;
            let iid: serde_json::Value =
                serde_json::from_str(&data).context(error::DeserializeJsonSnafu)?;

            iid.get("region")
                .context(error::IdentityDocMissingDataSnafu { missing: "region" })?
                .as_str()
                .context(error::WrongTypeSnafu {
                    field_name: "region",
                    expected_type: "string",
                })?
                .to_owned()
        } else {
            info!("Using IMDS for region");
            let mut client = ImdsClient::new();

            client
                .fetch_region()
                .await
                .context(error::ImdsRequestSnafu)?
                .unwrap_or_else(|| FALLBACK_REGION.to_owned())
        };

        Ok(region)
    }
}

#[async_trait]
impl UserDataProvider for Ec2IdentityDoc {
    async fn user_data(
        &self,
    ) -> std::result::Result<Option<SettingsJson>, Box<dyn std::error::Error>> {
        let region = Self::fetch_region().await?;

        trace!(
            "Retrieved region from instance identity document: {}",
            region
        );
        let val = json!({ "aws": {"region": region} });
        let json = SettingsJson::from_val(&val, "EC2 instance identity document").context(
            error::SettingsToJSONSnafu {
                from: "instance identity document",
            },
        )?;

        Ok(Some(json))
    }
}

mod error {
    use snafu::Snafu;
    use std::io;
    use std::path::PathBuf;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub(crate) enum Error {
        #[snafu(display("Error deserializing from JSON: {}", source))]
        DeserializeJson { source: serde_json::error::Error },

        #[snafu(display("Instance identity document missing {}", missing))]
        IdentityDocMissingData { missing: String },

        #[snafu(display("Unable to read input file '{}': {}", path.display(), source))]
        InputFileRead { path: PathBuf, source: io::Error },

        #[snafu(display("IMDS request failed: {}", source))]
        ImdsRequest { source: imdsclient::Error },

        #[snafu(display("Unable to serialize settings from {}: {}", from, source))]
        SettingsToJSON {
            from: String,
            source: crate::settings::Error,
        },

        #[snafu(display(
            "Wrong type while deserializing, expected '{}' to be type '{}'",
            field_name,
            expected_type
        ))]
        WrongType {
            field_name: &'static str,
            expected_type: &'static str,
        },
    }
}
type Result<T> = std::result::Result<T, error::Error>;
