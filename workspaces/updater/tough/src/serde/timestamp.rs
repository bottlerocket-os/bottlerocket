use crate::serde::{Meta, Metadata, Role};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::num::NonZeroU64;

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "_type")]
#[serde(rename = "timestamp")]
pub(crate) struct Timestamp {
    pub(crate) expires: DateTime<Utc>,
    pub(crate) meta: BTreeMap<String, Meta>,
    pub(crate) spec_version: String,
    pub(crate) version: NonZeroU64,
}

impl Metadata for Timestamp {
    const ROLE: Role = Role::Timestamp;

    fn expires(&self) -> &DateTime<Utc> {
        &self.expires
    }
}
