use serde::de::DeserializeOwned;

use crate::Result;

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
            .send()?
            .error_for_status()?
            .json()
            .map_err(Into::into)
    }
}
