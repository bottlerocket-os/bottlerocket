use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::modeled_types::{Identifier, SingleLineString};
use crate::{ContainerImage, NtpSettings, UpdatesSettings};

// Note: we have to use 'rename' here because the top-level Settings structure is the only one
// that uses its name in serialization; internal structures use the field name that points to it
#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename = "settings", rename_all = "kebab-case")]
pub struct Settings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<SingleLineString>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname: Option<SingleLineString>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub updates: Option<UpdatesSettings>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_containers: Option<HashMap<Identifier, ContainerImage>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ntp: Option<NtpSettings>,
}
