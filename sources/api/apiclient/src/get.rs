use snafu::{OptionExt, ResultExt};
use std::path::Path;

mod merge_json;
use merge_json::merge_json;

/// Fetches the given prefixes from the API and merges them into a single Value.  (It's not
/// expected that given prefixes would overlap, but if they do, later ones take precedence.)
pub async fn get_prefixes<P>(socket_path: P, prefixes: Vec<String>) -> Result<serde_json::Value>
where
    P: AsRef<Path>,
{
    let mut results: Vec<serde_json::Value> = Vec::with_capacity(prefixes.len());

    // Fetch all given prefixes into separate Values.
    for prefix in prefixes {
        let uri = format!("/?prefix={}", prefix);
        let method = "GET";
        let (_status, body) = crate::raw_request(&socket_path, &uri, method, None)
            .await
            .context(error::RequestSnafu { uri, method })?;
        let value = serde_json::from_str(&body).context(error::ResponseJsonSnafu { body })?;
        results.push(value);
    }

    // Merge results together.
    results
        .into_iter()
        .reduce(|mut merge_into, merge_from| {
            merge_json(&mut merge_into, merge_from);
            merge_into
        })
        .context(error::NoPrefixesSnafu)
}

/// Fetches the given URI from the API and returns the result as an untyped Value.
pub async fn get_uri<P>(socket_path: P, uri: String) -> Result<serde_json::Value>
where
    P: AsRef<Path>,
{
    let method = "GET";
    let (_status, body) = crate::raw_request(&socket_path, &uri, method, None)
        .await
        .context(error::RequestSnafu { uri, method })?;
    serde_json::from_str(&body).context(error::ResponseJsonSnafu { body })
}

mod error {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub enum Error {
        #[snafu(display("Must give prefixes to query"))]
        NoPrefixes,

        #[snafu(display("Failed {} request to '{}': {}", method, uri, source))]
        Request {
            method: String,
            uri: String,
            #[snafu(source(from(crate::Error, Box::new)))]
            source: Box<crate::Error>,
        },

        #[snafu(display("Response contained invalid JSON '{}' - {}", body, source))]
        ResponseJson {
            body: String,
            source: serde_json::Error,
        },
    }
}
pub use error::Error;
pub type Result<T> = std::result::Result<T, error::Error>;
