#![warn(clippy::pedantic)]

mod de;
pub mod error;
mod se;

use crate::error::Result;
use chrono::{DateTime, Duration, Utc};
use parse_datetime::parse_offset;
use rand::{thread_rng, Rng};
use semver::Version;
use serde::{Deserialize, Serialize};
use snafu::{ensure, OptionExt, ResultExt};
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fs;
use std::fs::File;
use std::ops::Bound::{Excluded, Included};
use std::path::Path;

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

/// UpdateWaves is provided for the specific purpose of deserializing
/// update waves from TOML files
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateWaves {
    pub waves: Vec<UpdateWave>,
}

impl UpdateWaves {
    /// Deserializes an UpdateWaves from a given path
    pub fn from_path<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let wave_data = fs::read_to_string(path).context(error::FileRead { path })?;
        toml::from_str(&wave_data).context(error::InvalidToml { path })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateWave {
    pub start_after: String,
    pub fleet_percentage: u32,
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

impl Release {
    /// Deserializes a Release from a given path
    pub fn from_path<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let release_data = fs::read_to_string(path).context(error::FileRead { path })?;
        toml::from_str(&release_data).context(error::InvalidToml { path })
    }
}

pub fn load_file(path: &Path) -> Result<Manifest> {
    let file = File::open(path).context(error::FileRead { path })?;
    serde_json::from_reader(file).context(error::ManifestParse)
}

pub fn write_file(path: &Path, manifest: &Manifest) -> Result<()> {
    let manifest = serde_json::to_string_pretty(&manifest).context(error::UpdateSerialize)?;
    fs::write(path, &manifest).context(error::FileWrite { path })?;
    Ok(())
}

impl Manifest {
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

    // Ensures wave dates and bounds are in ascending order.
    // Update.waves is a BTreeMap which means its keys are always ordered.
    // If a user has fleet percentages (which have been converted to seeds by
    // this point) out of order, we will catch it here as the dates will also
    // be out of order.
    fn validate_updates(updates: &[Update]) -> Result<()> {
        for update in updates {
            let mut waves = update.waves.iter().peekable();
            while let Some(wave) = waves.next() {
                if let Some(next) = waves.peek() {
                    ensure!(wave.1 < next.1, error::WavesUnordered);
                }
            }
        }
        Ok(())
    }

    /// Returns Updates matching variant, arch, and version
    fn get_matching_updates(
        &mut self,
        variant: String,
        arch: String,
        image_version: Version,
    ) -> Vec<&mut Update> {
        self.updates
            .iter_mut()
            // Find the update that exactly matches the specified update
            .filter(|update| {
                update.arch == arch && update.variant == variant && update.version == image_version
            })
            .collect()
    }

