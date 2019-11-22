#![deny(rust_2018_idioms)]
#![warn(clippy::pedantic)]

mod error;

use crate::error::Result;
use chrono::{DateTime, Utc};
use data_store_version::Version as DataVersion;
use semver::Version as SemVer;
use serde::{Deserialize, Serialize};
use signpost::State;
use simplelog::{Config as LogConfig, LevelFilter, TermLogger, TerminalMode};
use snafu::{ErrorCompat, OptionExt, ResultExt};
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::process;
use std::str::FromStr;
use tough::{HttpTransport, Limits, Repository, Settings};
use update_metadata::{Manifest, Update};

type HttpRepo<'a> = Repository<'a, HttpTransport>;

#[cfg(target_arch = "x86_64")]
const TARGET_ARCH: &str = "x86_64";
#[cfg(target_arch = "aarch64")]
const TARGET_ARCH: &str = "aarch64";

const TRUSTED_ROOT_PATH: &str = "/usr/share/updog/root.json";
const MIGRATION_PATH: &str = "/var/lib/thar/datastore/migrations";

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
enum Command {
    CheckUpdate,
    Whats,
    Prepare,
    Update,
    UpdateImage,
    UpdateApply,
}

#[derive(Debug, Deserialize)]
struct Config {
    metadata_base_url: String,
    target_base_url: String,
    seed: u32,
    // TODO API sourced configuration, eg.
    // blacklist: Option<Vec<SemVer>>,
    // mode: Option<{Automatic, Managed, Disabled}>
}

/// Prints a more specific message before exiting through usage().
fn usage_msg<S: AsRef<str>>(msg: S) -> ! {
    eprintln!("{}\n", msg.as_ref());
    usage();
}

fn usage() -> ! {
    #[rustfmt::skip]
    eprintln!("\
USAGE:
    updog <SUBCOMMAND> <OPTIONS>

SUBCOMMANDS:
    check-update            Show if an update is available
        [ -a | --all ]                Output all applicable updates

    prepare                 Download update files and migration targets

    update                  Perform an update if available
        [ -i | --image version ]      Update to a specfic image version
        [ -n | --now ]                Update immediately, ignoring wave limits
        [ -r | --reboot ]             Reboot into new update on success
        [ -t | --timestamp time ]     The timestamp from which to execute an update

    update-image            Download & write an update but do not update flags
        [ -i | --image version ]      Update to a specfic image version
        [ -n | --now ]                Update immediately, ignoring wave limits
        [ -t | --timestamp time ]     The timestamp to execute an update from

    update-apply            Update boot flags (after having called update-image)
        [ -r | --reboot ]             Reboot after updating boot flags

GLOBAL OPTIONS:
    [ -j | --json ]               JSON-formatted output
    [ --log-level trace|debug|info|warn|error ]  Set logging verbosity");
    std::process::exit(1)
}

fn load_config() -> Result<Config> {
    let path = "/etc/updog.toml";
    let s = fs::read_to_string(path).context(error::ConfigRead { path })?;
    let config: Config = toml::from_str(&s).context(error::ConfigParse { path })?;
    Ok(config)
}

fn load_repository<'a>(transport: &'a HttpTransport, config: &'a Config) -> Result<HttpRepo<'a>> {
    fs::create_dir_all("/var/lib/thar/updog").context(error::CreateMetadataCache)?;
    Repository::load(
        transport,
        Settings {
            root: File::open(TRUSTED_ROOT_PATH).context(error::OpenRoot {
                path: TRUSTED_ROOT_PATH,
            })?,
            datastore: Path::new("/var/lib/thar/updog"),
            metadata_base_url: &config.metadata_base_url,
            target_base_url: &config.target_base_url,
            limits: Limits {
                max_root_size: 1024 * 1024,         // 1 MiB
                max_targets_size: 1024 * 1024 * 10, // 10 MiB
                max_timestamp_size: 1024 * 1024,    // 1 MiB
                max_root_updates: 1024,
            },
        },
    )
    .context(error::Metadata)
}

fn load_manifest(repository: &HttpRepo<'_>) -> Result<Manifest> {
    let target = "manifest.json";
    serde_json::from_reader(
        repository
            .read_target(target)
            .context(error::Metadata)?
            .context(error::TargetNotFound { target })?,
    )
    .context(error::ManifestParse)
}

fn running_version() -> Result<(SemVer, String)> {
    let mut version: Option<SemVer> = None;
    let mut flavor: Option<String> = None;

    let reader = BufReader::new(File::open("/etc/os-release").context(error::VersionIdRead)?);
    for line in reader.lines() {
        let line = line.context(error::VersionIdRead)?;
        let line = line.trim();
        if version.is_none() {
            let key = "VERSION_ID=";
            if line.starts_with(key) {
                version = Some(
                    SemVer::parse(&line[key.len()..]).context(error::VersionIdParse { line })?,
                );
                continue;
            }
        }
        if flavor.is_none() {
            let key = "VARIANT_ID=";
            if line.starts_with(key) {
                flavor = Some(String::from(&line[key.len()..]));
                continue;
            }
        }
        if version.is_some() && flavor.is_some() {
            break;
        }
    }

    match (version, flavor) {
        (Some(v), Some(f)) => Ok((v, f)),
        _ => error::VersionIdNotFound.fail(),
    }
}

fn applicable_updates<'a>(manifest: &'a Manifest, flavor: &str) -> Vec<&'a Update> {
    let mut updates: Vec<&Update> = manifest
        .updates
        .iter()
        .filter(|u| u.flavor == *flavor && u.arch == TARGET_ARCH && u.version <= u.max_version)
        .collect();
    // sort descending
    updates.sort_unstable_by(|a, b| b.version.cmp(&a.version));
    updates
}

