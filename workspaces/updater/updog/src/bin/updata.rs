#![warn(clippy::pedantic)]

#[path = "../error.rs"]
mod error;

use crate::error::Result;
use chrono::{DateTime, Utc};
use data_store_version::Version as DVersion;
use semver::Version;
use snafu::{ErrorCompat, ResultExt};
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
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
    version: Version,

    // architecture image is built for
    #[structopt(short = "a", long = "arch")]
    arch: String,

    // corresponding datastore version for this image
    #[structopt(short = "d", long = "data-version")]
    datastore: DVersion,

    // maximum valid version
    #[structopt(short = "m", long = "max-version")]
    max_version: Version,

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
        let mut m: Manifest = match load_file(&self.file) {
            Ok(m) => m,
            _ => Manifest::default(), // TODO only if EEXIST
        };
        m.datastore_versions
            .insert(self.version.clone(), self.datastore);
        let u = Update {
            flavor: self.flavor,
            arch: self.arch,
            version: self.version,
            max_version: self.max_version,
            images: Images {
                root: self.root,
                boot: self.boot,
                hash: self.hash,
            },
            waves: BTreeMap::new(),
        };
        update_max_version(&mut m, &u.max_version, Some(&u.arch), Some(&u.flavor));
        m.updates.push(u);
        write_file(&self.file, &m)
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
    version: Version,

    // architecture image is built for
    #[structopt(short = "a", long = "arch")]
    arch: String,
}

impl RemoveUpdateArgs {
    fn run(&self) -> Result<()> {
        let mut m: Manifest = load_file(&self.file)?;
        m.updates.retain(|u| {
            !(u.arch == self.arch && u.flavor == self.flavor && u.version == self.version)
        });
        // Note: We don't revert the maximum version on removal
        write_file(&self.file, &m)
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
    version: Version,

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
    fn add(self) -> Result<()> {
        let mut m: Manifest = load_file(&self.file)?;
        let matching: Vec<&mut Update> = m
            .updates
            .iter_mut()
            .filter(|u| u.arch == self.arch && u.flavor == self.flavor && u.version == self.version)
            .collect();
        if matching.len() > 1 {
            println!("Multiple matching updates for wave - this is weird but not a disaster");
        }
        if let Some(start) = self.start {
            for u in matching {
                u.waves.insert(self.bound, start);
            }
            write_file(&self.file, &m)
        } else {
            error::WaveStartArg.fail()
        }
    }

    fn remove(self) -> Result<()> {
        let mut m: Manifest = load_file(&self.file)?;
        let matching: Vec<&mut Update> = m
            .updates
            .iter_mut()
            .filter(|u| u.arch == self.arch && u.flavor == self.flavor && u.version == self.version)
            .collect();
        for u in matching {
            u.waves.remove(&self.bound);
        }
        write_file(&self.file, &m)
    }
}

#[derive(Debug, StructOpt)]
struct MigrationArgs {
    // metadata file to create/modify
    file: PathBuf,

    // starting datastore version
    #[structopt(short = "f", long = "from")]
    from: DVersion,

    // target datastore version
    #[structopt(short = "t", long = "to")]
    to: DVersion,

    // whether to append to or replace any existing migration list
    #[structopt(short, long)]
    append: bool,

    // migration names
    migrations: Vec<String>,
}

impl MigrationArgs {
    fn add(self) -> Result<()> {
        let mut m: Manifest = load_file(&self.file)?;
        let mut migrations = self.migrations;
        if self.append {
            if let Some(e) = m.migrations.remove(&(self.from, self.to)) {
                migrations.extend_from_slice(&e);
            }
        }
        m.migrations.insert((self.from, self.to), migrations);
        write_file(&self.file, &m)
    }

    fn remove(self) -> Result<()> {
        let mut m: Manifest = load_file(&self.file)?;
        m.migrations.remove(&(self.from, self.to));
        write_file(&self.file, &m)
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
        let mut m: Manifest = load_file(&self.file)?;
        update_max_version(&mut m, &self.max_version, None, None);
        write_file(&self.file, &m)
    }
}

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
enum Command {
    Init(GeneralArgs),
    AddUpdate(AddUpdateArgs),
    AddWave(WaveArgs),
    AddMigration(MigrationArgs),
    SetMaxVersion(MaxVersionArgs),
    RemoveUpdate(RemoveUpdateArgs),
    RemoveMigration(MigrationArgs),
    RemoveWave(WaveArgs),
    Validate(GeneralArgs),
}

fn load_file(path: &Path) -> Result<Manifest> {
    serde_json::from_reader(File::open(path).context(error::ManifestRead { path })?)
        .context(error::ManifestParse)
}

fn write_file(path: &Path, manifest: &Manifest) -> Result<()> {
    let manifest = serde_json::to_string_pretty(&manifest).context(error::UpdateSerialize)?;
    fs::write(path, &manifest).context(error::ConfigWrite { path })?;
    Ok(())
}

/// Update the maximum version for all updates that optionally match the
/// architecture and flavor of some new update.
fn update_max_version(
    m: &mut Manifest,
    version: &Version,
    arch: Option<&str>,
    flavor: Option<&str>,
) {
    let matching: Vec<&mut Update> = m
        .updates
        .iter_mut()
        .filter(|u| match (arch, flavor) {
            (Some(arch), Some(flavor)) => u.arch == arch && u.flavor == flavor,
            (Some(arch), None) => u.arch == arch,
            (None, Some(flavor)) => u.flavor == flavor,
            _ => true,
        })
        .collect();
    for u in matching {
        u.max_version = version.clone();
    }
}

fn main_inner() -> Result<()> {
    match Command::from_args() {
        Command::Init(args) => write_file(&args.file, &Manifest::default()),
        Command::AddUpdate(args) => args.run(),
        Command::AddWave(args) => args.add(),
        Command::AddMigration(args) => args.add(),
        Command::SetMaxVersion(args) => args.run(),
        Command::RemoveUpdate(args) => args.run(),
        Command::RemoveWave(args) => args.remove(),
        Command::RemoveMigration(args) => args.remove(),
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
            eprintln!("{}", err);
            if let Some(var) = std::env::var_os("RUST_BACKTRACE") {
                if var != "0" {
                    if let Some(backtrace) = err.backtrace() {
                        eprintln!("\n{:?}", backtrace);
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
    use std::str::FromStr;
    use tempfile::NamedTempFile;

    #[test]
    fn max_versions() -> Result<()> {
        let tmpfd = NamedTempFile::new().context(error::TmpFileCreate)?;
        AddUpdateArgs {
            file: PathBuf::from(tmpfd.path()),
            flavor: String::from("yum"),
            arch: String::from("x86_64"),
            version: Version::parse("1.2.3").unwrap(),
            max_version: Version::parse("1.2.3").unwrap(),
            datastore: DVersion::from_str("1.0").unwrap(),
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
            version: Version::parse("1.2.5").unwrap(),
            max_version: Version::parse("1.2.3").unwrap(),
            datastore: DVersion::from_str("1.0").unwrap(),
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
            version: Version::parse("1.2.4").unwrap(),
            max_version: Version::parse("1.2.4").unwrap(),
            datastore: DVersion::from_str("1.0").unwrap(),
            boot: String::from("boot"),
            root: String::from("root"),
            hash: String::from("hash"),
        }
        .run()
        .unwrap();

        let m: Manifest = load_file(tmpfd.path())?;
        for u in m.updates {
            assert!(u.max_version == Version::parse("1.2.4").unwrap());
        }
        Ok(())
    }
}
