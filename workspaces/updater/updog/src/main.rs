#![warn(clippy::pedantic)]

mod error;
mod de;

use crate::error::Result;
use chrono::{DateTime, Utc};
use std::collections::{BTreeMap};
use platforms::{TARGET_ARCH};
use rand::{thread_rng, Rng};
use semver::Version;
use serde::{Serialize, Deserialize};
use signpost::State;
use snafu::{OptionExt, ResultExt, ErrorCompat};
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::ops::Bound::{Included, Excluded};
use std::{thread};
use std::time::Duration;
use tough::Repository;

const TRUSTED_ROOT_PATH: &str = "/usr/share/updog/root.json";
const MAX_SEED: u64 = 2048;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum Command {
    CheckUpdate,
    Update,
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    metadata_base_url: String,
    target_base_url: String,
    seed: Option<u64>,
    // TODO API sourced configuration, eg.
    // blacklist: Option<Vec<Version>>,
    // mode: Option<{Automatic, Managed, Disabled}>

}

#[derive(Debug, Deserialize)]
struct Images {
    boot: String,
    root: String,
    hash: String,
}

#[derive(Debug, Deserialize)]
struct Update {
    flavor: String,
    arch: String,
    version: Version,
    max_version: Version,
    #[serde(deserialize_with = "de::deserialize_keys")]
    waves: BTreeMap<u64, DateTime<Utc>>,
    images: Images,
}

impl Update {
    fn update_ready(&self, config: &Config) -> Result<bool> {

        if let Some(seed) = config.seed {
            // Has this client's wave started
            if let Some((_, wave)) = self.waves.range((Included(0), Included(seed)))
                                               .last() {
                return Ok(*wave <= Utc::now());
            }

            // Alternately have all waves passed
            if let Some((_, wave)) = self.waves.iter().last() {
                return Ok(*wave <= Utc::now());
            }

            return error::NoWave.fail();
        }
        error::MissingSeed.fail()
    }

    fn jitter(&self, config: &Config) -> Option<u64> {

        if let Some(seed) = config.seed {
            let prev = self.waves.range((Included(0), Included(seed)))
                                 .last();
            let next = self.waves.range((Excluded(seed), Excluded(MAX_SEED)))
                                 .next();
            match (prev, next) {
                (Some((_, start)), Some((_, end))) => {
                    if Utc::now() < *end {
                        return Some((end.timestamp() - start.timestamp()) as u64);
                    }
                },
                _ => (),
            }

        }
        None
    }
}

#[derive(Debug, Deserialize)]
struct Manifest {
    updates: Vec<Update>,
}

