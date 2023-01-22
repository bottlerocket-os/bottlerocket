mod de;
pub mod error;
mod se;

use crate::error::Result;
use chrono::{DateTime, Utc};
use parse_datetime::parse_offset;
use semver::Version;
use serde::{Deserialize, Serialize};
use snafu::{ensure, ResultExt};
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::ops::Bound::{Excluded, Included};
use std::path::Path;

pub const MAX_SEED: u32 = 2048;

#[derive(Debug, PartialEq, Eq)]
pub enum Wave {
    Initial {
        end_time: DateTime<Utc>,
        end_seed: u32,
    },
    General {
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        start_seed: u32,
        end_seed: u32,
    },
    Last {
        start_time: DateTime<Utc>,
        start_seed: u32,
    },
}

impl Wave {
    pub fn has_started(&self, time: DateTime<Utc>) -> bool {
        match self {
            Self::Initial { .. } => true,
            Self::General { start_time, .. } | Self::Last { start_time, .. } => *start_time <= time,
        }
    }

    pub fn has_passed(&self, time: DateTime<Utc>) -> bool {
        match self {
            Self::General { end_time, .. } | Self::Initial { end_time, .. } => *end_time <= time,
            Self::Last { start_time, .. } => *start_time <= time,
        }
    }
}

/// `UpdateWaves` is provided for the specific purpose of deserializing
/// update waves from TOML files
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateWaves {
    pub waves: Vec<UpdateWave>,
}

impl UpdateWaves {
    /// Deserializes an `UpdateWaves` from a given path
    pub fn from_path<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let wave_data = fs::read_to_string(path).context(error::FileReadSnafu { path })?;
        toml::from_str(&wave_data).context(error::InvalidTomlSnafu { path })
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
        let release_data = fs::read_to_string(path).context(error::FileReadSnafu { path })?;
        toml::from_str(&release_data).context(error::InvalidTomlSnafu { path })
    }
}

pub fn load_file(path: &Path) -> Result<Manifest> {
    let file = File::open(path).context(error::FileReadSnafu { path })?;
    serde_json::from_reader(file).context(error::ManifestParseSnafu)
}

pub fn write_file(path: &Path, manifest: &Manifest) -> Result<()> {
    let manifest = serde_json::to_string_pretty(&manifest).context(error::UpdateSerializeSnafu)?;
    fs::write(path, manifest).context(error::FileWriteSnafu { path })?;
    Ok(())
}

impl Manifest {
    /// Parses a `Manifest` from JSON, which is presented by a `Read` object.
    pub fn from_json<R: Read>(r: R) -> Result<Self> {
        serde_json::from_reader(r).context(error::ManifestParseSnafu)
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
            version: image_version,
            max_version,
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
                    ensure!(wave.1 < next.1, error::WavesUnorderedSnafu);
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
                    error::InvalidFleetPercentageSnafu {
                        provided: wave.fleet_percentage
                    }
                );

                let offset = parse_offset(&wave.start_after).context(error::BadOffsetSnafu {
                    offset: &wave.start_after,
                })?;
                update.waves.insert(seed, start_at + offset);

                // Get the appropriate seed from the percentage given
                // First get the percentage as a decimal,
                let percent = wave.fleet_percentage as f32 / 100_f32;
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
    /// - Some wave described by a start and end time, and the starting seed and ending seed.
    /// - The "0th" wave, which has an "end" time but no specified start time, and the ending seed.
    /// - The last wave, which has a start time but no specified end time, and the starting seed.
    /// - Nothing, if no waves are configured.
    #[must_use]
    pub fn update_wave(&self, seed: u32) -> Option<Wave> {
        let start_wave = self
            .waves
            .range((Included(0), Excluded(seed)))
            .map(|(k, v)| (*k, *v))
            .last();
        let end_wave = self
            .waves
            .range((Included(seed), Included(MAX_SEED)))
            .map(|(k, v)| (*k, *v))
            .next();

        match (start_wave, end_wave) {
            // Note that the key for each wave entry is the starting seed for that wave, the value is the DateTime
            (None, Some((end_seed, end_time))) => Some(Wave::Initial { end_seed, end_time }),
            (Some((start_seed, start_time)), Some((end_seed, end_time))) => Some(Wave::General {
                start_time,
                end_time,
                start_seed,
                end_seed,
            }),
            (Some((start_seed, start_time)), None) => Some(Wave::Last {
                start_time,
                start_seed,
            }),
            _ => None,
        }
    }

