use log::info;
use snafu::ResultExt;
use std::path::Path;

/// Requests a reboot through the API.
pub async fn reboot<P>(socket_path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let uri = "/actions/reboot";
    let method = "POST";
    let (_status, _body) = crate::raw_request(&socket_path, uri, method, None)
        .await
        .context(error::RequestSnafu { uri, method })?;

    info!("Rebooting, goodbye...");
    Ok(())
}

mod error {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub enum Error {
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
