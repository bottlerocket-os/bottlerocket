#![deny(rust_2018_idioms)]
#![warn(clippy::pedantic)]

#[path = "../error.rs"]
mod error;

#[macro_use]
extern crate log;

use crate::error::Result;
use chrono::{DateTime, Utc};
use data_store_version::Version as DataVersion;
use migrator::MIGRATION_FILENAME_RE;
use semver::Version as SemVer;
use simplelog::{Config as LogConfig, LevelFilter, TermLogger, TerminalMode};
use snafu::{ensure, ErrorCompat, OptionExt, ResultExt};
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use structopt::StructOpt;
use update_metadata::{Images, Manifest, Update};

#[derive(Debug, StructOpt)]
struct GeneralArgs {
    // metadata file to create/modify
    file: PathBuf,
}

#[derive(Debug, StructOpt)]
struct AddUpdateArgs {
    // metadata file to create/modify
    file: PathBuf,

    // image 'flavor', eg. 'aws-k8s'
    #[structopt(short = "f", long = "flavor")]
    flavor: String,

    // image version
    #[structopt(short = "v", long = "version")]
    image_version: SemVer,

    // architecture image is built for
    #[structopt(short = "a", long = "arch")]
    arch: String,

    // corresponding datastore version for this image
    #[structopt(short = "d", long = "data-version")]
    datastore_version: DataVersion,

    // maximum valid version
    #[structopt(short = "m", long = "max-version")]
    max_version: Option<SemVer>,

    // root image target name
    #[structopt(short = "r", long = "root")]
    root: String,

    // boot image target name
    #[structopt(short = "b", long = "boot")]
    boot: String,

    // verity "hash" image target name
    #[structopt(short = "h", long = "hash")]
    hash: String,
}

impl AddUpdateArgs {
    fn run(self) -> Result<()> {
        let mut manifest: Manifest = match load_file(&self.file) {
            Ok(m) => m,
            _ => Manifest::default(), // TODO only if EEXIST
        };

        let max_version = if let Some(version) = self.max_version {
            version
        } else {
            // Default to greater of the current max version and this version
            if let Some(update) = manifest.updates.first() {
                std::cmp::max(&self.image_version, &update.max_version).clone()
            } else {
                self.image_version.clone()
            }
        };
        let update = Update {
            flavor: self.flavor,
            arch: self.arch,
            version: self.image_version.clone(),
            max_version,
            images: Images {
                root: self.root,
                boot: self.boot,
                hash: self.hash,
            },
            waves: BTreeMap::new(),
        };
        manifest
            .datastore_versions
            .insert(self.image_version, self.datastore_version);
        update_max_version(
            &mut manifest,
            &update.max_version,
            Some(&update.arch),
            Some(&update.flavor),
        );
        info!("Maximum version set to {}", &update.max_version);
        manifest.updates.push(update);
        write_file(&self.file, &manifest)
    }
}

#[derive(Debug, StructOpt)]
struct RemoveUpdateArgs {
    // metadata file to create/modify
    file: PathBuf,

    // image 'flavor', eg. 'aws-k8s'
    #[structopt(short = "l", long = "flavor")]
    flavor: String,

    // image version
    #[structopt(short = "v", long = "version")]
    image_version: SemVer,

    // architecture image is built for
    #[structopt(short = "a", long = "arch")]
    arch: String,

    // Whether to clean up datastore mappings that no longer reference an
    // existing update. Migration paths for such datastore versions are
    // preserved.
    // This should _only_ be used if there are no existing users of the
    // specified Thar image version.
    #[structopt(short, long)]
    cleanup: bool,
}

impl RemoveUpdateArgs {
    fn run(&self) -> Result<()> {
        let mut manifest: Manifest = load_file(&self.file)?;
        // Remove any update that exactly matches the specified update
        manifest.updates.retain(|update| {
            update.arch != self.arch
                || update.flavor != self.flavor
                || update.version != self.image_version
        });
        if self.cleanup {
            let remaining: Vec<&Update> = manifest
                .updates
                .iter()
                .filter(|update| update.version == self.image_version)
                .collect();
            if remaining.is_empty() {
                manifest.datastore_versions.remove(&self.image_version);
            } else {
                info!(
                    "Cleanup skipped; {} {} updates remain",
                    remaining.len(),
                    self.image_version
                );
            }
        }
        // Note: We don't revert the maximum version on removal
        write_file(&self.file, &manifest)?;
        if let Some(current) = manifest.updates.first() {
            info!(
                "Update {}-{}-{} removed. Current maximum version: {}",
                self.arch, self.flavor, self.image_version, current.version
            );
        } else {
            info!(
                "Update {}-{}-{} removed. No remaining updates",
                self.arch, self.flavor, self.image_version
            );
        }
        Ok(())
    }
}