    /// Returns whether the update is available. An update is said to be 'ready/available' if the wave
    /// this host belongs to has fully passed, or if the host's position in the wave has passed, or
    /// if there are no waves.
    /// The position of the host within the wave is determined by the seed value.
    #[must_use]
    pub fn update_ready(&self, seed: u32, time: DateTime<Utc>) -> bool {
        // If this host is part of some update wave
        if let Some(wave) = self.update_wave(seed) {
            // If the wave has passed, the update is available (this includes passing the last wave start time)
            if wave.has_passed(time) {
                return true;
            } else if !wave.has_started(time) {
                return false;
            }
            let bound = match wave {
                // Hosts should not wind up in the special "initial" wave with no start time, but if they do,
                // we consider the update as being available immediately.
                Wave::Initial { .. } => None,
                Wave::General {
                    start_time,
                    end_time,
                    start_seed,
                    end_seed,
                } => Some((start_time, Some(end_time), start_seed, end_seed)),
                // Last wave has no end time nor end seed; Let end seed be `MAX_SEED` since all the
                // remaining hosts are in this last wave
                Wave::Last {
                    start_time,
                    start_seed,
                } => Some((start_time, None, start_seed, MAX_SEED)),
            };
            if let Some((start_time, Some(end_time), start_seed, end_seed)) = bound {
                // This host is not part of last wave
                // Determine the duration of this host's wave
                let wave_duration = end_time - start_time;
                let num_seeds_allocated_to_wave = (end_seed - start_seed) as i32;
                if num_seeds_allocated_to_wave == 0 {
                    // Empty wave, no host should have been allocated to it
                    return true;
                }
                let time_per_seed = wave_duration / num_seeds_allocated_to_wave;
                // Derive the target time position within the wave given the host's seed.
                let target_time = start_time + (time_per_seed * (seed as i32));
                // If the current time is past the target time position in the wave, the update is
                // marked available
                return time >= target_time;
            }
        }
        // There are no waves, so we consider the update available
        true
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
    let mut migrations = find_migrations_forward(lower, higher, manifest)?;
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
        migrations.sort_unstable_by(|(_, a), (_, b)| b.cmp(a));
        if let Some(transition) = migrations.first() {
            // If a transition doesn't require a migration the array will be empty
            if let Some(migrations) = manifest.migrations.get(transition) {
                targets.extend_from_slice(migrations);
            }
            version = &transition.1;
        } else {
            return error::MissingMigrationSnafu {
                current: version.clone(),
                target: to.clone(),
            }
            .fail();
        }
    }
    Ok(targets)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Duration, NaiveDate, Utc};

    fn test_time() -> DateTime<Utc> {
        // DateTime for 1/1/2000 00:00:00
        DateTime::<Utc>::from_utc(
            NaiveDate::from_ymd_opt(2000, 1, 1)
                .unwrap()
                .and_hms_milli_opt(0, 0, 0, 0)
                .unwrap(),
            Utc,
        )
    }

    fn test_update() -> Update {
        Update {
            variant: "bottlerocket".to_string(),
            arch: "test".to_string(),
            version: Version::parse("1.1.1").unwrap(),
            max_version: Version::parse("1.1.1").unwrap(),
            waves: BTreeMap::new(),
            images: Images {
                boot: String::from("boot"),
                root: String::from("root"),
                hash: String::from("hash"),
            },
        }
    }

    #[test]
    fn test_update_ready_no_wave() {
        let time = test_time();
        let seed = 100;
        let update = test_update();
        assert!(
            update.update_ready(seed, time),
            "no waves specified, update should be ready"
        );
    }

    #[test]
    fn test_update_ready_single_wave() {
        let time = test_time();
        let mut update = test_update();
        // One single wave (0th wave does not count) for every update that spans over 2048 millisecond,
        // Each seed will be mapped to a single millisecond within this wave,
        // e.g. seed 1 -> update is ready 1 millisecond past start of wave
        // seed 500 -> update is ready 500 millisecond past start of wave, etc
        update.waves.insert(0, time);
        update
            .waves
            .insert(MAX_SEED, time + Duration::milliseconds(i64::from(MAX_SEED)));

        for seed in (100..500).step_by(100) {
            assert!(
                !update.update_ready(seed, time + Duration::milliseconds(i64::from(seed) - 1)),
                "seed: {}, time: {}, wave start time: {}, wave start seed: {}, {} milliseconds hasn't passed yet",
                seed,
                time,
                time,
                0,
                seed
            );
            assert!(
                update.update_ready(seed, time + Duration::milliseconds(i64::from(seed))),
                "seed: {}, time: {}, wave start time: {}, wave start seed: {}, update should be ready",
                seed,
                time + Duration::milliseconds(100),
                time,
                0,
            );
        }
    }

    fn add_test_waves(update: &mut Update) {
        let time = test_time();
        update.waves.insert(0, time);
        // First wave ends 200 milliseconds into the update and has seeds 0 - 50
        update.waves.insert(50, time + Duration::milliseconds(200));
        // Second wave ends 1024 milliseconds into the update and has seeds 50 - 100
        update
            .waves
            .insert(100, time + Duration::milliseconds(1024));
        // Third wave ends 4096 milliseconds into the update and has seeds 100 - 1024
        update
            .waves
            .insert(1024, time + Duration::milliseconds(4096));
    }

    #[test]
    fn test_update_ready_second_wave() {
        let time = test_time();
        let mut update = test_update();
        add_test_waves(&mut update);
        // Now we should be in the second wave
        let seed = 60;

        for duration in (0..200).step_by(10) {
            assert!(
                !update.update_ready(seed, time + Duration::milliseconds(duration)),
                "seed should not part of first wave",
            );
        }

        let seed_time_position = (1024 - 200) / (100 - 50) * seed;
        for duration in (200..seed_time_position).step_by(2) {
            assert!(
                !update.update_ready(
                    seed, time + Duration::milliseconds(i64::from(duration))
                ),
                "update should not be ready, it's the second wave but not at position within wave yet: {}", duration,
            );
        }

        for duration in (seed_time_position..1024).step_by(4) {
            assert!(
                update.update_ready(
                    seed,
                    time + Duration::milliseconds(200)
                        + Duration::milliseconds(i64::from(duration))
                ),
                "update should be ready now that we're passed the allocated time position within the second wave: {}", duration,
            );
        }

        for duration in (1024..4096).step_by(8) {
            assert!(
                update.update_ready(seed, time + Duration::milliseconds(i64::from(duration))),
                "update should be ready after the third wave starts and onwards",
            );
        }
    }

    #[test]
    fn test_update_ready_third_wave() {
        let time = test_time();
        let mut update = test_update();
        add_test_waves(&mut update);
        let seed = 148;

        for duration in (0..200).step_by(10) {
            assert!(
                !update.update_ready(seed, time + Duration::milliseconds(duration)),
                "seed should not part of first wave",
            );
        }

        for duration in (200..1024).step_by(4) {
            assert!(
                !update.update_ready(seed, time + Duration::milliseconds(duration)),
                "seed should not part of second wave",
            );
        }

        let seed_time_position = (4096 - 1024) / (1024 - 100) * seed;
        for duration in (1024..seed_time_position).step_by(4) {
            assert!(
                !update.update_ready(
                    seed,
                    time + Duration::milliseconds(200)
                        + Duration::milliseconds(i64::from(duration))
                ),
                "update should not be ready, it's the third wave but not at position within wave yet: {}", duration,
            );
        }

        for duration in (seed_time_position..4096).step_by(4) {
            assert!(
                update.update_ready(
                    seed,
                    time + Duration::milliseconds(1024 + 200)
                        + Duration::milliseconds(i64::from(duration))
                ),
                "update should be ready now that we're passed the allocated time position within the third wave: {}", duration,
            );
        }
    }

    #[test]
    fn test_update_ready_final_wave() {
        let mut update = Update {
            variant: String::from("bottlerocket"),
            arch: String::from("test"),
            version: Version::parse("1.0.0").unwrap(),
            max_version: Version::parse("1.1.0").unwrap(),
            waves: BTreeMap::new(),
            images: Images {
                boot: String::from("boot"),
                root: String::from("root"),
                hash: String::from("hash"),
            },
        };
        let seed = 1024;
        // Construct a DateTime object for 1/1/2000 00:00:00
        let time = DateTime::<Utc>::from_utc(
            NaiveDate::from_ymd_opt(2000, 1, 1)
                .unwrap()
                .and_hms_milli_opt(0, 0, 0, 0)
                .unwrap(),
            Utc,
        );

        update.waves.insert(0, time - Duration::hours(3));
        update.waves.insert(256, time - Duration::hours(2));
        update.waves.insert(512, time - Duration::hours(1));

        assert!(
            // Last wave should have already passed
            update.update_ready(seed, time),
            "update should be ready"
        );
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
}
