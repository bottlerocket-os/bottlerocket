#![warn(clippy::pedantic)]

mod de;
pub mod error;
mod se;

use chrono::{DateTime, Duration, Utc};
use data_store_version::Version as DVersion;
use rand::{thread_rng, Rng};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::ops::Bound::{Excluded, Included};

pub const MAX_SEED: u32 = 2048;

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
    pub version: Version,
    pub max_version: Version,
    #[serde(deserialize_with = "de::deserialize_bound")]
    pub waves: BTreeMap<u32, DateTime<Utc>>,
    pub images: Images,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Manifest {
    pub updates: Vec<Update>,
    #[serde(deserialize_with = "de::deserialize_migration")]
    #[serde(serialize_with = "se::serialize_migration")]
    pub migrations: BTreeMap<(DVersion, DVersion), Vec<String>>,
    #[serde(deserialize_with = "de::deserialize_datastore_map")]
    #[serde(serialize_with = "se::serialize_datastore_map")]
    pub datastore_versions: BTreeMap<Version, DVersion>,
}

impl Update {
    pub fn update_wave(&self, seed: u32) -> Option<&DateTime<Utc>> {
        self.waves
            .range((Included(0), Included(seed)))
            .last()
            .map(|(_, wave)| wave)
    }

    pub fn update_ready(&self, seed: u32) -> bool {
        // Has this client's wave started
        if let Some(wave) = self.update_wave(seed) {
            return *wave <= Utc::now();
        }

        // Alternately have all waves passed
        if let Some((_, wave)) = self.waves.iter().last() {
            return *wave <= Utc::now();
        }

        // Or there are no waves
        true
    }

    pub fn jitter(&self, seed: u32) -> Option<DateTime<Utc>> {
        let prev = self.update_wave(seed);
        let next = self
            .waves
            .range((Excluded(seed), Excluded(MAX_SEED)))
            .next()
            .map(|(_, wave)| wave);
        if let (Some(start), Some(end)) = (prev, next) {
            if Utc::now() < *end {
                let mut rng = thread_rng();
                if let Some(range) = end.timestamp().checked_sub(start.timestamp()) {
                    return Some(*start + Duration::seconds(rng.gen_range(1, range)));
                }
            }
        }
        None
    }
}
