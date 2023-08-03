//! This module contains the interface `SettingsResolver` for fetching settings from the Bottlerocket API.
//!
//! To do so we thinly wrap the apiclient to simplify retrieving JSON settings values.
use super::as_std_err;
use crate::v2::ExtensionRequirement;
use async_trait::async_trait;
use cached::proc_macro::once;
use constants::API_SETTINGS_URI;
use serde_json::{json, Map, Value};
use snafu::{ensure, OptionExt, ResultExt};
use std::path::{Path, PathBuf};

const SETTINGS_API_URI: &str = "/";

/// An interface which abstracts away the fetching of Bottlerocket settings for template rendering.
#[async_trait]
pub trait SettingsResolver {
    async fn fetch_settings<I>(
        &self,
        extension_requirements: I,
    ) -> std::result::Result<serde_json::Value, Box<dyn std::error::Error>>
    where
        I: Iterator<Item = ExtensionRequirement> + Send;
}

/// `SettingsResolver` implementation that fetches settings from the Bottlerocket API.
#[derive(Debug, Clone)]
pub struct BottlerocketSettingsResolver {
    pub api_socket: PathBuf,
}

impl BottlerocketSettingsResolver {
    pub fn new(api_socket: PathBuf) -> Self {
        Self { api_socket }
    }
}

impl Default for BottlerocketSettingsResolver {
    fn default() -> Self {
        Self {
            api_socket: constants::API_SOCKET.into(),
        }
    }
}

#[async_trait]
impl SettingsResolver for BottlerocketSettingsResolver {
    /// Fetches requested settings from the Bottlerocket API.
    async fn fetch_settings<I>(
        &self,
        extension_requirements: I,
    ) -> std::result::Result<serde_json::Value, Box<dyn std::error::Error>>
    where
        I: Iterator<Item = ExtensionRequirement> + Send,
    {
        // TODO: Modify this to use per-setting requests in the future.
        let all_settings = get_settings_json(&self.api_socket)
            .await?
            .as_object()
            .cloned()
            .context(error::NonJSONObjectSnafu {
                key: "settings".to_string(),
            })
            .map_err(as_std_err)?;

        let settings = Self::minimize_settings(
            &Self::extract_key_from_api_response("settings", &all_settings).map_err(as_std_err)?,
            extension_requirements,
        );
        let os = Self::extract_key_from_api_response("os", &all_settings).map_err(as_std_err)?;

        let template_settings = json!({
            "settings": settings,
            "os": os,
        });

        Ok(template_settings)
    }
}

impl BottlerocketSettingsResolver {
    /// Given all settings from the Bottlerocket API, returns a JSON object containing only the requested settings.
    fn minimize_settings<I>(
        all_settings: &Map<String, Value>,
        extension_requirements: I,
    ) -> Map<String, Value>
    where
        I: Iterator<Item = ExtensionRequirement>,
    {
        // TODO: Extension version is disregarded until extensions are implemented on the API side.
        extension_requirements
            // TODO: Disambiguate unset settings vs extension not installed in the API.
            // This isn't possible until extensions are implemented on the API side, so here empty
            // requested settings always silently proceed.
            .filter_map(|extension_requirement| {
                let setting_name = &extension_requirement.name;
                all_settings
                    .get(setting_name)
                    .cloned()
                    .map(|setting_value| (setting_name.to_string(), setting_value))
            })
            // Collect errors from fetching the settings before merging
            .collect()
    }

    /// Extracts a JSON value from a response from the Bottlerocket API based on a given key.
    fn extract_key_from_api_response(
        key: &str,
        response: &Map<String, Value>,
    ) -> Result<Map<String, Value>> {
        response
            .get(key)
            .context(error::MalformedApiResponseSnafu {
                reason: format!("Response missing `{}` key.", key),
            })?
            .as_object()
            .cloned()
            .context(error::NonJSONObjectSnafu {
                key: key.to_string(),
            })
    }
}

/// Fetches a JSON object containing all settings from the Bottlerocket API.
///
/// This returns the object present at the API root "/", including the "settings" and "os" keys.
/// Results are cached, only calling the API on the first function execution.
#[once(result = true, sync_writes = true)]
pub async fn get_settings_json(socket_path: &Path) -> Result<Value> {
    let method = "GET";
    trace!("{}ing from {}", method, SETTINGS_API_URI);
    let (code, response_body) = apiclient::raw_request(socket_path, SETTINGS_API_URI, method, None)
        .await
        .context(error::APIRequestSnafu {
            method,
            uri: SETTINGS_API_URI,
        })?;

    ensure!(
        code.is_success(),
        error::APIResponseSnafu {
            method,
            uri: SETTINGS_API_URI.to_string(),
            code,
            response_body,
        }
    );
    trace!("JSON response: {}", response_body);
    serde_json::from_str(&response_body).context(error::ResponseJsonSnafu {
        method,
        uri: API_SETTINGS_URI,
    })
}

