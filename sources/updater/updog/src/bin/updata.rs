#![warn(clippy::pedantic)]

#[path = "../error.rs"]
mod error;

#[macro_use]
extern crate log;

use crate::error::Result;

use argh::FromArgs;
use chrono::{DateTime, Utc};
use semver::Version;
use simplelog::{Config as LogConfig, LevelFilter, SimpleLogger};
use snafu::{ErrorCompat, OptionExt, ResultExt};
use std::fs;
use std::path::PathBuf;
use update_metadata::{Images, Manifest, Release, UpdateWaves};

/// Create an empty manifest
#[derive(Debug, FromArgs)]
#[argh(subcommand, name = "init")]
struct InitArgs {
    /// metadata file to create/modify
    #[argh(positional)]
    file: PathBuf,
}

/// Validate a manifest file, but make no changes
#[derive(Debug, FromArgs)]
#[argh(subcommand, name = "validate")]
struct ValidateArgs {
    /// metadata file to create/modify
    #[argh(positional)]
    file: PathBuf,
}

/// Add a new update to the manifest, not including wave information
#[derive(Debug, FromArgs)]
#[argh(subcommand, name = "add-update")]
struct AddUpdateArgs {
    /// metadata file to create/modify
    #[argh(positional)]
    file: PathBuf,

    /// image 'variant', eg. 'aws-ecs-1'
    #[argh(option, short = 'f', long = "variant")]
    variant: String,

    /// image version
    #[argh(option, short = 'v', long = "version")]
    image_version: Version,

    /// architecture image is built for
    #[argh(option, short = 'a', long = "arch")]
    arch: String,

    /// maximum valid version
    #[argh(option, short = 'm', long = "max-version")]
    max_version: Option<Version>,

    /// root image target name
    #[argh(option, short = 'r', long = "root")]
    root: String,

    /// boot image target name
    #[argh(option, short = 'b', long = "boot")]
    boot: String,

    /// verity "hash" image target name
    #[argh(option, short = 'h', long = "hash")]
    hash: String,
}

impl AddUpdateArgs {
    fn run(self) -> Result<()> {
        let mut manifest: Manifest = update_metadata::load_file(&self.file)?;
        manifest.add_update(
            self.image_version,
            self.max_version,
            self.arch,
            self.variant,
            Images {
                root: self.root,
                boot: self.boot,
                hash: self.hash,
            },
        )?;
        update_metadata::write_file(&self.file, &manifest)?;
        Ok(())
    }
}

/// Remove an update from the manifest, including wave information
#[derive(Debug, FromArgs)]
#[argh(subcommand, name = "remove-update")]
struct RemoveUpdateArgs {
    /// metadata file to create/modify
    #[argh(positional)]
    file: PathBuf,

    /// image 'variant', eg. 'aws-ecs-1'
    #[argh(option, short = 'l', long = "variant")]
    variant: String,

    /// image version
    #[argh(option, short = 'v', long = "version")]
    image_version: Version,

    /// architecture image is built for
    #[argh(option, short = 'a', long = "arch")]
    arch: String,
}

impl RemoveUpdateArgs {
    fn run(&self) -> Result<()> {
        let mut manifest: Manifest = update_metadata::load_file(&self.file)?;
        // Remove any update that exactly matches the specified update
        manifest.updates.retain(|update| {
            update.arch != self.arch
                || update.variant != self.variant
                || update.version != self.image_version
        });
        // Note: We don't revert the maximum version on removal
        update_metadata::write_file(&self.file, &manifest)?;
        if let Some(current) = manifest.updates.first() {
            info!(
                "Update {}-{}-{} removed. Current maximum version: {}",
                self.arch, self.variant, self.image_version, current.version
            );
        } else {
            info!(
                "Update {}-{}-{} removed. No remaining updates",
                self.arch, self.variant, self.image_version
            );
        }
        Ok(())
    }
}

/// Set waves for an update
#[derive(Debug, FromArgs)]
#[argh(subcommand, name = "set-waves")]
struct WaveArgs {
    /// metadata file to create/modify
    #[argh(positional)]
    file: PathBuf,

    /// image 'variant', eg. 'aws-ecs-1'
    #[argh(option, short = 'l', long = "variant")]
    variant: String,

    /// image version
    #[argh(option, short = 'v', long = "version")]
    image_version: Version,

    /// architecture image is built for
    #[argh(option, short = 'a', long = "arch")]
    arch: String,

    /// file that contains wave structure
    #[argh(option, short = 'w', long = "wave-file")]
    wave_file: Option<PathBuf>,