// TODO use config if there is api-sourced configuration that could affect this
// TODO updog.toml may include settings that cause us to ignore/delay
// certain/any updates;
//  Ignore Specific Target Version
//  Ingore Any Target
//  ...
fn update_required<'a>(
    _config: &Config,
    manifest: &'a Manifest,
    version: &SemVer,
    flavor: &str,
    force_version: Option<SemVer>,
) -> Option<&'a Update> {
    let updates = applicable_updates(manifest, flavor);

    if let Some(forced_version) = force_version {
        return updates.into_iter().find(|u| u.version == forced_version);
    }

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
    repository: &HttpRepo<'_>,
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
        .create(true)
        .open(disk_path.as_ref())
        .context(error::OpenPartition {
            path: disk_path.as_ref(),
        })?;
    io::copy(&mut reader, &mut f).context(error::WriteUpdate)?;
    Ok(())
}

fn migration_targets(
    from: DataVersion,
    to: DataVersion,
    manifest: &Manifest,
) -> Result<Vec<String>> {
    let mut targets = Vec::new();
    let mut version = from;
    while version != to {
        let mut migrations: Vec<&(DataVersion, DataVersion)> = manifest
            .migrations
            .keys()
            .filter(|(f, t)| *f == version && *t <= to)
            .collect();

        // There can be muliple paths to the same target, eg.
        //      (1.0, 1.1) => [...]
        //      (1.0, 1.2) => [...]
        // Choose one with the highest *to* version, <= our target
        migrations.sort_unstable_by(|(_, a), (_, b)| b.cmp(&a));
        if let Some(transition) = migrations.first() {
            // If a transition doesn't require a migration the array will be empty
            if let Some(migrations) = manifest.migrations.get(transition) {
                targets.extend_from_slice(&migrations);
            }
            version = transition.1;
        } else {
            return error::MissingMigration {
                current: version,
                target: to,
            }
            .fail();
        }
    }
    Ok(targets)
}

/// Store required migrations for a datastore version update in persistent
/// storage. All intermediate migrations between the current version and the
/// target version must be retrieved.
fn retrieve_migrations(
    repository: &HttpRepo<'_>,
    manifest: &Manifest,
    update: &Update,
) -> Result<()> {
    let (version_current, _) = running_version()?;
    let datastore_current =
        manifest
            .datastore_versions
            .get(&version_current)
            .context(error::MissingVersion {
                version: version_current.to_string(),
            })?;
    let datastore_target =
        manifest
            .datastore_versions
            .get(&update.version)
            .context(error::MissingVersion {
                version: update.version.to_string(),
            })?;

    if datastore_current == datastore_target {
        return Ok(());
    }

    // the migrations required for foo to bar and bar to foo are
    // the same; we can pretend we're always upgrading from foo to
    // bar and use the same logic to obtain the migrations
    let target = std::cmp::max(datastore_target, datastore_current);
    let start = std::cmp::min(datastore_target, datastore_current);

    let dir = Path::new(MIGRATION_PATH);
    if !dir.exists() {
        fs::create_dir(&dir).context(error::DirCreate { path: &dir })?;
    }
    for name in migration_targets(*start, *target, &manifest)? {
        write_target_to_disk(repository, &name, dir.join(&name))?;
    }

    Ok(())
}

