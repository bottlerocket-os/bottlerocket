use serde::de::DeserializeOwned;
use snafu::ResultExt;
use std::path::Path;

use crate::{error, Result};

/// Simple helper that extends the API client, abstracting the repeated request logic and
/// deserialization from JSON.
pub(crate) fn get_json<T, P, S1, S2, S3>(
    socket_path: P,
    uri: S1,
    // Query parameter name, query parameter value
    query: Option<(S2, S3)>,
) -> Result<T>
where
    T: DeserializeOwned,
    P: AsRef<Path>,
    S1: AsRef<str>,
    S2: AsRef<str>,
    S3: AsRef<str>,
{
    let mut uri = uri.as_ref().to_string();
    // Simplest query string handling; parameters come from API responses so we trust them enough
    // to send back
    if let Some((query_param, query_arg)) = query {
        uri = format!("{}?{}={}", uri, query_param.as_ref(), query_arg.as_ref());
    }

    let method = "GET";
    trace!("{}ing from {}", method, uri);
    let (code, response_body) = apiclient::raw_request(socket_path, &uri, method, None)
        .context(error::APIRequest { method, uri: &uri })?;

    if !code.is_success() {
        return error::APIResponse {
            method,
            uri,
            code,
            response_body,
        }
        .fail();
    }
    trace!("JSON response: {}", response_body);

    serde_json::from_str(&response_body).context(error::ResponseJson { method, uri })
}