    // The user can specify the starting point for the the wave offsets, if they don't want them to
    // start when they run this tool.
    //
    // For example, let's say you have a wave with start_after "1 hour" and you run this tool at
    // 2020-02-02 02:00.  If you don't specify --start-at, it will assume "now", and the wave will
    // start 1 hour from then, i.e. 2020-02-02 03:00.
    //
    // If instead you specify --start-at "2020-02-02T10:00:00Z" then the first wave will start 1
    // hour after that, i.e. 2020-02-02 11:00.
    /// wave offsets will be relative to this RFC3339 datetime, instead of right now
    #[argh(option, long = "start-at")]
    start_at: Option<DateTime<Utc>>,
}

impl WaveArgs {
    fn set(self) -> Result<()> {
        let mut manifest: Manifest = update_metadata::load_file(&self.file)?;

        let wave_file = self.wave_file.context(error::WaveFileArgSnafu)?;
        let wave_str =
            fs::read_to_string(&wave_file).context(error::ConfigReadSnafu { path: &wave_file })?;
        let waves: UpdateWaves =
            toml::from_str(&wave_str).context(error::ConfigParseSnafu { path: &wave_file })?;

        let start_at = self.start_at.unwrap_or_else(Utc::now);
        let num_matching = manifest.set_waves(
            self.variant,
            self.arch,
            self.image_version,
            start_at,
            &waves,
        )?;

        if num_matching > 1 {
            warn!("Multiple matching updates for wave - this is weird but not a disaster");
        }
        update_metadata::write_file(&self.file, &manifest)?;
        Ok(())
    }
}

/// Copy the migrations from an input file to an output file
#[derive(Debug, FromArgs)]
#[argh(subcommand, name = "set-migrations")]
struct MigrationArgs {
    /// file to get migrations from (probably Release.toml)
    #[argh(option, short = 'f', long = "from")]
    from: PathBuf,

    /// file to write migrations to (probably manifest.json)
    #[argh(option, short = 't', long = "to")]
    to: PathBuf,
}

impl MigrationArgs {
    fn set(self) -> Result<()> {
        // Load the file we will be writing to
        let mut manifest: Manifest = update_metadata::load_file(&self.to)?;

        // Load the file we will be reading from
        let release_data =
            fs::read_to_string(&self.from).context(error::ConfigReadSnafu { path: &self.from })?;
        let release: Release =
            toml::from_str(&release_data).context(error::ReleaseParseSnafu { path: &self.from })?;

        // Replace the manifest 'migrations' section with the new data
        manifest.migrations = release.migrations;

        update_metadata::write_file(&self.to, &manifest)?;
        Ok(())
    }
}

/// Set the global maximum image version
#[derive(Debug, FromArgs)]
#[argh(subcommand, name = "set-max-version")]
struct MaxVersionArgs {
    /// metadata file to create/modify
    #[argh(positional)]
    file: PathBuf,

    /// maximum valid version
    #[argh(option, short = 'v', long = "max-version")]
    max_version: Version,
}

impl MaxVersionArgs {
    fn run(self) -> Result<()> {
        let mut manifest: Manifest = update_metadata::load_file(&self.file)?;
        manifest.update_max_version(&self.max_version, None, None);
        update_metadata::write_file(&self.file, &manifest)?;
        Ok(())
    }
}

#[derive(FromArgs, Debug)]
#[argh(subcommand)]
enum Command {
    Init(InitArgs),
    AddUpdate(AddUpdateArgs),
    SetWaves(WaveArgs),
    SetMaxVersion(MaxVersionArgs),
    RemoveUpdate(RemoveUpdateArgs),
    SetMigrations(MigrationArgs),
    Validate(ValidateArgs),
}

#[derive(FromArgs, Debug)]
/// Top-level command.
struct TopLevel {
    #[argh(subcommand)]
    cmd: Command,
}