fn update_image(update: &Update, repository: &HttpRepo<'_>) -> Result<()> {
    let mut gpt_state = State::load().context(error::PartitionTableRead)?;
    gpt_state.clear_inactive();
    // Write out the clearing of the inactive partition immediately, because we're about to
    // overwrite the partition set with update data and don't want it to be used until we
    // know we're done with all components.
    gpt_state.write().context(error::PartitionTableWrite)?;

    let inactive = gpt_state.inactive_set();

    // TODO Do we want to recover the inactive side on an error?
    write_target_to_disk(repository, &update.images.root, &inactive.root)?;
    write_target_to_disk(repository, &update.images.boot, &inactive.boot)?;
    write_target_to_disk(repository, &update.images.hash, &inactive.hash)?;
    Ok(())
}

fn update_flags() -> Result<()> {
    let mut gpt_state = State::load().context(error::PartitionTableRead)?;
    gpt_state.upgrade_to_inactive();
    gpt_state.write().context(error::PartitionTableWrite)?;
    Ok(())
}

/// Struct to hold the specified command line argument values
struct Arguments {
    subcommand: String,
    log_level: LevelFilter,
    json: bool,
    ignore_wave: bool,
    force_version: Option<SemVer>,
    all: bool,
    reboot: bool,
    timestamp: Option<DateTime<Utc>>,
}