pub mod error {
    use http::StatusCode;
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub))]
    pub enum Error {
        #[snafu(display("Error {}ing to {}: {}", method, uri, source))]
        APIRequest {
            method: String,
            uri: String,
            #[snafu(source(from(apiclient::Error, Box::new)))]
            source: Box<apiclient::Error>,
        },

        #[snafu(display("Error {} when {}ing to {}: {}", code, method, uri, response_body))]
        APIResponse {
            method: String,
            uri: String,
            code: StatusCode,
            response_body: String,
        },

        #[snafu(display("Received a non-JSON object for Bottlerocket API key '{}'", key))]
        NonJSONObject { key: String },

        #[snafu(display(
            "Nothing found when requesting Bottlerocket setting '{}'",
            setting_name
        ))]
        NoSuchSetting { setting_name: String },

        #[snafu(display(
            "Error deserializing response as JSON from {} to '{}': {}",
            method,
            uri,
            source
        ))]
        ResponseJson {
            method: &'static str,
            uri: String,
            source: serde_json::Error,
        },

        #[snafu(display("Malformed API response: '{}'", reason))]
        MalformedApiResponse { reason: String },
    }
}

pub use error::Error;
type Result<T> = std::result::Result<T, error::Error>;

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::json;

    use crate::v2::ExtensionRequirement;

    #[test]
    fn test_minimize_settings() {
        let test_cases = [
            (json!({}), vec![], json!({})),
            (json!({"nothing": "selected"}), vec![], json!({})),
            (
                json!({
                    "relevant": "setting",
                    "irrelevant": "setting",
                }),
                vec![ExtensionRequirement {
                    name: "relevant".to_string(),
                    version: "v1".to_string(),
                    ..Default::default()
                }],
                json!({
                    "relevant": "setting",
                }),
            ),
            (
                json!({
                    "one": "setting",
                    "two": "settings",
                    "three": "settingses",
                }),
                vec![
                    ExtensionRequirement {
                        name: "one".to_string(),
                        version: "v1".to_string(),
                        ..Default::default()
                    },
                    ExtensionRequirement {
                        name: "two".to_string(),
                        version: "v1".to_string(),
                        ..Default::default()
                    },
                    ExtensionRequirement {
                        name: "three".to_string(),
                        version: "v1".to_string(),
                        ..Default::default()
                    },
                ],
                json!({
                    "one": "setting",
                    "two": "settings",
                    "three": "settingses",
                }),
            ),
        ];

        for (all_settings, extension_requirements, expected_settings) in test_cases.into_iter() {
            let minimized_settings = BottlerocketSettingsResolver::minimize_settings(
                &all_settings.as_object().unwrap(),
                extension_requirements.into_iter(),
            );
            assert_eq!(
                minimized_settings,
                expected_settings.as_object().unwrap().clone()
            );
        }
    }

    #[test]
    fn test_extract_key_from_api_response() {
        let success_test_cases = [
            (
                json!({"top1": {"inner": "value1"}, "top2": {"inner": "value2"}}),
                "top1",
                json!({"inner": "value1"}),
            ),
            (
                json!({"top1": {"inner": "value1"}}),
                "top1",
                json!({"inner": "value1"}),
            ),
        ];

        let failure_test_cases = [
            (
                json!({"settings": {"motd": "hello"}}),
                "requested-key-not-present",
            ),
            (json!({"settings": "not-an-object"}), "settings"),
        ];

        for (response, key, expected_value) in success_test_cases.into_iter() {
            let value: serde_json::Value =
                BottlerocketSettingsResolver::extract_key_from_api_response(
                    key,
                    response.as_object().unwrap(),
                )
                .unwrap()
                .into();
            assert_eq!(value, expected_value);
        }

        for (response, key) in failure_test_cases.into_iter() {
            assert!(BottlerocketSettingsResolver::extract_key_from_api_response(
                key,
                response.as_object().unwrap()
            )
            .is_err());
        }
    }
}