fn main_inner() -> Result<()> {
    // SimpleLogger will send errors to stderr and anything less to stdout.
    SimpleLogger::init(LevelFilter::Info, LogConfig::default()).context(error::LoggerSnafu)?;

    let cmd: TopLevel = argh::from_env();
    match cmd.cmd {
        Command::Init(args) => {
            match update_metadata::write_file(&args.file, &Manifest::default()) {
                Ok(_) => Ok(()),
                Err(e) => Err(error::Error::UpdateMetadata { source: e }),
            }
        }
        Command::AddUpdate(args) => args.run(),
        Command::SetWaves(args) => args.set(),
        Command::SetMaxVersion(args) => args.run(),
        Command::RemoveUpdate(args) => args.run(),
        Command::SetMigrations(args) => args.set(),
        Command::Validate(args) => match update_metadata::load_file(&args.file) {
            Ok(_) => Ok(()),
            Err(e) => Err(error::Error::UpdateMetadata { source: e }),
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
    use std::fs::File;
    use std::path::Path;
    use tempfile::NamedTempFile;

    #[test]
    fn test_set_waves() {
        // A basic manifest with a single update, no migrations, and two
        // image:datastore mappings
        let path = "tests/data/single_wave.json";
        let mut manifest: Manifest = serde_json::from_reader(File::open(path).unwrap()).unwrap();
        let wave_path = "tests/data/default_waves.toml";
        let waves: UpdateWaves = toml::from_str(&fs::read_to_string(wave_path).unwrap()).unwrap();
        let variant = manifest.updates[0].variant.clone();
        let arch = manifest.updates[0].arch.clone();
        let image_version = manifest.updates[0].version.clone();

        assert!(manifest
            .set_waves(variant, arch, image_version, Utc::now(), &waves)
            .is_ok());

        assert!(manifest.updates[0].waves.len() == 4);
    }

    #[test]
    // Ensure that we can update a blank manifest
    fn test_migration_copy() -> Result<()> {
        let release_path = "tests/data/release.toml";
        let temp_manifest = NamedTempFile::new().context(error::TmpFileCreateSnafu)?;

        // Create a new blank manifest
        update_metadata::write_file(temp_manifest.path(), &Manifest::default()).unwrap();

        // Copy the migration data to the new manifest
        MigrationArgs {
            from: PathBuf::from(&release_path),
            to: PathBuf::from(temp_manifest.path()),
        }
        .set()
        .unwrap();

        // Make sure the manifest has the correct releases
        let manifest: Manifest = update_metadata::load_file(temp_manifest.path()).unwrap();
        let release_data = fs::read_to_string(release_path).unwrap();
        let release: Release = toml::from_str(&release_data).unwrap();
        assert_eq!(manifest.migrations, release.migrations);
        Ok(())
    }

    #[test]
    // Ensure that we can update an existing manifest
    fn test_migration_update() -> Result<()> {
        let release_path = "tests/data/release.toml";
        let example_manifest = "tests/data/example.json";

        // Write example data to temp manifest so we don't overwrite the file
        // when we call MigrationsArgs.set() below
        let temp_manifest = NamedTempFile::new().context(error::TmpFileCreateSnafu)?;
        let example_data = fs::read_to_string(example_manifest).unwrap();
        fs::write(&temp_manifest, example_data).unwrap();

        // Copy the migration data to the existing manifest
        MigrationArgs {
            from: PathBuf::from(&release_path),
            to: PathBuf::from(&temp_manifest.path()),
        }
        .set()
        .unwrap();

        // Make sure the manifest has the correct releases
        let manifest: Manifest =
            update_metadata::load_file(Path::new(&temp_manifest.path())).unwrap();
        let release_data = fs::read_to_string(release_path).unwrap();
        let release: Release = toml::from_str(&release_data).unwrap();
        assert_eq!(manifest.migrations, release.migrations);
        Ok(())
    }

    #[test]
    fn max_versions() -> Result<()> {
        let tmpfd = NamedTempFile::new().context(error::TmpFileCreateSnafu)?;
        update_metadata::write_file(tmpfd.path(), &Manifest::default()).unwrap();
        AddUpdateArgs {
            file: PathBuf::from(tmpfd.path()),
            variant: String::from("yum"),
            arch: String::from("x86_64"),
            image_version: Version::parse("1.2.3").unwrap(),
            max_version: Some(Version::parse("1.2.3").unwrap()),
            boot: String::from("boot"),
            root: String::from("root"),
            hash: String::from("hash"),
        }
        .run()
        .unwrap();
        AddUpdateArgs {
            file: PathBuf::from(tmpfd.path()),
            variant: String::from("yum"),
            arch: String::from("x86_64"),
            image_version: Version::parse("1.2.5").unwrap(),
            max_version: Some(Version::parse("1.2.3").unwrap()),
            boot: String::from("boot"),
            root: String::from("root"),
            hash: String::from("hash"),
        }
        .run()
        .unwrap();
        AddUpdateArgs {
            file: PathBuf::from(tmpfd.path()),
            variant: String::from("yum"),
            arch: String::from("x86_64"),
            image_version: Version::parse("1.2.4").unwrap(),
            max_version: Some(Version::parse("1.2.4").unwrap()),
            boot: String::from("boot"),
            root: String::from("root"),
            hash: String::from("hash"),
        }
        .run()
        .unwrap();

        let m: Manifest = update_metadata::load_file(tmpfd.path())?;
        for u in m.updates {
            assert!(u.max_version == Version::parse("1.2.4").unwrap());
        }
        Ok(())
    }
}
