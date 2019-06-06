use serde::de::DeserializeOwned;
use snafu::ResultExt;

use crate::{error, Result};

/// This trait extends the client from reqwest, abstracting the repeated
/// request logic and returning JSON or an error if there was one.
pub(crate) trait ReqwestClientExt {
    fn get_json<T: DeserializeOwned>(
        &self,
        uri: String,
        query_param: String,
        query: String,
    ) -> Result<T>;
}

impl ReqwestClientExt for reqwest::Client {
    fn get_json<T: DeserializeOwned>(
        &self,
        uri: String,
        query_param: String,
        query: String,
    ) -> Result<T> {
        self.get(&uri)
            .query(&[(query_param, query)])
            .send().context(error::APIRequest {
                method: "GET",
                uri: uri.as_str(),
            })?
            .error_for_status()
            .context(error::APIResponse {
                method: "GET",
                uri: uri.as_str(),
            })?
            .json()
            .context(error::ResponseJson {
                method: "GET",
                uri: uri.as_str(),
            })
    }
}
