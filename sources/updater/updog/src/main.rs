#![deny(rust_2018_idioms)]
#![warn(clippy::pedantic)]

mod error;
mod transport;

use crate::error::Result;
use crate::transport::{HttpQueryRepo, HttpQueryTransport};
use bottlerocket_release::BottlerocketRelease;
use chrono::{DateTime, Utc};
use semver::Version;
use serde::{Deserialize, Serialize};
use signal_hook::{iterator::Signals, SIGTERM};
use signpost::State;
use simplelog::{Config as LogConfig, LevelFilter, TermLogger, TerminalMode};
use snafu::{ensure, ErrorCompat, OptionExt, ResultExt};
use std::fs::{self, File, OpenOptions, Permissions};
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process;
use std::str::FromStr;
use std::thread;
use tempfile::TempDir;
use tough::{ExpirationEnforcement, Limits, Repository, Settings};
use update_metadata::{find_migrations, load_manifest, Manifest, Update};

#[cfg(target_arch = "x86_64")]
const TARGET_ARCH: &str = "x86_64";
#[cfg(target_arch = "aarch64")]
const TARGET_ARCH: &str = "aarch64";

/// The root.json file as required by TUF.
const TRUSTED_ROOT_PATH: &str = "/usr/share/updog/root.json";

/// This is where we store the TUF targets used by migrator after reboot.
const MIGRATION_PATH: &str = "/var/lib/bottlerocket-migrations";

/// This is where we store the TUF metadata used by migrator after reboot.
const METADATA_PATH: &str = "/var/cache/bottlerocket-metadata";

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
    targets_base_url: String,
    seed: u32,
    // TODO API sourced configuration, eg.
    // blacklist: Option<Vec<Version>>,
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
        [ --ignore-waves ]            Ignore release schedule when checking
                                      for a new update

    prepare                 Download update files and migration targets

    update                  Perform an update if available
        [ -i | --image version ]      Update to a specfic image version
        [ -n | --now ]                Update immediately, ignoring any release schedule
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

fn load_repository<'a>(
    transport: &'a HttpQueryTransport,
    config: &'a Config,
    tough_datastore: &'a Path,
) -> Result<HttpQueryRepo<'a>> {
    fs::create_dir_all(METADATA_PATH).context(error::CreateMetadataCache {
        path: METADATA_PATH,
    })?;
    Repository::load(
        transport,
        Settings {
            root: File::open(TRUSTED_ROOT_PATH).context(error::OpenRoot {
                path: TRUSTED_ROOT_PATH,
            })?,
            datastore: tough_datastore,
            metadata_base_url: &config.metadata_base_url,
            targets_base_url: &config.targets_base_url,
            limits: Limits::default(),
            expiration_enforcement: ExpirationEnforcement::Safe,
        },
    )
    .context(error::Metadata)
}

fn applicable_updates<'a>(manifest: &'a Manifest, variant: &str) -> Vec<&'a Update> {
    let mut updates: Vec<&Update> = manifest
        .updates
        .iter()
        .filter(|u| u.variant == *variant && u.arch == TARGET_ARCH && u.version <= u.max_version)
        .collect();
    // sort descending
    updates.sort_unstable_by(|a, b| b.version.cmp(&a.version));
    updates
}