#[derive(Debug, StructOpt)]
struct WaveArgs {
    // metadata file to create/modify
    file: PathBuf,

    // image 'flavor', eg. 'aws-k8s'
    #[structopt(short = "l", long = "flavor")]
    flavor: String,

    // image version
    #[structopt(short = "v", long = "version")]
    image_version: SemVer,

    // architecture image is built for
    #[structopt(short = "a", long = "arch")]
    arch: String,

    // start bound id for this wave (0 <= x < 2048)
    #[structopt(short = "b", long = "bound-id")]
    bound: u32,

    // start time for this wave
    #[structopt(short = "s", long = "start-time")]
    start: Option<DateTime<Utc>>,
}

impl WaveArgs {
    fn validate(updates: &[Update]) -> Result<()> {
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

    fn add(self) -> Result<()> {
        let mut manifest: Manifest = load_file(&self.file)?;
        let matching: Vec<&mut Update> = manifest
            .updates
            .iter_mut()
            // Find the update that exactly matches the specified update
            .filter(|update| {
                update.arch == self.arch
                    && update.flavor == self.flavor
                    && update.version == self.image_version
            })
            .collect();
        if matching.len() > 1 {
            warn!("Multiple matching updates for wave - this is weird but not a disaster");
        }
        let start = self.start.context(error::WaveStartArg)?;
        for update in matching {
            update.waves.insert(self.bound, start);
        }
        Self::validate(&manifest.updates)?;
        write_file(&self.file, &manifest)
    }

    fn remove(self) -> Result<()> {
        let mut manifest: Manifest = load_file(&self.file)?;
        let matching: Vec<&mut Update> = manifest
            .updates
            .iter_mut()
            .filter(|update| {
                update.arch == self.arch
                    && update.flavor == self.flavor
                    && update.version == self.image_version
            })
            .collect();
        for update in matching {
            update.waves.remove(&self.bound);
        }
        write_file(&self.file, &manifest)
    }
}

#[derive(Debug, StructOpt)]
struct MigrationArgs {
    // metadata file to create/modify
    file: PathBuf,

    // starting datastore version
    #[structopt(short = "f", long = "from")]
    from: DataVersion,

    // target datastore version
    #[structopt(short = "t", long = "to")]
    to: DataVersion,

    // whether to append to or replace any existing migration list
    #[structopt(short, long)]
    append: bool,

    // migration names
    migrations: Vec<String>,
}

impl MigrationArgs {
    fn add(self) -> Result<()> {
        let mut manifest: Manifest = load_file(&self.file)?;

        // Check each migration matches the filename conventions used by the migrator
        for name in &self.migrations {
            let captures = MIGRATION_FILENAME_RE
                .captures(&name)
                .context(error::MigrationNaming)?;

            let version_match = captures
                .name("version")
                .context(error::BadRegexVersion { name })?;
            let version = DataVersion::from_str(version_match.as_str())
                .context(error::BadDataVersion { key: name })?;
            if version != self.to {
                return error::MigrationInvalidTarget {
                    name,
                    to: self.to,
                    version,
                }
                .fail();
            }

            let _ = captures
                .name("name")
                .context(error::BadRegexName { name })?;
        }

        // If --append is set, append the new migrations to the existing vec.
        if self.append && manifest.migrations.contains_key(&(self.from, self.to)) {
            let migrations = manifest.migrations.get_mut(&(self.from, self.to)).context(
                error::MigrationMutable {
                    from: self.from,
                    to: self.to,
                },
            )?;
            migrations.extend_from_slice(&self.migrations);
        // Otherwise just overwrite the existing migrations
        } else {
            manifest
                .migrations
                .insert((self.from, self.to), self.migrations);
        }
        write_file(&self.file, &manifest)
    }

    fn remove(self) -> Result<()> {
        let mut manifest: Manifest = load_file(&self.file)?;
        ensure!(
            manifest.migrations.contains_key(&(self.from, self.to)),
            error::MigrationNotPresent {
                from: self.from,
                to: self.to,
            }
        );
        manifest.migrations.remove(&(self.from, self.to));
        write_file(&self.file, &manifest)
    }
}

#[derive(Debug, StructOpt)]
struct MaxVersionArgs {
    // metadata file to create/modify
    file: PathBuf,