/// Parse the command line arguments to get the user-specified values
fn parse_args(args: std::env::Args) -> Arguments {
    let mut subcommand = None;
    let mut log_level = None;
    let mut update_version = None;
    let mut ignore_wave = false;
    let mut json = false;
    let mut all = false;
    let mut reboot = false;
    let mut timestamp = None;

    let mut iter = args.skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_ref() {
            "--log-level" => {
                let log_level_str = iter
                    .next()
                    .unwrap_or_else(|| usage_msg("Did not give argument to --log-level"));
                log_level = Some(LevelFilter::from_str(&log_level_str).unwrap_or_else(|_| {
                    usage_msg(format!("Invalid log level '{}'", log_level_str))
                }));
            }
            "-i" | "--image" => match iter.next() {
                Some(v) => match SemVer::parse(&v) {
                    Ok(v) => update_version = Some(v),
                    _ => usage(),
                },
                _ => usage(),
            },
            "-n" | "--now" => {
                ignore_wave = true;
            }
            "-t" | "--timestamp" => match iter.next() {
                Some(t) => match DateTime::parse_from_rfc3339(&t) {
                    Ok(t) => timestamp = Some(DateTime::from_utc(t.naive_utc(), Utc)),
                    _ => usage(),
                },
                _ => usage(),
            },
            "-j" | "--json" => {
                json = true;
            }
            "-r" | "--reboot" => {
                reboot = true;
            }
            "-a" | "--all" => {
                all = true;
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
        log_level: log_level.unwrap_or_else(|| LevelFilter::Info),
        json,
        ignore_wave,
        force_version: update_version,
        all,
        reboot,
        timestamp,
    }
}

fn output<T: Serialize>(json: bool, object: T, string: &str) -> Result<()> {
    if json {
        println!(
            "{}",
            serde_json::to_string(&object).context(error::UpdateSerialize)?
        );
    } else {
        println!("{}", string);
    }
    Ok(())
}

fn main_inner() -> Result<()> {
    // Parse and store the arguments passed to the program
    let arguments = parse_args(std::env::args());

    // TerminalMode::Mixed will send errors to stderr and anything less to stdout.
    TermLogger::init(
        arguments.log_level,
        LogConfig::default(),
        TerminalMode::Mixed,
    )
    .context(error::Logger)?;

    let command =
        serde_plain::from_str::<Command>(&arguments.subcommand).unwrap_or_else(|_| usage());

    let config = load_config()?;
    let transport = HttpTransport::new();
    let repository = load_repository(&transport, &config)?;
    let manifest = load_manifest(&repository)?;
    let (current_version, flavor) = running_version().unwrap();

    match command {
        Command::CheckUpdate | Command::Whats => {
            let updates = if arguments.all {
                applicable_updates(&manifest, &flavor)
            } else if let Some(u) = update_required(
                &config,
                &manifest,
                &current_version,
                &flavor,
                arguments.force_version,
            ) {
                vec![u]
            } else {
                vec![]
            };
            if arguments.json {
                println!(
                    "{}",
                    serde_json::to_string(&updates).context(error::UpdateSerialize)?
                );
            } else {
                for u in updates {
                    if let Some(datastore_version) = manifest.datastore_versions.get(&u.version) {
                        eprintln!("{}-{} ({})", u.flavor, u.version, datastore_version);
                    } else {
                        eprintln!("{}-{} (Missing datastore mapping!)", u.flavor, u.version);
                    }
                }
            }
        }
        Command::Update | Command::UpdateImage => {
            if let Some(u) = update_required(
                &config,
                &manifest,
                &current_version,
                &flavor,
                arguments.force_version,
            ) {
                if u.update_ready(config.seed) || arguments.ignore_wave {
                    eprintln!("Starting update to {}", u.version);

                    if arguments.ignore_wave {
                        eprintln!("** Updating immediately **");
                    } else {
                        let jitter = match arguments.timestamp {
                            Some(t) => Some(t),
                            _ => u.jitter(config.seed),
                        };

                        if let Some(j) = jitter {
                            if j > Utc::now() {
                                // not yet!
                                output(arguments.json, &j, &format!("{}", j))?;
                                return Ok(());
                            }
                        }
                    }

                    retrieve_migrations(&repository, &manifest, u)?;
                    update_image(u, &repository)?;
                    if command == Command::Update {
                        update_flags()?;
                        if arguments.reboot {
                            process::Command::new("shutdown")
                                .arg("-r")
                                .status()
                                .context(error::RebootFailure)?;
                        }
                    }
                    output(
                        arguments.json,
                        &u,
                        &format!("Update applied: {}-{}", u.flavor, u.version),
                    )?;
                } else if let Some(wave) = u.jitter(config.seed) {
                    // return the jittered time of our wave in the update
                    output(
                        arguments.json,
                        &wave,
                        &format!("Update available at {}", &wave),
                    )?;
                } else {
                    eprintln!("Update available in later wave");
                }
            } else {
                eprintln!("No update required");
            }
        }
        Command::UpdateApply => {
            // TODO Guard against being called repeatedly
            update_flags()?;
            if arguments.reboot {
                process::Command::new("shutdown")
                    .arg("-r")
                    .status()
                    .context(error::RebootFailure)?;
            }
        }
        Command::Prepare => {
            // TODO unimplemented
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration as TestDuration;
    use std::collections::BTreeMap;
    use std::str::FromStr;
    use update_metadata::Images;

    #[test]
    fn test_manifest_json() {
        let s = fs::read_to_string("tests/data/example.json").unwrap();
        let manifest: Manifest = serde_json::from_str(&s).unwrap();
        assert!(
            manifest.updates.len() > 0,
            "Failed to parse update manifest"
        );

        assert!(manifest.migrations.len() > 0, "Failed to parse migrations");
        let from = DataVersion::from_str("1.0").unwrap();
        let to = DataVersion::from_str("1.1").unwrap();
        assert!(manifest.migrations.contains_key(&(from, to)));
        let migration = manifest.migrations.get(&(from, to)).unwrap();
        assert!(migration[0] == "migrate_1.1_foo");

        assert!(
            manifest.datastore_versions.len() > 0,
            "Failed to parse version map"
        );
        let thar_version = SemVer::parse("1.11.0").unwrap();
        let data_version = manifest.datastore_versions.get(&thar_version);
        let version = DataVersion::from_str("1.0").unwrap();
        assert!(data_version.is_some());
        assert!(*data_version.unwrap() == version);
    }

    #[test]
    fn test_serde_reader() {
        let file = File::open("tests/data/example_2.json").unwrap();
        let buffer = BufReader::new(file);
        let manifest: Manifest = serde_json::from_reader(buffer).unwrap();
        assert!(manifest.updates.len() > 0);
    }

    #[test]
    fn test_update_ready() {
        let config = Config {
            metadata_base_url: String::from("foo"),
            target_base_url: String::from("bar"),
            seed: 123,
        };
        let mut update = Update {
            flavor: String::from("thar"),
            arch: String::from("test"),
            version: SemVer::parse("1.0.0").unwrap(),
            max_version: SemVer::parse("1.1.0").unwrap(),
            waves: BTreeMap::new(),
            images: Images {
                boot: String::from("boot"),
                root: String::from("root"),
                hash: String::from("hash"),
            },
        };

        assert!(
            update.update_ready(config.seed),
            "No waves specified but no update"
        );

        update
            .waves
            .insert(1024, Utc::now() + TestDuration::hours(1));

        assert!(!update.update_ready(config.seed), "Incorrect wave chosen");

        update.waves.insert(0, Utc::now() - TestDuration::hours(1));

        assert!(update.update_ready(config.seed), "Update wave missed");
    }

    #[test]
    fn test_final_wave() {
        let config = Config {
            metadata_base_url: String::from("foo"),
            target_base_url: String::from("bar"),
            seed: 512,
        };
        let mut update = Update {
            flavor: String::from("thar"),
            arch: String::from("test"),
            version: SemVer::parse("1.0.0").unwrap(),
            max_version: SemVer::parse("1.1.0").unwrap(),
            waves: BTreeMap::new(),
            images: Images {
                boot: String::from("boot"),
                root: String::from("root"),
                hash: String::from("hash"),
            },
        };

        update.waves.insert(0, Utc::now() - TestDuration::hours(3));
        update
            .waves
            .insert(256, Utc::now() - TestDuration::hours(2));
        update
            .waves
            .insert(512, Utc::now() - TestDuration::hours(1));

        assert!(
            update.update_ready(config.seed),
            "All waves passed but no update"
        );
    }

    #[test]
    fn test_versions() {
        let s = fs::read_to_string("tests/data/regret.json").unwrap();
        let manifest: Manifest = serde_json::from_str(&s).unwrap();
        let config = Config {
            metadata_base_url: String::from("foo"),
            target_base_url: String::from("bar"),
            seed: 123,
        };
        // max_version is 1.20.0 in manifest
        let version = SemVer::parse("1.25.0").unwrap();
        let flavor = String::from("thar-aws-eks");

        assert!(
            update_required(&config, &manifest, &version, &flavor, None).is_none(),
            "Updog tried to exceed max_version"
        );
    }

    #[test]
    fn test_multiple() -> Result<()> {
        let s = fs::read_to_string("tests/data/multiple.json").unwrap();
        let manifest: Manifest = serde_json::from_str(&s).unwrap();
        let config = Config {
            metadata_base_url: String::from("foo"),
            target_base_url: String::from("bar"),
            seed: 123,
        };

        let version = SemVer::parse("1.10.0").unwrap();
        let flavor = String::from("thar-aws-eks");
        let result = update_required(&config, &manifest, &version, &flavor, None);

        assert!(result.is_some(), "Updog failed to find an update");

        if let Some(u) = result {
            assert!(
                u.version == SemVer::parse("1.15.0").unwrap(),
                "Incorrect version: {}, should be 1.15.0",
                u.version
            );
        }

        Ok(())
    }

    #[test]
    fn bad_bound() {
        assert!(
            serde_json::from_str::<Manifest>(include_str!("../tests/data/bad-bound.json")).is_err()
        );
    }

    #[test]
    fn duplicate_bound() {
        assert!(serde_json::from_str::<Manifest>(include_str!(
            "../tests/data/duplicate-bound.json"
        ))
        .is_err());
    }

    #[test]
    fn test_migrations() -> Result<()> {
        let s = fs::read_to_string("tests/data/migrations.json").unwrap();
        let manifest: Manifest = serde_json::from_str(&s).unwrap();

        let from = DataVersion::from_str("1.0").unwrap();
        let to = DataVersion::from_str("1.3").unwrap();
        let targets = migration_targets(from, to, &manifest)?;

        assert!(targets.len() == 3);
        let mut i = targets.iter();
        assert!(i.next().unwrap() == "migration_1.1_a");
        assert!(i.next().unwrap() == "migration_1.1_b");
        assert!(i.next().unwrap() == "migration_1.3_shortcut");
        Ok(())
    }

    #[test]
    fn serialize_metadata() -> Result<()> {
        let s = fs::read_to_string("tests/data/example_2.json").unwrap();
        let manifest: Manifest = serde_json::from_str(&s).unwrap();
        println!(
            "{}",
            serde_json::to_string_pretty(&manifest).context(error::UpdateSerialize)?
        );
        Ok(())
    }

    #[test]
    fn force_update_version() {
        let s = fs::read_to_string("tests/data/multiple.json").unwrap();
        let manifest: Manifest = serde_json::from_str(&s).unwrap();
        let config = Config {
            metadata_base_url: String::from("foo"),
            target_base_url: String::from("bar"),
            seed: 123,
        };

        let version = SemVer::parse("1.10.0").unwrap();
        let forced = SemVer::parse("1.13.0").unwrap();
        let flavor = String::from("thar-aws-eks");
        let result = update_required(&config, &manifest, &version, &flavor, Some(forced));

        assert!(result.is_some(), "Updog failed to find an update");

        if let Some(u) = result {
            assert!(
                u.version == SemVer::parse("1.13.0").unwrap(),
                "Incorrect version: {}, should be forced to 1.13.0",
                u.version
            );
        }
    }
}
