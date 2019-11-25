#![warn(clippy::pedantic)]

mod de;
pub mod error;
mod se;

use chrono::{DateTime, Duration, Utc};
use data_store_version::Version as DataVersion;
use rand::{thread_rng, Rng};
use semver::Version as SemVer;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::ops::Bound::{Excluded, Included};

pub const MAX_SEED: u32 = 2048;

#[derive(Debug, PartialEq, Eq)]
pub enum Wave {
    Initial {
        end: DateTime<Utc>,
    },
    General {
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    },
    Last {
        start: DateTime<Utc>,
    },
}

impl Wave {
    pub fn has_started(&self) -> bool {
        match self {
            Self::Initial { .. } => true,
            Self::General { start, .. } | Self::Last { start } => *start <= Utc::now(),
        }
    }

    pub fn has_passed(&self) -> bool {
        match self {
            Self::Initial { end } => *end <= Utc::now(),
            Self::General { end, .. } => *end <= Utc::now(),
            Self::Last { start } => *start <= Utc::now(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Images {
    pub boot: String,
    pub root: String,
    pub hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Update {
    pub flavor: String,
    pub arch: String,
    pub version: SemVer,
    pub max_version: SemVer,
    #[serde(deserialize_with = "de::deserialize_bound")]
    pub waves: BTreeMap<u32, DateTime<Utc>>,
    pub images: Images,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Manifest {
    pub updates: Vec<Update>,
    #[serde(deserialize_with = "de::deserialize_migration")]
    #[serde(serialize_with = "se::serialize_migration")]
    pub migrations: BTreeMap<(DataVersion, DataVersion), Vec<String>>,
    pub datastore_versions: BTreeMap<SemVer, DataVersion>,
}

impl Update {
    /// Returns the update wave that Updog belongs to, based on the seed value.
    /// Depending on the waves described in the update, the possible results are
    /// - Some wave described by a start and end time.
    /// - The "0th" wave, which has an "end" time but no specified start time.
    /// - The last wave, which has a start time but no specified end time.
    /// - Nothing, if no waves are configured.
    pub fn update_wave(&self, seed: u32) -> Option<Wave> {
        let start = self
            .waves
            .range((Included(0), Excluded(seed)))
            .last()
            .map(|(_, wave)| *wave);
        let end = self
            .waves
            .range((Included(seed), Included(MAX_SEED)))
            .next()
            .map(|(_, wave)| *wave);

        match (start, end) {
            (None, Some(end)) => Some(Wave::Initial { end }),
            (Some(start), Some(end)) => Some(Wave::General { start, end }),
            (Some(start), None) => Some(Wave::Last { start }),
            _ => None,
        }
    }

    pub fn update_ready(&self, seed: u32) -> bool {
        // Has this client's wave started
        if let Some(wave) = self.update_wave(seed) {
            return wave.has_started();
        }

        // Or there are no waves
        true
    }

    pub fn jitter(&self, seed: u32) -> Option<DateTime<Utc>> {
        if let Some(wave) = self.update_wave(seed) {
            if wave.has_passed() {
                return None;
            }
            let bounds = match self.update_wave(seed) {
                Some(Wave::Initial { end }) => Some((Utc::now(), end)),
                Some(Wave::General { start, end }) => Some((start, end)),
                Some(Wave::Last { start: _ }) | None => None,
            };
            if let Some((start, end)) = bounds {
                let mut rng = thread_rng();
                if let Some(range) = end.timestamp().checked_sub(start.timestamp()) {
                    return Some(start + Duration::seconds(rng.gen_range(1, range)));
                }
            }
        }
        None
    }
}