    // maximum valid version
    #[structopt(short, long)]
    max_version: SemVer,
}

impl MaxVersionArgs {
    fn run(self) -> Result<()> {
        let mut manifest: Manifest = load_file(&self.file)?;
        update_max_version(&mut manifest, &self.max_version, None, None);
        write_file(&self.file, &manifest)
    }
}

#[derive(Debug, StructOpt)]
struct MappingArgs {
    // metadata file to create/modify
    file: PathBuf,

    #[structopt(short, long)]
    image_version: SemVer,

    #[structopt(short, long)]
    data_version: DataVersion,
}

impl MappingArgs {
    fn run(self) -> Result<()> {
        let mut manifest: Manifest = load_file(&self.file)?;
        let version = self.image_version.clone();
        let old = manifest
            .datastore_versions
            .insert(self.image_version, self.data_version);
        if let Some(old) = old {
            warn!(
                "Warning: New mapping ({},{}) replaced old mapping ({},{})",
                version, self.data_version, version, old
            );
        }
        write_file(&self.file, &manifest)
    }
}

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
enum Command {
    /// Create an empty manifest
    Init(GeneralArgs),
    /// Add a new update to the manifest, not including wave information
    AddUpdate(AddUpdateArgs),
    /// Add a (bound_id, time) wave to an existing update
    AddWave(WaveArgs),
    /// Add one or more migrations to a (from, to) datastore mapping
    AddMigration(MigrationArgs),
    /// Add a image_version:data_store_version mapping to the manifest
    AddVersionMapping(MappingArgs),
    /// Set the global maximum image version
    SetMaxVersion(MaxVersionArgs),
    /// Remove an update from the manifest, including wave information
    RemoveUpdate(RemoveUpdateArgs),
    /// Remove all migrations for a (from, to) datastore mapping
    RemoveMigrations(MigrationArgs),
    /// Remove a (bound_id, time) wave from an update
    RemoveWave(WaveArgs),
    /// Validate a manifest file, but make no changes
    Validate(GeneralArgs),
}

fn load_file(path: &Path) -> Result<Manifest> {
    let file = File::open(path).context(error::ManifestRead { path })?;
    serde_json::from_reader(file).context(error::ManifestParse)
}

fn write_file(path: &Path, manifest: &Manifest) -> Result<()> {
    let manifest = serde_json::to_string_pretty(&manifest).context(error::UpdateSerialize)?;
    fs::write(path, &manifest).context(error::ConfigWrite { path })?;
    Ok(())
}

/// Update the maximum version for all updates that optionally match the
/// architecture and flavor of some new update.
fn update_max_version(
    manifest: &mut Manifest,
    version: &SemVer,
    arch: Option<&str>,
    flavor: Option<&str>,
) {
    let matching: Vec<&mut Update> = manifest
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

fn main_inner() -> Result<()> {
    // TerminalMode::Mixed will send errors to stderr and anything less to stdout.
    TermLogger::init(LevelFilter::Info, LogConfig::default(), TerminalMode::Mixed)
        .context(error::Logger)?;

    match Command::from_args() {
        Command::Init(args) => write_file(&args.file, &Manifest::default()),
        Command::AddUpdate(args) => args.run(),
        Command::AddWave(args) => args.add(),
        Command::AddMigration(args) => args.add(),
        Command::AddVersionMapping(args) => args.run(),
        Command::SetMaxVersion(args) => args.run(),
        Command::RemoveUpdate(args) => args.run(),
        Command::RemoveWave(args) => args.remove(),
        Command::RemoveMigrations(args) => args.remove(),
        Command::Validate(args) => match load_file(&args.file) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        },
    }
}

fn main() -> ! {
    std::process::exit(match main_inner() {
        Ok(()) => 0,
        Err(err) => {
            error!("{}", err);
            if let Some(var) = std::env::var_os("RUST_BACKTRACE") {
                if var != "0" {
                    if let Some(backtrace) = err.backtrace() {
                        error!("\n{:?}", backtrace);
                    }
                }
            }
            1
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use std::str::FromStr;
    use tempfile::NamedTempFile;

    #[test]
    fn max_versions() -> Result<()> {
        let tmpfd = NamedTempFile::new().context(error::TmpFileCreate)?;
        AddUpdateArgs {
            file: PathBuf::from(tmpfd.path()),
            flavor: String::from("yum"),
            arch: String::from("x86_64"),
            image_version: SemVer::parse("1.2.3").unwrap(),
            max_version: Some(SemVer::parse("1.2.3").unwrap()),
            datastore_version: DataVersion::from_str("1.0").unwrap(),
            boot: String::from("boot"),
            root: String::from("root"),
            hash: String::from("hash"),
        }
        .run()
        .unwrap();
        AddUpdateArgs {
            file: PathBuf::from(tmpfd.path()),
            flavor: String::from("yum"),
            arch: String::from("x86_64"),
            image_version: SemVer::parse("1.2.5").unwrap(),
            max_version: Some(SemVer::parse("1.2.3").unwrap()),
            datastore_version: DataVersion::from_str("1.0").unwrap(),
            boot: String::from("boot"),
            root: String::from("root"),
            hash: String::from("hash"),
        }
        .run()
        .unwrap();
        AddUpdateArgs {
            file: PathBuf::from(tmpfd.path()),
            flavor: String::from("yum"),
            arch: String::from("x86_64"),
            image_version: SemVer::parse("1.2.4").unwrap(),
            max_version: Some(SemVer::parse("1.2.4").unwrap()),
            datastore_version: DataVersion::from_str("1.0").unwrap(),
            boot: String::from("boot"),
            root: String::from("root"),
            hash: String::from("hash"),
        }
        .run()
        .unwrap();

        let m: Manifest = load_file(tmpfd.path())?;
        for u in m.updates {
            assert!(u.max_version == SemVer::parse("1.2.4").unwrap());
        }
        Ok(())
    }

    #[test]
    fn datastore_mapping() -> Result<()> {
        let tmpfd = NamedTempFile::new().context(error::TmpFileCreate)?;
        AddUpdateArgs {
            file: PathBuf::from(tmpfd.path()),
            flavor: String::from("yum"),
            arch: String::from("x86_64"),
            image_version: SemVer::parse("1.2.3").unwrap(),
            max_version: Some(SemVer::parse("1.2.3").unwrap()),
            datastore_version: DataVersion::from_str("1.0").unwrap(),
            boot: String::from("boot"),
            root: String::from("root"),
            hash: String::from("hash"),
        }
        .run()
        .unwrap();
        AddUpdateArgs {
            file: PathBuf::from(tmpfd.path()),
            flavor: String::from("yum"),
            arch: String::from("x86_64"),
            image_version: SemVer::parse("1.2.5").unwrap(),
            max_version: Some(SemVer::parse("1.2.3").unwrap()),
            datastore_version: DataVersion::from_str("1.1").unwrap(),
            boot: String::from("boot"),
            root: String::from("root"),
            hash: String::from("hash"),
        }
        .run()
        .unwrap();
        AddUpdateArgs {
            file: PathBuf::from(tmpfd.path()),
            flavor: String::from("yum"),
            arch: String::from("x86_64"),
            image_version: SemVer::parse("1.2.4").unwrap(),
            max_version: Some(SemVer::parse("1.2.4").unwrap()),
            datastore_version: DataVersion::from_str("1.0").unwrap(),
            boot: String::from("boot"),
            root: String::from("root"),
            hash: String::from("hash"),
        }
        .run()
        .unwrap();

        // TODO this needs to test against ARCH and FLAVOR not being considered
        RemoveUpdateArgs {
            file: PathBuf::from(tmpfd.path()),
            flavor: String::from("yum"),
            arch: String::from("x86_64"),
            image_version: SemVer::parse("1.2.4").unwrap(),
            cleanup: true,
        }
        .run()
        .unwrap();

        let m: Manifest = load_file(tmpfd.path())?;
        assert!(m
            .datastore_versions
            .contains_key(&SemVer::parse("1.2.3").unwrap()));
        Ok(())
    }

    #[test]
    fn ordered_waves() -> Result<()> {
        let tmpfd = NamedTempFile::new().context(error::TmpFileCreate)?;
        AddUpdateArgs {
            file: PathBuf::from(tmpfd.path()),
            flavor: String::from("yum"),
            arch: String::from("x86_64"),
            image_version: SemVer::parse("1.2.3").unwrap(),
            max_version: Some(SemVer::parse("1.2.3").unwrap()),
            datastore_version: DataVersion::from_str("1.0").unwrap(),
            boot: String::from("boot"),
            root: String::from("root"),
            hash: String::from("hash"),
        }
        .run()
        .unwrap();

        WaveArgs {
            file: PathBuf::from(tmpfd.path()),
            flavor: String::from("yum"),
            arch: String::from("x86_64"),
            image_version: SemVer::parse("1.2.3").unwrap(),
            bound: 1024,
            start: Some(Utc::now()),
        }
        .add()
        .unwrap();

        assert!(WaveArgs {
            file: PathBuf::from(tmpfd.path()),
            flavor: String::from("yum"),
            arch: String::from("x86_64"),
            image_version: SemVer::parse("1.2.3").unwrap(),
            bound: 1536,
            start: Some(Utc::now() - Duration::hours(1)),
        }
        .add()
        .is_err());

        Ok(())
    }
}