// TODO use config if there is api-sourced configuration that could affect this
// TODO updog.toml may include settings that cause us to ignore/delay
// certain/any updates;
//  Ignore Specific Target Version
//  Ignore Any Target
//  ...
fn update_required<'a>(
    _config: &Config,
    manifest: &'a Manifest,
    version: &Version,
    variant: &str,
    force_version: Option<Version>,
) -> Option<&'a Update> {
    let updates = applicable_updates(manifest, variant);

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
    repository: &HttpQueryRepo<'_>,
    target: &str,
    disk_path: P,
) -> Result<()> {
    let reader = repository
        .read_target(target)
        .context(error::Metadata)?
        .context(error::TargetNotFound { target })?;
    // Note: the file extension for the compression type we're using should be removed in
    // retrieve_migrations below.
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

/// Store required migrations for an update in persistent storage. All intermediate migrations
/// between the current version and the target version must be retrieved.
fn retrieve_migrations(
    repository: &HttpQueryRepo<'_>,
    transport: &HttpQueryTransport,
    manifest: &Manifest,
    update: &Update,
    current_version: &Version,
) -> Result<()> {
    // the migrations required for foo to bar and bar to foo are
    // the same; we can pretend we're always upgrading from foo to
    // bar and use the same logic to obtain the migrations
    let target = std::cmp::max(&update.version, &current_version);
    let start = std::cmp::min(&update.version, &current_version);

    let dir = Path::new(MIGRATION_PATH);
    if !dir.exists() {
        fs::create_dir(&dir).context(error::DirCreate { path: &dir })?;
    }

    // find the list of migrations in the manifest based on our from and to versions.
    let mut targets = find_migrations(start, target, &manifest)?;

    // DEPRECATED CODE BEGIN ///////////////////////////////////////////////////////////////////////
    // write unsigned migrations for backward compatibility. note that signed migrations will have
    // a sha prefix because we use consistent snapshots in our TUF repository. old versions of
    // migrator will ignore signed migrations because they do not match the regex, and new versions
    // of migrator will be unaffected by the presence of these unsigned migrations. this loop should
    // be removed when we no longer support backward compatibility. signed migrations will remain
    // lz4 compressed and will not be marked as executable, but unsigned migrations are uncompressed
    // and marked as executable. original comment follows...
    // download each migration, making sure they are executable and removing
    // known extensions from our compression, e.g. .lz4
    for name in &targets {
        let mut destination = dir.join(&name);
        if destination.extension() == Some("lz4".as_ref()) {
            destination.set_extension("");
        }
        write_target_to_disk(repository, &name, &destination)?;
        fs::set_permissions(&destination, Permissions::from_mode(0o755))
            .context(error::SetPermissions { path: destination })?;
    }
    // DEPRECATED CODE END /////////////////////////////////////////////////////////////////////////

    // we need to store the manifest so that migrator can independently and securely determine the
    // migration list. this is true even if there are no migrations.
    targets.push("manifest.json".to_owned());
    repository
        .cache(METADATA_PATH, MIGRATION_PATH, Some(&targets), true)
        .context(error::RepoCacheMigrations)?;
    // Set a query parameter listing the required migrations
    transport
        .queries_get_mut()
        .context(error::TransportBorrow)?
        .push(("migrations".to_owned(), targets.join(",")));

    Ok(())
}

fn update_image(update: &Update, repository: &HttpQueryRepo<'_>) -> Result<()> {
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

    gpt_state.mark_inactive_valid();
    gpt_state.write().context(error::PartitionTableWrite)?;
    Ok(())
}

fn update_flags() -> Result<()> {
    let mut gpt_state = State::load().context(error::PartitionTableRead)?;
    gpt_state
        .upgrade_to_inactive()
        .context(error::InactivePartitionUpgrade)?;
    gpt_state.write().context(error::PartitionTableWrite)?;
    Ok(())
}

fn set_common_query_params(
    transport: &HttpQueryTransport,
    current_version: &Version,
    config: &Config,
) -> Result<()> {
    let mut transport_borrow = transport
        .queries_get_mut()
        .context(error::TransportBorrow)?;

    transport_borrow.push((String::from("version"), current_version.to_string()));
    transport_borrow.push((String::from("seed"), config.seed.to_string()));

    Ok(())
}

/// List any available update that matches the current variant, ignoring waves
fn list_updates(manifest: &Manifest, variant: &str, json: bool) -> Result<()> {
    let updates = applicable_updates(manifest, variant);
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&updates).context(error::UpdateSerialize)?
        );
    } else {
        for u in updates {
            eprintln!("{}", &fmt_full_version(&u));
        }
    }
    Ok(())
}

/// Struct to hold the specified command line argument values
struct Arguments {
    subcommand: String,
    log_level: LevelFilter,
    json: bool,
    ignore_waves: bool,
    force_version: Option<Version>,
    all: bool,
    reboot: bool,
    timestamp: Option<DateTime<Utc>>,
    variant: Option<String>,
}

/// Parse the command line arguments to get the user-specified values
fn parse_args(args: std::env::Args) -> Arguments {
    let mut subcommand = None;
    let mut log_level = None;
    let mut update_version = None;
    let mut ignore_waves = false;
    let mut json = false;
    let mut all = false;
    let mut reboot = false;
    let mut timestamp = None;
    let mut variant = None;

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
                Some(v) => match Version::parse(&v) {
                    Ok(v) => update_version = Some(v),
                    _ => usage(),
                },
                _ => usage(),
            },
            "--variant" => {
                variant = Some(
                    iter.next()
                        .unwrap_or_else(|| usage_msg("Did not give argument to --variant")),
                );
            }
            "-n" | "--now" | "--ignore-waves" => {
                ignore_waves = true;
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
        ignore_waves,
        force_version: update_version,
        all,
        reboot,
        timestamp,
        variant,
    }
}

