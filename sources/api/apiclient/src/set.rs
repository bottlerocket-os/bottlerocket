use crate::rando;
use snafu::ResultExt;
use std::path::Path;

/// Changes the requested settings through the API, then commits and applies the transaction
/// containing those changes.  The given Settings only has to be populated (i.e. Option::Some) with
/// the settings you want to change.  If you're deserializing a request from a user, for example,
/// the created Settings will only have the requested keys populated.
pub async fn set<P>(socket_path: P, settings: &model::Settings) -> Result<()>
where
    P: AsRef<Path>,
{
    // We use a specific transaction ID so we don't commit any other changes that may be pending.
    let transaction = format!("apiclient-set-{}", rando());

    // Send the settings changes to the server.
    let uri = format!("/settings?tx={}", transaction);
    let method = "PATCH";
    let request_body = serde_json::to_string(&settings).context(error::SerializeSnafu)?;
    let (_status, _body) = crate::raw_request(&socket_path, &uri, method, Some(request_body))
        .await
        .context(error::RequestSnafu { uri, method })?;

    // Commit the transaction and apply it to the system.
    let uri = format!("/tx/commit_and_apply?tx={}", transaction);
    let method = "POST";
    let (_status, _body) = crate::raw_request(&socket_path, &uri, method, None)
        .await
        .context(error::RequestSnafu { uri, method })?;

    Ok(())
}

mod error {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub enum Error {
        #[snafu(display("Unable to serialize data: {}", source))]
        Serialize { source: serde_json::Error },

        #[snafu(display("Failed {} request to '{}': {}", method, uri, source))]
        Request {
            method: String,
            uri: String,
            #[snafu(source(from(crate::Error, Box::new)))]
            source: Box<crate::Error>,
        },
    }
}
pub use error::Error;
pub type Result<T> = std::result::Result<T, error::Error>;
