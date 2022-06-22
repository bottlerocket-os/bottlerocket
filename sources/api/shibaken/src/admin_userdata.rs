/// This module contains utilities for populating userdata for the admin-container with user SSH keys from IMDS.
use argh::FromArgs;
use imdsclient::ImdsClient;
use serde::Serialize;
use snafu::ResultExt;

use crate::error::{self, Result};

#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "generate-admin-userdata")]
/// Fetch and populate the admin container's user-data with authorized ssh keys.
pub(crate) struct GenerateAdminUserdata {}

impl GenerateAdminUserdata {
    pub(crate) async fn run(self) -> Result<()> {
        let public_keys = fetch_public_keys_from_imds().await?;

        let user_data = UserData::new(public_keys);

        log::info!("Generating user-data");
        // Serialize user_data to a JSON string that can be read by the admin container.
        let user_data_json =
            serde_json::to_string(&user_data).context(error::SerializeJsonSnafu)?;
        log::debug!("{}", &user_data_json);

        log::info!("Encoding user-data");
        // admin container user-data must be base64-encoded to be passed through to the admin container
        // using a setting, rather than another arbitrary storage mechanism. This approach allows the
        // user to bypass shibaken and use their own user-data if desired.
        let user_data_base64 = base64::encode(&user_data_json);

        log::info!("Outputting base64-encoded user-data");
        // sundog expects JSON-serialized output so that many types can be represented, allowing the
        // API model to use more accurate types.
        let output = serde_json::to_string(&user_data_base64).context(error::SerializeJsonSnafu)?;

        println!("{}", output);

        Ok(())
    }
}

#[derive(Serialize)]
struct UserData {
    ssh: Ssh,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
struct Ssh {
    authorized_keys: Vec<String>,
}
impl UserData {
    fn new(public_keys: Vec<String>) -> Self {
        UserData {
            ssh: Ssh {
                authorized_keys: public_keys,
            },
        }
    }
}

/// Returns a list of public keys.
async fn fetch_public_keys_from_imds() -> Result<Vec<String>> {
    log::info!("Connecting to IMDS");
    let mut client = ImdsClient::new();
    let public_keys = client
        .fetch_public_ssh_keys()
        .await
        .context(error::ImdsClientSnafu)?
        .unwrap_or_default();
    Ok(public_keys)
}
