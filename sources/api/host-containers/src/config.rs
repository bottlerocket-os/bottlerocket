use modeled_types::{Identifier, Url, ValidBase64};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct HostContainersConfig {
    pub(crate) host_containers: Option<HashMap<Identifier, HostContainer>>,
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct HostContainer {
    pub(crate) source: Option<Url>,
    pub(crate) enabled: Option<bool>,
    pub(crate) superpowered: Option<bool>,
    pub(crate) user_data: Option<ValidBase64>,
}