    /// Adds a vec of waves to update, returns number of matching updates for wave
    // Wave format in `manifest.json` is slightly different from the wave structs
    // provided to this function. For example, if two `UpdateWave` structs are
    // passed to this function:
    // [
    //   UpdateWave { start_after: "1 hour", fleet_percentage: 1 },
    //   UpdateWave { start_after: "1 day", fleet_percentage: 100},
    // ]
    //
    // The resulting `waves` section of the applicable update looks like:
    // waves: {
    //   "0": "<UTC datetime of 1 hour from now>",
    //   "20": "<UTC datetime of 1 day from now>"
    // }
    //
    // This might look odd until you understand that the first wave begins
    // at the time specified, and includes seeds 0-19, or 1%, of the seeds
    // available (`MAX_SEED` in this file). The next wave begins at the time
    // specified and includes seeds 20-MAX_SEED, or 100% of the rest of the
    // seeds available. We do this so that the waves input can be more
    // understandable for human operators, with times relative to when they
    // start a release, but still have absolute times and seeds that are more
    // understandable in our update code.
    pub fn set_waves(
        &mut self,
        variant: String,
        arch: String,
        image_version: Version,
        start_at: DateTime<Utc>,
        waves: &UpdateWaves,
    ) -> Result<usize> {
        let matching = self.get_matching_updates(variant, arch, image_version);
        let num_matching = matching.len();

        for update in matching {
            update.waves.clear();

            // The first wave has a 0 seed
            let mut seed = 0;
            for wave in &waves.waves {
                ensure!(
                    wave.fleet_percentage > 0 && wave.fleet_percentage <= 100,
                    error::InvalidFleetPercentage {
                        provided: wave.fleet_percentage
                    }
                );

                let offset = parse_offset(&wave.start_after).context(error::BadOffset {
                    offset: &wave.start_after,
                })?;
                update.waves.insert(seed, start_at + offset);

                // Get the appropriate seed from the percentage given
                // First get the percentage as a decimal,
                let percent = wave.fleet_percentage as f32 / 100 as f32;
                // then, get seed from the percentage of MAX_SEED as a u32
                seed = (percent * MAX_SEED as f32) as u32;
            }
        }
        Self::validate_updates(&self.updates)?;
        Ok(num_matching)
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

pub fn find_migrations(from: &Version, to: &Version, manifest: &Manifest) -> Result<Vec<String>> {
    // early exit if there is no work to do.
    if from == to {
        return Ok(Vec::new());
    }
    // express the versions in ascending order
    let (lower, higher, is_reversed) = match from.cmp(to) {
        Ordering::Less | Ordering::Equal => (from, to, false),
        Ordering::Greater => (to, from, true),
    };
    let mut migrations = find_migrations_forward(&lower, &higher, manifest)?;
    // if the direction is backward, reverse the order of the migration list.
    if is_reversed {
        migrations = migrations.into_iter().rev().collect();
    }
    Ok(migrations)
}

/// Finds the migration from one version to another. The migration direction must be forward, that
/// is, `from` must be less than or equal to `to`. The caller may reverse the Vec returned by this
/// function to migrate backward.
fn find_migrations_forward(
    from: &Version,
    to: &Version,
    manifest: &Manifest,
) -> Result<Vec<String>> {
    let mut targets = Vec::new();
    let mut version = from;
    while version != to {
        let mut migrations: Vec<&(Version, Version)> = manifest
            .migrations
            .keys()
            .filter(|(f, t)| *f == *version && *t <= *to)
            .collect();

        // There can be multiple paths to the same target, e.g.
        //      (1.0.0, 1.1.0) => [...]
        //      (1.0.0, 1.2.0) => [...]
        // Choose one with the highest *to* version, <= our target
        migrations.sort_unstable_by(|(_, a), (_, b)| b.cmp(&a));
        if let Some(transition) = migrations.first() {
            // If a transition doesn't require a migration the array will be empty
            if let Some(migrations) = manifest.migrations.get(transition) {
                targets.extend_from_slice(&migrations);
            }
            version = &transition.1;
        } else {
            return error::MissingMigration {
                current: version.clone(),
                target: to.clone(),
            }
            .fail();
        }
    }
    Ok(targets)
}

pub fn load_manifest<T: tough::Transport>(repository: &tough::Repository<T>) -> Result<Manifest> {
    let target = "manifest.json";
    serde_json::from_reader(
        repository
            .read_target(target)
            .context(error::ManifestLoad)?
            .context(error::ManifestNotFound)?,
    )
    .context(error::ManifestParse)
}

#[test]
fn test_migrations_forward() {
    // A manifest with four migration tuples starting at 1.0 and ending at 1.3.
    // There is a shortcut from 1.1 to 1.3, skipping 1.2
    let path = "./tests/data/migrations.json";
    let manifest: Manifest = serde_json::from_reader(File::open(path).unwrap()).unwrap();
    let from = Version::parse("1.0.0").unwrap();
    let to = Version::parse("1.5.0").unwrap();
    let targets = find_migrations(&from, &to, &manifest).unwrap();

    assert!(targets.len() == 3);
    let mut i = targets.iter();
    assert!(i.next().unwrap() == "migration_1.1.0_a");
    assert!(i.next().unwrap() == "migration_1.1.0_b");
    assert!(i.next().unwrap() == "migration_1.5.0_shortcut");
}

#[test]
fn test_migrations_backward() {
    // The same manifest as `test_migrations_forward` but this time we will migrate backward.
    let path = "./tests/data/migrations.json";
    let manifest: Manifest = serde_json::from_reader(File::open(path).unwrap()).unwrap();
    let from = Version::parse("1.5.0").unwrap();
    let to = Version::parse("1.0.0").unwrap();
    let targets = find_migrations(&from, &to, &manifest).unwrap();

    assert!(targets.len() == 3);
    let mut i = targets.iter();
    assert!(i.next().unwrap() == "migration_1.5.0_shortcut");
    assert!(i.next().unwrap() == "migration_1.1.0_b");
    assert!(i.next().unwrap() == "migration_1.1.0_a");
}
