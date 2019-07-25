#![warn(clippy::pedantic)]

mod error;

use crate::error::Result;
use serde::Deserialize;
use signpost::State;
use snafu::{OptionExt, ResultExt};
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use tough::Repository;

static TRUSTED_ROOT_PATH: &str = "/usr/share/updog/root.json";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum Command {
    CheckUpdate,
    Update,
}

#[derive(Debug, Deserialize)]
struct Config {
    metadata_base_url: String,
    target_base_url: String,
}

#[derive(Debug, Deserialize)]
struct Manifest {
    version: u64,
    max_version: u64,
}

fn usage() -> ! {
    #[rustfmt::skip]
    eprintln!("\
USAGE:
    updog <SUBCOMMAND>

SUBCOMMANDS:
    check-update            Show if an update is available
    update                  Perform an update if available");
    std::process::exit(1)
}

fn load_config() -> Result<Config> {
    let path = "/etc/updog.toml";
    let s = fs::read_to_string(path).context(error::ConfigRead { path })?;
    toml::from_str(&s).context(error::ConfigParse { path })
}

fn load_repository(config: &Config) -> Result<Repository> {
    fs::create_dir_all("/var/lib/thar/updog").context(error::CreateMetadataCache)?;
    Repository::load(
        File::open(TRUSTED_ROOT_PATH).context(error::OpenRoot {
            path: TRUSTED_ROOT_PATH,
        })?,
        "/var/lib/thar/updog",
        1024 * 1024, // max allowed root.json size, 1 MiB
        1024 * 1024, // max allowed timestamp.json size, 1 MiB
        &config.metadata_base_url,
        &config.target_base_url,
    )
    .context(error::Metadata)
}

fn load_manifest(repository: &Repository) -> Result<Manifest> {
    let target = "manifest.json";
    serde_json::from_reader(
        repository
            .read_target(target)
            .context(error::Metadata)?
            .context(error::TargetNotFound { target })?,
    )
    .context(error::ManifestParse)
}

fn running_version() -> Result<u64> {
    let reader = BufReader::new(File::open("/etc/os-release").context(error::VersionIdRead)?);
    for line in reader.lines() {
        let line = line.context(error::VersionIdRead)?;
        let line = line.trim();
        let key = "VERSION_ID=";
        if line.starts_with(key) {
            return Ok(line[key.len()..].parse().context(error::VersionIdParse)?);
        }
    }
    error::VersionIdNotFound.fail()
}

fn update_required(manifest: &Manifest) -> Result<bool> {
    let version = running_version()?;
    // TODO waves and whatnot
    Ok(
        // If the current running version is less than the current published version, update.
        version < manifest.version ||
        // If the current running version is greater than the max version ever published, update.
        version > manifest.max_version,
    )
}

fn write_target_to_disk<P: AsRef<Path>>(
    repository: &Repository,
    target: &str,
    disk_path: P,
) -> Result<()> {
    let reader = repository
        .read_target(target)
        .context(error::Metadata)?
        .context(error::TargetNotFound { target })?;
    let mut reader = lz4::Decoder::new(reader).context(error::Lz4Decode { target })?;
    let mut f = OpenOptions::new()
        .write(true)
        .open(disk_path.as_ref())
        .context(error::OpenPartition {
            path: disk_path.as_ref(),
        })?;
    io::copy(&mut reader, &mut f).context(error::WriteUpdate)?;
    Ok(())
}

fn main_inner() -> Result<()> {
    // TODO Fix this later when we decide our logging story
    // Start the logger
    stderrlog::new()
        .timestamp(stderrlog::Timestamp::Millisecond)
        .verbosity(3)
        .color(stderrlog::ColorChoice::Never)
        .init()
        .unwrap();

    let command_str = std::env::args().nth(1).unwrap_or_else(|| usage());
    let command = serde_plain::from_str::<Command>(&command_str).unwrap_or_else(|_| usage());

    match command {
        Command::CheckUpdate => {
            println!(
                "{:?}",
                update_required(&load_manifest(&load_repository(&load_config()?)?)?)?
            );
        }
        Command::Update => {
            let repository = load_repository(&load_config()?)?;
            let manifest = load_manifest(&repository)?;
            if !update_required(&manifest)? {
                eprintln!("No update required");
                return Ok(());
            }

            // TODO figure out the correct flavor + arch
            let mut gpt_state = State::load().context(error::PartitionTableRead)?;
            gpt_state.clear_inactive();
            // Write out the clearing of the inactive partition immediately, because we're about to
            // overwrite the partition set with update data and don't want it to be used until we
            // know we're done with all components.
            gpt_state.write().context(error::PartitionTableWrite)?;

            let inactive = gpt_state.inactive_set();
            write_target_to_disk(&repository, "thar-x86_64-boot.ext4.lz4", &inactive.boot)?;
            write_target_to_disk(&repository, "thar-x86_64-root.ext4.lz4", &inactive.root)?;
            write_target_to_disk(&repository, "thar-x86_64-root.verity.lz4", &inactive.hash)?;

            gpt_state.upgrade_to_inactive();
            gpt_state.write().context(error::PartitionTableWrite)?;
        }
    }

    Ok(())
}

fn main() -> ! {
    std::process::exit(match main_inner() {
        Ok(()) => 0,
        Err(err) => {
            eprintln!("{}", err);
            // TODO print backtraces when RUST_BACKTRACE is set
            1
        }
    })
}
