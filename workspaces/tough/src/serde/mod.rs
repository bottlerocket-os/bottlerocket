mod conv;
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
use crate::serde::conv::{Conv, Hex};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_plain::forward_display_to_serde;
use snafu::{ensure, OptionExt, ResultExt};
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
    pub(crate) fn verify(&self, root: &Signed<Root>) -> Result<()> {
        let role_keys = root
            .signed
            .roles
            .get(&T::ROLE)
            .context(error::MissingRole { role: T::ROLE })?;
        let mut valid = 0;

        // TODO(iliana): actually implement Canonical JSON instead of just hoping that what we get
        // out of serde_json is Canonical JSON
        let data = serde_json::to_vec(&self.signed).context(error::JsonSerialization)?;

        for signature in &self.signatures {
            if role_keys.keyids.contains(&signature.keyid) {
                if let Some(key) = root.signed.keys.get(&signature.keyid) {
                    if key.verify(&data, signature.sig.as_slice()) {
                        valid += 1;
                    }
                }
            }
        }

        ensure!(
            valid >= u64::from(role_keys.threshold),
            error::SignatureThreshold {
                threshold: role_keys.threshold,
                valid,
            }
        );
        Ok(())
    }

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
    pub(crate) keyid: Conv<Hex>,
    pub(crate) sig: Conv<Hex>,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Meta {
    pub(crate) hashes: Hashes,
    pub(crate) length: usize,
    pub(crate) version: NonZeroU64,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Hashes {
    pub(crate) sha256: Conv<Hex>,
}
