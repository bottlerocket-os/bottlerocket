#![warn(clippy::pedantic)]

mod de;
pub mod error;
mod se;

use chrono::{DateTime, Duration, Utc};
use migrator::MIGRATION_FILENAME_RE;
use rand::{thread_rng, Rng};
use semver::Version;
use serde::{Deserialize, Serialize};
use snafu::{ensure, OptionExt, ResultExt};
use std::collections::BTreeMap;
use std::fs;
use std::fs::File;
use std::ops::Bound::{Excluded, Included};
use std::path::Path;
use std::str::FromStr;

use crate::error::Result;

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
    pub variant: String,
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
    pub migrations: BTreeMap<(Version, Version), Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Release {
    pub version: Version,
    /// For now, this matches the Manifest struct, but having a separate struct gives us the
    /// flexibility to have a different, human-oriented representation in the release TOML compared
    /// to the machine-oriented representation in the manifest.
    #[serde(deserialize_with = "de::deserialize_migration")]
    #[serde(serialize_with = "se::serialize_migration")]
    pub migrations: BTreeMap<(Version, Version), Vec<String>>,
}

pub fn load_file(path: &Path) -> Result<Manifest> {
    let file = File::open(path).context(error::ManifestRead { path })?;
    serde_json::from_reader(file).context(error::ManifestParse)
}

pub fn write_file(path: &Path, manifest: &Manifest) -> Result<()> {
    let manifest = serde_json::to_string_pretty(&manifest).context(error::UpdateSerialize)?;
    fs::write(path, &manifest).context(error::ManifestWrite { path })?;
    Ok(())
}

impl Manifest {
    pub fn add_migration(
        &mut self,
        append: bool,
        from: Version,
        to: Version,
        migration_list: Vec<String>,
    ) -> Result<()> {
        // Check each migration matches the filename conventions used by the migrator
        for name in &migration_list {
            let captures = MIGRATION_FILENAME_RE
                .captures(&name)
                .context(error::MigrationNaming)?;

            let version_match = captures
                .name("version")
                .context(error::BadRegexVersion { name })?;
            let version = Version::from_str(version_match.as_str())
                .context(error::BadVersion { key: name })?;
            ensure!(
                version == to,
                error::MigrationInvalidTarget { name, to, version }
            );

            let _ = captures
                .name("name")
                .context(error::BadRegexName { name })?;
        }

        // If append is true, append the new migrations to the existing vec.
        if append && self.migrations.contains_key(&(from.clone(), to.clone())) {
            let migrations = self
                .migrations
                .get_mut(&(from.clone(), to.clone()))
                .context(error::MigrationMutable { from, to })?;
            migrations.extend_from_slice(&migration_list);
        // Otherwise just overwrite the existing migrations
        } else {
            self.migrations.insert((from, to), migration_list);
        }
        Ok(())
    }

    pub fn add_update(
        &mut self,
        image_version: Version,
        max_version: Option<Version>,
        arch: String,
        variant: String,
        images: Images,
    ) -> Result<()> {
        let max_version = if let Some(version) = max_version {
            version
        } else {
            // Default to greater of the current max version and this version
            if let Some(update) = self.updates.first() {
                std::cmp::max(&image_version, &update.max_version).clone()
            } else {
                image_version.clone()
            }
        };
        let update = Update {
            variant,
            arch,
            version: image_version.clone(),
            max_version: max_version.clone(),
            images,
            waves: BTreeMap::new(),
        };
        self.update_max_version(
            &update.max_version,
            Some(&update.arch),
            Some(&update.variant),
        );
        self.updates.push(update);
        Ok(())
    }

    /// Update the maximum version for all updates that optionally match the
    /// architecture and variant of some new update.
    pub fn update_max_version(
        &mut self,
        version: &Version,
        arch: Option<&str>,
        variant: Option<&str>,
    ) {
        let matching: Vec<&mut Update> = self
            .updates
            .iter_mut()
            .filter(|update| match (arch, variant) {
                (Some(arch), Some(variant)) => update.arch == arch && update.variant == variant,
                (Some(arch), None) => update.arch == arch,
                (None, Some(variant)) => update.variant == variant,
                _ => true,
            })
            .collect();
        for u in matching {
            u.max_version = version.clone();
        }
    }

    fn validate_updates(updates: &[Update]) -> Result<()> {
        for update in updates {
            let mut waves = update.waves.iter().peekable();
            while let Some(wave) = waves.next() {
                if let Some(next) = waves.peek() {
                    ensure!(
                        wave.1 < next.1,
                        error::WavesUnordered {
                            wave: *wave.0,
                            next: *next.0
                        }
                    );
                }
            }
        }
        Ok(())
    }

    /// Adds a wave to update, returns number of matching updates for wave
    pub fn add_wave(
        &mut self,
        variant: String,
        arch: String,
        image_version: Version,
        bound: u32,
        start: DateTime<Utc>,
    ) -> Result<usize> {
        let matching: Vec<&mut Update> = self
            .updates
            .iter_mut()
            // Find the update that exactly matches the specified update
            .filter(|update| {
                update.arch == arch && update.variant == variant && update.version == image_version
            })
            .collect();
        let num_matching = matching.len();
        for update in matching {
            update.waves.insert(bound, start);
        }
        Self::validate_updates(&self.updates)?;
        Ok(num_matching)
    }

    pub fn remove_wave(
        &mut self,
        variant: String,
        arch: String,
        image_version: Version,
        bound: u32,
    ) -> Result<()> {
        let matching: Vec<&mut Update> = self
            .updates
            .iter_mut()
            .filter(|update| {
                update.arch == arch && update.variant == variant && update.version == image_version
            })
            .collect();
        for update in matching {
            update.waves.remove(&bound);
        }
        Ok(())
    }
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
