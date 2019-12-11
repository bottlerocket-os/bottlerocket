#![warn(clippy::pedantic)]

mod de;
pub mod error;
mod se;

use chrono::{DateTime, Duration, Utc};
use data_store_version::Version as DataVersion;
use migrator::MIGRATION_FILENAME_RE;
use rand::{thread_rng, Rng};
use semver::Version as SemVer;
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
        from: DataVersion,
        to: DataVersion,
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
            let version = DataVersion::from_str(version_match.as_str())
                .context(error::BadDataVersion { key: name })?;
            ensure!(
                version == to,
                error::MigrationInvalidTarget { name, to, version }
            );

            let _ = captures
                .name("name")
                .context(error::BadRegexName { name })?;
        }

        // If append is true, append the new migrations to the existing vec.
        if append && self.migrations.contains_key(&(from, to)) {
            let migrations = self
                .migrations
                .get_mut(&(from, to))
                .context(error::MigrationMutable { from: from, to: to })?;
            migrations.extend_from_slice(&migration_list);
        // Otherwise just overwrite the existing migrations
        } else {
            self.migrations.insert((from, to), migration_list);
        }
        Ok(())
    }

    pub fn add_update(
        &mut self,
        image_version: SemVer,
        max_version: Option<SemVer>,
        datastore_version: DataVersion,
        arch: String,
        flavor: String,
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
            flavor,
            arch,
            version: image_version.clone(),
            max_version: max_version.clone(),
            images,
            waves: BTreeMap::new(),
        };
        self.datastore_versions
            .insert(image_version, datastore_version);
        self.update_max_version(
            &update.max_version,
            Some(&update.arch),
            Some(&update.flavor),
        );
        self.updates.push(update);
        Ok(())
    }

    /// Update the maximum version for all updates that optionally match the
    /// architecture and flavor of some new update.
    pub fn update_max_version(
        &mut self,
        version: &SemVer,
        arch: Option<&str>,
        flavor: Option<&str>,
    ) {
        let matching: Vec<&mut Update> = self
            .updates
            .iter_mut()
            .filter(|update| match (arch, flavor) {
                (Some(arch), Some(flavor)) => update.arch == arch && update.flavor == flavor,
                (Some(arch), None) => update.arch == arch,
                (None, Some(flavor)) => update.flavor == flavor,
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
        flavor: String,
        arch: String,
        image_version: SemVer,
        bound: u32,
        start: DateTime<Utc>,
    ) -> Result<usize> {
        let matching: Vec<&mut Update> = self
            .updates
            .iter_mut()
            // Find the update that exactly matches the specified update
            .filter(|update| {
                update.arch == arch && update.flavor == flavor && update.version == image_version
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
        flavor: String,
        arch: String,
        image_version: SemVer,
        bound: u32,
    ) -> Result<()> {
        let matching: Vec<&mut Update> = self
            .updates
            .iter_mut()
            .filter(|update| {
                update.arch == arch && update.flavor == flavor && update.version == image_version
            })
            .collect();
        for update in matching {
            update.waves.remove(&bound);
        }
        Ok(())
    }
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