fn usage() -> ! {
    #[rustfmt::skip]
    eprintln!("\
USAGE:
    updog <SUBCOMMAND> <OPTIONS>

SUBCOMMANDS:
    check-update            Show if an update is available
    update                  Perform an update if available
OPTIONS:
    [ --verbose --verbose ... ]   Increase log verbosity");
    std::process::exit(1)
}

fn load_config() -> Result<Config> {
    let path = "/etc/updog.toml";
    let s = fs::read_to_string(path).context(error::ConfigRead { path })?;
    let mut config: Config = toml::from_str(&s).context(error::ConfigParse { path })?;
    if config.seed.is_none() {
        let mut rng = thread_rng();
        config.seed = Some(rng.gen_range(0, MAX_SEED));
        println!("new seed {:?}, storing to {}", config.seed, &path);
        let s = toml::to_string(&config).context(error::ConfigSerialize { path })?;
        fs::write(&path, &s).context(error::ConfigWrite { path })?;
    }
    Ok(config)
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

fn running_version() -> Result<(Version, String)> {
    let mut version: Option<Version> = None;
    let mut flavor: Option<String> = None;

    let reader = BufReader::new(File::open("/etc/os-release").context(error::VersionIdRead)?);
    for line in reader.lines() {
        let line = line.context(error::VersionIdRead)?;
        let line = line.trim();
        if version.is_none() {
            let key = "VERSION_ID=";
            if line.starts_with(key) {
                version = Some(Version::parse(&line[key.len()..]).context(error::VersionIdParse)?);
            }
        } else if flavor.is_none() {
            let key = "VARIANT_ID=";
            if line.starts_with(key) {
                flavor = Some(String::from(&line[key.len()..]));
            }
        } else {
            break;
        }
    }

    match (version, flavor) {
        (Some(v), Some(f))    => Ok((v, f)),
        _                     => error::VersionIdNotFound.fail(),
    }
}

// TODO use config if there is api-sourced configuration that could affect this
// TODO updog.toml may include settings that cause us to ignore/delay
// certain/any updates;
//  Ignore Specific Target Version
//  Ingore Any Target
//  ...
fn update_required<'a>(_config: &Config,
    manifest: &'a Manifest,
    version: &Version,
    flavor: &String)
    -> Option<&'a Update> {

    let mut updates: Vec<&Update> = manifest.updates
        .iter()
        .filter(|u|
            u.flavor == *flavor &&
            u.arch == TARGET_ARCH.as_str() &&
            u.version <= u.max_version)
        .collect();

    // sort descending
    updates.sort_unstable_by(|a, b| b.version.cmp(&a.version));
    for update in updates {
        // If the current running version is greater than the max version ever published,
        // or moves us to a valid version <= the maximum version, update.
        if *version < update.version || *version > update.max_version {
            return Some(update);
        }
    }
    None
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

fn update_image(update: &Update, repository: &Repository, jitter: Option<u64>) -> Result<()> {

    // Jitter the exact update time
    // Now: lazy spin
    // If range > calling_interval we could just exit and wait until updog
    // is called again.
    // Alternately if Updog is going to be driven by some orchestrator
    // then the jitter could be reduced or left to the caller.
    if let Some(jitter) = jitter {
        let mut rng = thread_rng();
        let jitter = Duration::new(rng.gen_range(1, jitter), 0);
        println!("Waiting {:?} till update", jitter);
        thread::sleep(jitter);
    }

    let mut gpt_state = State::load().context(error::PartitionTableRead)?;
    gpt_state.clear_inactive();
    // Write out the clearing of the inactive partition immediately, because we're about to
    // overwrite the partition set with update data and don't want it to be used until we
    // know we're done with all components.
    gpt_state.write().context(error::PartitionTableWrite)?;

    let inactive = gpt_state.inactive_set();

    // TODO Do we want to recover the inactive side on an error?
    write_target_to_disk(repository, &update.images.boot, &inactive.boot)?;
    write_target_to_disk(repository, &update.images.root, &inactive.root)?;
    write_target_to_disk(repository, &update.images.hash, &inactive.hash)?;

    gpt_state.upgrade_to_inactive();
    gpt_state.write().context(error::PartitionTableWrite)?;
    Ok(())
}

/// Struct to hold the specified command line argument values
struct Arguments {
    subcommand: String,
    verbosity: usize,
}

/// Parse the command line arguments to get the user-specified values
fn parse_args(args: std::env::Args) -> Arguments {
    let mut subcommand = None;
    let mut verbosity: usize = 3; // Default log level to 3 (Info)

    for arg in args.skip(1) {
        match arg.as_ref() {
            "-v" | "--verbose" => {
                verbosity += 1;
            }
            // Assume any arguments not prefixed with '-' is a subcommand
            s if !s.starts_with('-') => {
                if subcommand.is_some() {
                    usage();
                }
                subcommand = Some(s.to_string());
            }
            _ => usage(),
        }
    }

    Arguments {
        subcommand: subcommand.unwrap_or_else(|| usage()),
        verbosity,
    }
}

fn main_inner() -> Result<()> {
    // Parse and store the arguments passed to the program
    let arguments = parse_args(std::env::args());

    // TODO Fix this later when we decide our logging story
    // TODO Will this also cover telemetry or via another mechanism?
    // Start the logger
    stderrlog::new()
        .timestamp(stderrlog::Timestamp::Millisecond)
        .verbosity(arguments.verbosity)
        .color(stderrlog::ColorChoice::Never)
        .init()
        .unwrap();

    let command =
        serde_plain::from_str::<Command>(&arguments.subcommand).unwrap_or_else(|_| usage());

    let config = load_config()?;
    let repository = load_repository(&config)?;
    let manifest = load_manifest(&repository)?;
    let (current_version, flavor) = running_version().unwrap();

    match command {
        Command::CheckUpdate => {
            match update_required(&config, &manifest, &current_version, &flavor) {
                Some(u) => println!("{}-{}", u.flavor, u.version),
                _       => return error::NoUpdate.fail(),
            }
        }
        Command::Update => {
            if let Some(u) = update_required(&config, &manifest, &current_version, &flavor) {
                if u.update_ready(&config)? {
                    update_image(u, &repository, u.jitter(&config))?;
                    println!("Update applied: {}-{}", u.flavor, u.version);
                } else {
                    eprintln!("Update available in later wave");
                }
            } else {
                eprintln!("No update required");
            }
        }
    }

    Ok(())
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
