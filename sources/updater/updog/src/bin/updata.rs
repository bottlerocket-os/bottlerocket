#![deny(rust_2018_idioms)]
#![warn(clippy::pedantic)]

#[path = "../error.rs"]
mod error;

#[macro_use]
extern crate log;

use crate::error::Result;
use chrono::{DateTime, Utc};
use semver::Version;
use simplelog::{Config as LogConfig, LevelFilter, TermLogger, TerminalMode};
use snafu::{ErrorCompat, OptionExt, ResultExt};
use std::fs;
use std::path::PathBuf;
use structopt::StructOpt;
use update_metadata::{Images, Manifest, Release, UpdateWaves};

#[derive(Debug, StructOpt)]
struct GeneralArgs {
    // metadata file to create/modify
    file: PathBuf,
}

#[derive(Debug, StructOpt)]
struct AddUpdateArgs {
    // metadata file to create/modify
    file: PathBuf,

    // image 'variant', eg. 'aws-k8s-1.15'
    #[structopt(short = "f", long = "variant")]
    variant: String,

    // image version
    #[structopt(short = "v", long = "version")]
    image_version: Version,

    // architecture image is built for
    #[structopt(short = "a", long = "arch")]
    arch: String,

    // maximum valid version
    #[structopt(short = "m", long = "max-version")]
    max_version: Option<Version>,

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
        let mut manifest: Manifest = match update_metadata::load_file(&self.file) {
            Ok(m) => m,
            _ => Manifest::default(), // TODO only if EEXIST
        };

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

#[derive(Debug, StructOpt)]
struct RemoveUpdateArgs {
    // metadata file to create/modify
    file: PathBuf,

    // image 'variant', eg. 'aws-k8s-1.15'
    #[structopt(short = "l", long = "variant")]
    variant: String,

    // image version
    #[structopt(short = "v", long = "version")]
    image_version: Version,

    // architecture image is built for
    #[structopt(short = "a", long = "arch")]
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

#[derive(Debug, StructOpt)]
struct WaveArgs {
    // metadata file to create/modify
    file: PathBuf,

    // image 'variant', eg. 'aws-k8s-1.15'
    #[structopt(short = "l", long = "variant")]
    variant: String,

    // image version
    #[structopt(short = "v", long = "version")]
    image_version: Version,

    // architecture image is built for
    #[structopt(short = "a", long = "arch")]
    arch: String,

    // file that contains wave structure
    #[structopt(short = "w", long = "wave-file", conflicts_with_all = &["bound", "start"])]
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
    /// Wave offsets will be relative to this RFC3339 datetime, instead of right now
    #[structopt(long = "start-at")]
    start_at: Option<DateTime<Utc>>,
}

impl WaveArgs {
    fn set(self) -> Result<()> {
        let mut manifest: Manifest = update_metadata::load_file(&self.file)?;

        let wave_file = self.wave_file.context(error::WaveFileArg)?;
        let wave_str =
            fs::read_to_string(&wave_file).context(error::ConfigRead { path: &wave_file })?;
        let waves: UpdateWaves =
            toml::from_str(&wave_str).context(error::ConfigParse { path: &wave_file })?;

        let start_at = self.start_at.unwrap_or(Utc::now());
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

#[derive(Debug, StructOpt)]
struct MigrationArgs {
    // file to get migrations from (probably Release.toml)
    #[structopt(short = "f", long = "from")]
    from: PathBuf,

    // file to write migrations to (probably manifest.json)
    #[structopt(short = "t", long = "to")]
    to: PathBuf,
}

impl MigrationArgs {
    fn set(self) -> Result<()> {
        // Load the file we will be writing to
        let mut manifest: Manifest = update_metadata::load_file(&self.to)?;

        // Load the file we will be reading from
        let release_data =
            fs::read_to_string(&self.from).context(error::ConfigRead { path: &self.from })?;
        let release: Release =
            toml::from_str(&release_data).context(error::ReleaseParse { path: &self.from })?;

        // Replace the manifest 'migrations' section with the new data
        manifest.migrations = release.migrations;

        update_metadata::write_file(&self.to, &manifest)?;
        Ok(())
    }
}

#[derive(Debug, StructOpt)]
struct MaxVersionArgs {
    // metadata file to create/modify
    file: PathBuf,

    // maximum valid version
    #[structopt(short, long)]
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

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
enum Command {
    /// Create an empty manifest
    Init(GeneralArgs),
    /// Add a new update to the manifest, not including wave information
    AddUpdate(AddUpdateArgs),
    /// Set waves for an update
    SetWaves(WaveArgs),
    /// Set the global maximum image version
    SetMaxVersion(MaxVersionArgs),
    /// Remove an update from the manifest, including wave information
    RemoveUpdate(RemoveUpdateArgs),
    /// Copy the migrations from an input file to an output file
    SetMigrations(MigrationArgs),
    /// Validate a manifest file, but make no changes
    Validate(GeneralArgs),
}

fn main_inner() -> Result<()> {
    // TerminalMode::Mixed will send errors to stderr and anything less to stdout.
    TermLogger::init(LevelFilter::Info, LogConfig::default(), TerminalMode::Mixed)
        .context(error::Logger)?;

    match Command::from_args() {
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
        let waves: UpdateWaves = toml::from_str(&fs::read_to_string(&wave_path).unwrap()).unwrap();
        let variant = manifest.updates[0].variant.clone();
        let arch = manifest.updates[0].arch.clone();
        let image_version = manifest.updates[0].version.clone();

        assert!(manifest
            .set_waves(variant, arch, image_version, Utc::now(), &waves)
            .is_ok());

        assert!(manifest.updates[0].waves.len() == 4)
    }

    #[test]
    // Ensure that we can update a blank manifest
    fn test_migration_copy() -> Result<()> {
        let release_path = "tests/data/release.toml";
        let temp_manifest = NamedTempFile::new().context(error::TmpFileCreate)?;

        // Create a new blank manifest
        update_metadata::write_file(&temp_manifest.path(), &Manifest::default()).unwrap();

        // Copy the migration data to the new manifest
        MigrationArgs {
            from: PathBuf::from(&release_path),
            to: PathBuf::from(temp_manifest.path()),
        }
        .set()
        .unwrap();

        // Make sure the manifest has the correct releases
        let manifest: Manifest = update_metadata::load_file(&temp_manifest.path()).unwrap();
        let release_data = fs::read_to_string(&release_path).unwrap();
        let release: Release = toml::from_str(&release_data).unwrap();
        assert_eq!(manifest.migrations, release.migrations);
        Ok(())
    }

    #[test]
    // Ensure that we can update an existing manifest
    fn test_migration_update() -> Result<()> {
        let release_path = "tests/data/release.toml";
        let example_manifest = "tests/data/example.json";

        // Write example data to temp manifest so we dont' overwrite the file
        // when we call MigrationsArgs.set() below
        let temp_manifest = NamedTempFile::new().context(error::TmpFileCreate)?;
        let example_data = fs::read_to_string(&example_manifest).unwrap();
        fs::write(&temp_manifest, &example_data).unwrap();

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
        let release_data = fs::read_to_string(&release_path).unwrap();
        let release: Release = toml::from_str(&release_data).unwrap();
        assert_eq!(manifest.migrations, release.migrations);
        Ok(())
    }

    #[test]
    fn max_versions() -> Result<()> {
        let tmpfd = NamedTempFile::new().context(error::TmpFileCreate)?;
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