fn fmt_full_version(update: &Update) -> String {
    format!("{} {}", update.variant, update.version)
}

fn output<T: Serialize>(json: bool, object: T, string: &str) -> Result<()> {
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&object).context(error::UpdateSerialize)?
        );
    } else {
        println!("{}", string);
    }
    Ok(())
}

fn initiate_reboot() -> Result<()> {
    // Set up signal handler for termination signals
    let signals = Signals::new(&[SIGTERM]).context(error::Signal)?;
    let signals_bg = signals.clone();
    thread::spawn(move || {
        for _sig in signals_bg.forever() {
            // Ignore termination signals in case updog gets terminated
            // before getting to exit normally by itself after invoking
            // `shutdown -r` to complete the update.
        }
    });
    if let Err(err) = process::Command::new("shutdown")
        .arg("-r")
        .status()
        .context(error::RebootFailure)
    {
        // Kill the signal handling thread
        signals.close();
        return Err(err);
    }
    Ok(())
}

#[allow(clippy::too_many_lines)]
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
    let current_release = BottlerocketRelease::new().context(error::ReleaseVersion)?;
    let variant = arguments.variant.unwrap_or(current_release.variant_id);
    let transport = HttpQueryTransport::new();
    set_common_query_params(&transport, &current_release.version_id, &config)?;
    let tough_datastore = TempDir::new().context(error::CreateTempDir)?;
    let repository = load_repository(&transport, &config, tough_datastore.path())?;
    let manifest = load_manifest(&repository)?;

    match command {
        Command::CheckUpdate | Command::Whats => {
            if arguments.all {
                return list_updates(&manifest, &variant, arguments.json);
            }

            let update = update_required(
                &config,
                &manifest,
                &current_release.version_id,
                &variant,
                arguments.force_version,
            )
            .context(error::UpdateNotAvailable)?;

            if !arguments.ignore_waves {
                ensure!(
                    update.update_ready(config.seed),
                    error::UpdateNotReady {
                        version: update.version.clone()
                    }
                );
            }
            output(arguments.json, &update, &fmt_full_version(&update))?;
        }
        Command::Update | Command::UpdateImage => {
            if let Some(u) = update_required(
                &config,
                &manifest,
                &current_release.version_id,
                &variant,
                arguments.force_version,
            ) {
                if u.update_ready(config.seed) || arguments.ignore_waves {
                    eprintln!("Starting update to {}", u.version);

                    if arguments.ignore_waves {
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

                    transport
                        .queries_get_mut()
                        .context(error::TransportBorrow)?
                        .push((String::from("target"), u.version.to_string()));

                    retrieve_migrations(
                        &repository,
                        &transport,
                        &manifest,
                        u,
                        &current_release.version_id,
                    )?;
                    update_image(u, &repository)?;
                    if command == Command::Update {
                        update_flags()?;
                        if arguments.reboot {
                            initiate_reboot()?;
                        }
                    }
                    output(
                        arguments.json,
                        &u,
                        &format!("Update applied: {}", fmt_full_version(&u)),
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
            update_flags()?;
            if arguments.reboot {
                initiate_reboot()?;
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
    use update_metadata::{Images, Wave};

    #[test]
    fn test_manifest_json() {
        // Loads a general example of a manifest that includes an update with waves,
        // a set of migrations, and some datastore mappings.
        // This tests checks that it parses and the following properties are correct:
        // - the (1.0, 1.1) migrations exist with the migration "migrate_1.1_foo"
        // - the image:datastore mappings exist
        // - there is a mapping between 1.11.0 and 1.0
        let path = "tests/data/example.json";
        let manifest: Manifest = serde_json::from_reader(File::open(path).unwrap()).unwrap();
        assert!(
            manifest.updates.len() > 0,
            "Failed to parse update manifest"
        );

        assert!(manifest.migrations.len() > 0, "Failed to parse migrations");
        let from = Version::parse("1.11.0").unwrap();
        let to = Version::parse("1.12.0").unwrap();
        assert!(manifest
            .migrations
            .contains_key(&(from.clone(), to.clone())));
        let migration = manifest.migrations.get(&(from, to)).unwrap();
        assert!(migration[0] == "migrate_1.12.0_foo");
    }

    #[test]
    fn test_serde_reader() {
        // A basic manifest with a single update, no migrations, and two
        // image:datastore mappings
        let path = "tests/data/example_2.json";
        let manifest: Manifest = serde_json::from_reader(File::open(path).unwrap()).unwrap();
        assert!(manifest.updates.len() > 0);
    }

    #[test]
    fn test_update_ready() {
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

        let seed = 123;
        assert!(
            update.update_ready(seed),
            "No waves specified but no update"
        );

        update
            .waves
            .insert(1024, Utc::now() + TestDuration::hours(1));

        assert!(update.update_ready(seed), "0th wave not ready");

        update
            .waves
            .insert(100, Utc::now() + TestDuration::minutes(30));

        assert!(!update.update_ready(seed), "1st wave scheduled early");

        let early_seed = 50;
        update
            .waves
            .insert(49, Utc::now() - TestDuration::minutes(30));

        assert!(update.update_ready(early_seed), "Update wave missed");
    }

    #[test]
    fn test_final_wave() {
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

        update.waves.insert(0, Utc::now() - TestDuration::hours(3));
        update
            .waves
            .insert(256, Utc::now() - TestDuration::hours(2));
        update
            .waves
            .insert(512, Utc::now() - TestDuration::hours(1));

        assert!(update.update_ready(seed), "All waves passed but no update");
    }

    #[test]
    fn test_versions() {
        // A manifest with a single update whose version exceeds the max version.
        // update in manifest has
        // - version: 1.25.0
        // - max_version: 1.20.0
        let path = "tests/data/regret.json";
        let manifest: Manifest = serde_json::from_reader(File::open(path).unwrap()).unwrap();
        let config = Config {
            metadata_base_url: String::from("foo"),
            targets_base_url: String::from("bar"),
            seed: 123,
        };
        let version = Version::parse("1.18.0").unwrap();
        let variant = String::from("bottlerocket-aws-eks");

        assert!(
            update_required(&config, &manifest, &version, &variant, None).is_none(),
            "Updog tried to exceed max_version"
        );
    }

    #[test]
    fn older_versions() {
        // A manifest with two updates, both less than 0.1.3
        let path = "tests/data/example_3.json";
        let manifest: Manifest = serde_json::from_reader(File::open(path).unwrap()).unwrap();
        let config = Config {
            metadata_base_url: String::from("foo"),
            targets_base_url: String::from("bar"),
            seed: 1487,
        };

        let version = Version::parse("0.1.3").unwrap();
        let variant = String::from("aws-k8s-1.15");
        let update = update_required(&config, &manifest, &version, &variant, None);

        assert!(update.is_some(), "Updog ignored max version");
        assert!(
            update.unwrap().version == Version::parse("0.1.2").unwrap(),
            "Updog didn't choose the most recent valid version"
        );
    }

    #[test]
    fn test_multiple() {
        // A manifest with four updates; two valid, one which exceeds the max
        // version, and one which is for an aarch64 target. This asserts that
        // upgrading from the version 1.10.0 results in updating to 1.15.0
        // instead of 1.13.0 (lower), 1.25.0 (too high), or 1.16.0 (wrong arch).
        let path = "tests/data/multiple.json";
        let manifest: Manifest = serde_json::from_reader(File::open(path).unwrap()).unwrap();
        let config = Config {
            metadata_base_url: String::from("foo"),
            targets_base_url: String::from("bar"),
            seed: 123,
        };

        let version = Version::parse("1.10.0").unwrap();
        let variant = String::from("bottlerocket-aws-eks");
        let result = update_required(&config, &manifest, &version, &variant, None);

        assert!(result.is_some(), "Updog failed to find an update");

        if let Some(u) = result {
            assert!(
                u.version == Version::parse("1.15.0").unwrap(),
                "Incorrect version: {}, should be 1.15.0",
                u.version
            );
        }
    }

    #[test]
    fn force_update_version() {
        // A manifest with four updates; two valid, one which exceeds the max
        // version, and one which is for an aarch64 target. This tests forces
        // a downgrade to 1.13.0, instead of 1.15.0 like it would be in the
        // above test, test_multiple.
        let path = "tests/data/multiple.json";
        let manifest: Manifest = serde_json::from_reader(File::open(path).unwrap()).unwrap();
        let config = Config {
            metadata_base_url: String::from("foo"),
            targets_base_url: String::from("bar"),
            seed: 123,
        };

        let version = Version::parse("1.10.0").unwrap();
        let forced = Version::parse("1.13.0").unwrap();
        let variant = String::from("bottlerocket-aws-eks");
        let result = update_required(&config, &manifest, &version, &variant, Some(forced));

        assert!(result.is_some(), "Updog failed to find an update");

        if let Some(u) = result {
            assert!(
                u.version == Version::parse("1.13.0").unwrap(),
                "Incorrect version: {}, should be forced to 1.13.0",
                u.version
            );
        }
    }

    #[test]
    fn bad_bound() {
        // This manifest has an invalid key for one of the update's waves
        assert!(
            serde_json::from_str::<Manifest>(include_str!("../tests/data/bad-bound.json")).is_err()
        );
    }

    #[test]
    fn duplicate_bound() {
        // This manifest has two waves with a bound id of 0
        assert!(serde_json::from_str::<Manifest>(include_str!(
            "../tests/data/duplicate-bound.json"
        ))
        .is_err());
    }

    #[test]
    fn serialize_metadata() {
        // A basic manifest with a single update
        let path = "tests/data/example_2.json";
        let manifest: Manifest = serde_json::from_reader(File::open(path).unwrap()).unwrap();
        assert!(serde_json::to_string_pretty(&manifest)
            .context(error::UpdateSerialize)
            .is_ok());
    }

    #[test]
    fn early_wave() {
        let mut u = Update {
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

        // | ---- (100, "now") ---
        let first_bound = Utc::now();
        u.waves.insert(100, first_bound);
        assert!(
            u.update_wave(1).unwrap() == Wave::Initial { end: first_bound },
            "Expected to be 0th wave"
        );
        assert!(u.jitter(1).is_none(), "Expected immediate update");
        assert!(
            u.update_wave(101).unwrap() == Wave::Last { start: first_bound },
            "Expected to be final wave"
        );
        assert!(u.jitter(101).is_none(), "Expected immediate update");

        // | ---- (100, "now") ---- (200, "+1hr") ---
        let second_bound = Utc::now() + TestDuration::hours(1);
        u.waves.insert(200, second_bound);
        assert!(
            u.update_wave(1).unwrap() == Wave::Initial { end: first_bound },
            "Expected to be 0th wave"
        );
        assert!(u.jitter(1).is_none(), "Expected immediate update");

        assert!(
            u.update_wave(100).unwrap() == Wave::Initial { end: first_bound },
            "Expected to be 0th wave (just!)"
        );
        assert!(u.jitter(100).is_none(), "Expected immediate update");

        assert!(
            u.update_wave(150).unwrap()
                == Wave::General {
                    start: first_bound,
                    end: second_bound,
                },
            "Expected to be some bounded wave"
        );
        assert!(
            u.jitter(150).is_some(),
            "Expected to have to wait for update"
        );

        assert!(
            u.update_wave(201).unwrap()
                == Wave::Last {
                    start: second_bound
                },
            "Expected to be final wave"
        );
        assert!(u.jitter(201).is_none(), "Expected immediate update");
    }

    #[test]
    /// Make sure that update_ready() doesn't return true unless the client's
    /// wave is also ready.
    fn check_update_waves() {
        let mut manifest = Manifest::default();
        let mut update = Update {
            variant: String::from("aws-k8s-1.15"),
            arch: String::from(TARGET_ARCH),
            version: Version::parse("1.1.1").unwrap(),
            max_version: Version::parse("1.1.1").unwrap(),
            waves: BTreeMap::new(),
            images: Images {
                boot: String::from("boot"),
                root: String::from("boot"),
                hash: String::from("boot"),
            },
        };

        let current_version = Version::parse("1.0.0").unwrap();
        let variant = String::from("aws-k8s-1.15");
        let config = Config {
            metadata_base_url: String::from("foo"),
            targets_base_url: String::from("bar"),
            seed: 512,
        };

        // Two waves; the 0th wave, and the final wave which starts in one hour
        update
            .waves
            .insert(1024, Utc::now() + TestDuration::hours(1));
        manifest.updates.push(update);

        let potential_update =
            update_required(&config, &manifest, &current_version, &variant, None).unwrap();

        assert!(
            potential_update.update_ready(512),
            "0th wave doesn't appear ready"
        );
        assert!(
            !potential_update.update_ready(2000),
            "Later wave incorrectly sees update"
        );
    }
}
