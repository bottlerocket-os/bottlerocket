mod decoded;
mod key;
mod root;
mod snapshot;
mod targets;
mod timestamp;

pub(crate) use root::Root;
pub(crate) use snapshot::Snapshot;
pub(crate) use targets::{Target, Targets};
pub(crate) use timestamp::Timestamp;

use crate::error::{self, Result};
use crate::serde::decoded::{Decoded, Hex};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_plain::forward_display_to_serde;
use snafu::ensure;
use std::num::NonZeroU64;

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "kebab-case")]
pub enum Role {
    Root,
    Snapshot,
    Targets,
    Timestamp,
}

forward_display_to_serde!(Role);

pub(crate) trait Metadata {
    const ROLE: Role;

    fn expires(&self) -> &DateTime<Utc>;
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Signed<T> {
    pub(crate) signatures: Vec<Signature>,
    pub(crate) signed: T,
}

#[allow(clippy::use_self)] // false positive
impl<T: Metadata + Serialize> Signed<T> {
    pub(crate) fn check_expired(&self) -> Result<()> {
        ensure!(
            Utc::now() < *self.signed.expires(),
            error::ExpiredMetadata { role: T::ROLE }
        );
        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Signature {
    pub(crate) keyid: Decoded<Hex>,
    pub(crate) sig: Decoded<Hex>,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Meta {
    pub(crate) hashes: Hashes,
    pub(crate) length: usize,
    pub(crate) version: NonZeroU64,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Hashes {
    pub(crate) sha256: Decoded<Hex>,
}
