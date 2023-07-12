use snafu::ResultExt;
use std::path::Path;

/// Handles requesting a CIS benchmark report.
async fn get_cis_report<P>(
    socket_path: P,
    report_type: &str,
    format: Option<String>,
    level: Option<i32>,
) -> Result<String>
where
    P: AsRef<Path>,
{
    let method = "GET";

    let mut query: Vec<String> = vec![format!("type={}", report_type)];
    if let Some(query_format) = format {
        query.push(format!("format={}", query_format));
    }
    if let Some(query_level) = level {
        query.push(format!("level={}", query_level));
    }

    let uri = format!("/report/cis?{}", query.join("&"));

    let (_status, body) = crate::raw_request(&socket_path, &uri, method, None)
        .await
        .context(error::RequestSnafu { uri, method })?;

    Ok(body)
}

/// Requests a Bottlerocket CIS compliance report through the API.
pub async fn get_bottlerocket_cis_report<P>(
    socket_path: P,
    format: Option<String>,
    level: Option<i32>,
) -> Result<String>
where
    P: AsRef<Path>,
{
    get_cis_report(socket_path, "bottlerocket", format, level).await
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
