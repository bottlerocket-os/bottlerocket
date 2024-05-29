use crate::{rando, SettingsInput};
use snafu::ResultExt;
use std::path::Path;

/// Changes the requested settings through the API, then commits and applies the transaction
/// containing those changes.  The given Settings only has to be populated (i.e. Option::Some) with
/// the settings you want to change.  If you're deserializing a request from a user, for example,
/// the created Settings will only have the requested keys populated.
pub async fn set<P>(socket_path: P, settings: SettingsInput) -> Result<()>
where
    P: AsRef<Path>,
{
    // We use a specific transaction ID so we don't commit any other changes that may be pending.
    let transaction = format!("apiclient-set-{}", rando());

    // Send the settings changes to the server.
    let (uri, settings_data) = match settings {
        SettingsInput::KeyPair(value) => (format!("/settings/keypair?tx={}", transaction), value),
        SettingsInput::Json(value) => (format!("/settings?tx={}", transaction), value),
    };
    let method = "PATCH";
    let (_status, _body) = crate::raw_request(&socket_path, &uri, method, Some(settings_data))
        .await
        .context(error::RequestSnafu)?;

    // Commit the transaction and apply it to the system.
    let uri = format!("/tx/commit_and_apply?tx={}", transaction);
    let method = "POST";
    let (_status, _body) = crate::raw_request(&socket_path, &uri, method, None)
        .await
        .context(error::RequestSnafu)?;

    Ok(())
}

mod error {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub enum Error {
        #[snafu(display("Unable to serialize data: {}", source))]
        Serialize { source: serde_json::Error },

        #[snafu(display("{}", source))]
        Request {
            #[snafu(source(from(crate::Error, Box::new)))]
            source: Box<crate::Error>,
        },
    }
}
pub use error::Error;
pub type Result<T> = std::result::Result<T, error::Error>;
