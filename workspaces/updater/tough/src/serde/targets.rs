use crate::serde::{Hashes, Metadata, Role};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::num::NonZeroU64;

// We do not handle delegation in this library.
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "_type")]
#[serde(rename = "targets")]
pub(crate) struct Targets {
    pub(crate) expires: DateTime<Utc>,
    pub(crate) spec_version: String,
    pub(crate) targets: BTreeMap<String, Target>,
    pub(crate) version: NonZeroU64,
}

impl Metadata for Targets {
    const ROLE: Role = Role::Targets;

    fn expires(&self) -> &DateTime<Utc> {
        &self.expires
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Target {
    #[serde(default)]
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub(crate) custom: BTreeMap<String, serde_json::Value>,
    pub(crate) hashes: Hashes,
    pub(crate) length: usize,
}
