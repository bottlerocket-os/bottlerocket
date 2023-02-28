#![warn(clippy::pedantic)]

mod error;
mod transport;

use crate::error::Result;
use crate::transport::{HttpQueryTransport, QueryParams};
use bottlerocket_release::BottlerocketRelease;
use chrono::Utc;
use log::debug;
use model::modeled_types::FriendlyVersion;
use semver::Version;
use serde::{Deserialize, Serialize};
use signal_hook::consts::SIGTERM;
use signal_hook::iterator::Signals;
use signpost::State;
use simplelog::{Config as LogConfig, LevelFilter, SimpleLogger};
use snafu::{ErrorCompat, OptionExt, ResultExt};
use std::convert::{TryFrom, TryInto};
use std::fs::{self, File, OpenOptions};
use std::io;
use std::path::Path;
use std::process;
use std::str::FromStr;
use std::thread;
use tough::{Repository, RepositoryLoader};
use update_metadata::{find_migrations, Manifest, Update};
use url::Url;

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
    UpdateRevert,
}

#[derive(Debug, Deserialize)]
struct Config {
    metadata_base_url: String,
    targets_base_url: String,
    seed: u32,
    version_lock: String,
    ignore_waves: bool,
    https_proxy: Option<String>,
    no_proxy: Option<Vec<String>>,
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
        [ -a | --all ]                Output all available updates, even if they're not upgrades
        [ --ignore-waves ]            Ignore release schedule when checking
                                      for a new update

    prepare                 Download update files and migration targets

    update                  Perform an update if available
        [ -i | --image version ]      Update to a specific image version
        [ -n | --now ]                Update immediately, ignoring any release schedule
        [ -r | --reboot ]             Reboot into new update on success

    update-image            Download & write an update but do not update flags
        [ -i | --image version ]      Update to a specific image version
        [ -n | --now ]                Update immediately, ignoring wave limits
        [ -t | --timestamp time ]     The timestamp to execute an update from

    update-apply            Update boot flags (after having called update-image)
        [ -r | --reboot ]             Reboot after updating boot flags

    update-revert           Revert actions done by 'update-apply'

GLOBAL OPTIONS:
    [ -j | --json ]               JSON-formatted output
    [ --log-level trace|debug|info|warn|error ]  Set logging verbosity");
    std::process::exit(1)
}

fn load_config() -> Result<Config> {
    let path = "/etc/updog.toml";
    let s = fs::read_to_string(path).context(error::ConfigReadSnafu { path })?;
    let config: Config = toml::from_str(&s).context(error::ConfigParseSnafu { path })?;
    Ok(config)
}

fn load_repository(transport: HttpQueryTransport, config: &Config) -> Result<Repository> {
    fs::create_dir_all(METADATA_PATH).context(error::CreateMetadataCacheSnafu {
        path: METADATA_PATH,
    })?;
    RepositoryLoader::new(
        File::open(TRUSTED_ROOT_PATH).context(error::OpenRootSnafu {
            path: TRUSTED_ROOT_PATH,
        })?,
        Url::parse(&config.metadata_base_url).context(error::UrlParseSnafu {
            url: &config.metadata_base_url,
        })?,
        Url::parse(&config.targets_base_url).context(error::UrlParseSnafu {
            url: &config.targets_base_url,
        })?,
    )
    .transport(transport)
    .load()
    .context(error::MetadataSnafu)
}

fn applicable_updates<'a>(
    manifest: &'a Manifest,
    variant: &str,
    ignore_waves: bool,
    seed: u32,
) -> Vec<&'a Update> {
    let mut updates: Vec<&Update> = manifest
        .updates
        .iter()
        .filter(|u| {
            u.variant == *variant
                && u.arch == TARGET_ARCH
                && u.version <= u.max_version
                && (ignore_waves || u.update_ready(seed, Utc::now()))
        })
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
    manifest: &'a Manifest,
    version: &Version,
    variant: &str,
    ignore_waves: bool,
    seed: u32,
    version_lock: &str,
    force_version: Option<Version>,
) -> Result<Option<&'a Update>> {
    let updates = applicable_updates(manifest, variant, ignore_waves, seed);

    if let Some(forced_version) = force_version {
        return Ok(updates.into_iter().find(|u| u.version == forced_version));
    }

    if version_lock != "latest" {
        // Make sure the version string from the config is a valid version string that might be prefixed with 'v'
        let friendly_version_lock =
            FriendlyVersion::try_from(version_lock).context(error::BadVersionConfigSnafu {
                version_str: version_lock,
            })?;
        // Convert back to semver::Version
        let semver_version_lock =
            friendly_version_lock
                .try_into()
                .context(error::BadVersionSnafu {
                    version_str: version_lock,
                })?;
        // If the configured version-lock matches our current version, we won't update to the same version
        return if semver_version_lock == *version {
            Ok(None)
        } else {
            Ok(updates
                .into_iter()
                .find(|u| u.version == semver_version_lock))
        };
    }

    for update in updates {
        // If the current running version is greater than the max version ever published,
        // or moves us to a valid version <= the maximum version, update.
        if *version < update.version || *version > update.max_version {
            return Ok(Some(update));
        }
    }
    Ok(None)
}

fn write_target_to_disk<P: AsRef<Path>>(
    repository: &Repository,
    target: &str,
    disk_path: P,
) -> Result<()> {
    let target = target
        .try_into()
        .context(error::TargetNameSnafu { target })?;
    let reader = repository
        .read_target(&target)
        .context(error::MetadataSnafu)?
        .context(error::TargetNotFoundSnafu {
            target: target.raw(),
        })?;
    // Note: the file extension for the compression type we're using should be removed in
    // retrieve_migrations below.
    let mut reader = lz4::Decoder::new(reader).context(error::Lz4DecodeSnafu {
        target: target.raw(),
    })?;
    let mut f = OpenOptions::new()
        .write(true)
        .create(true)
        .open(disk_path.as_ref())
        .context(error::OpenPartitionSnafu {
            path: disk_path.as_ref(),
        })?;
    io::copy(&mut reader, &mut f).context(error::WriteUpdateSnafu)?;
    Ok(())
}

/// Store required migrations for an update in persistent storage. All intermediate migrations
/// between the current version and the target version must be retrieved.
fn retrieve_migrations(
    repository: &Repository,
    query_params: &mut QueryParams,
    manifest: &Manifest,
    update: &Update,
    current_version: &Version,
) -> Result<()> {
    // the migrations required for foo to bar and bar to foo are
    // the same; we can pretend we're always upgrading from foo to
    // bar and use the same logic to obtain the migrations
    let target = std::cmp::max(&update.version, current_version);
    let start = std::cmp::min(&update.version, current_version);

    let dir = Path::new(MIGRATION_PATH);
    if !dir.exists() {
        fs::create_dir(dir).context(error::DirCreateSnafu { path: &dir })?;
    }

    // find the list of migrations in the manifest based on our from and to versions.
    let mut targets = find_migrations(start, target, manifest)?;

    // we need to store the manifest so that migrator can independently and securely determine the
    // migration list. this is true even if there are no migrations.
    targets.push("manifest.json".to_owned());
    repository
        .cache(METADATA_PATH, MIGRATION_PATH, Some(&targets), true)
        .context(error::RepoCacheMigrationsSnafu)?;
    // Set a query parameter listing the required migrations
    query_params.add("migrations", targets.join(","));
    Ok(())
}

fn update_image(update: &Update, repository: &Repository) -> Result<()> {
    let mut gpt_state = State::load().context(error::PartitionTableReadSnafu)?;
    gpt_state.clear_inactive();
    // Write out the clearing of the inactive partition immediately, because we're about to
    // overwrite the partition set with update data and don't want it to be used until we
    // know we're done with all components.
    gpt_state.write().context(error::PartitionTableWriteSnafu)?;

    let inactive = gpt_state.inactive_set();

    // TODO Do we want to recover the inactive side on an error?
    write_target_to_disk(repository, &update.images.root, &inactive.root)?;
    write_target_to_disk(repository, &update.images.boot, &inactive.boot)?;
    write_target_to_disk(repository, &update.images.hash, &inactive.hash)?;

    gpt_state.mark_inactive_valid();
    gpt_state.write().context(error::PartitionTableWriteSnafu)?;
    Ok(())
}

fn update_flags() -> Result<()> {
    let mut gpt_state = State::load().context(error::PartitionTableReadSnafu)?;
    gpt_state
        .upgrade_to_inactive()
        .context(error::InactivePartitionUpgradeSnafu)?;
    gpt_state.write().context(error::PartitionTableWriteSnafu)?;
    Ok(())
}

fn revert_update_flags() -> Result<()> {
    let mut gpt_state = State::load().context(error::PartitionTableReadSnafu)?;
    gpt_state.cancel_upgrade();
    gpt_state.write().context(error::PartitionTableWriteSnafu)?;
    Ok(())
}

fn set_common_query_params(
    query_params: &mut QueryParams,
    current_version: &Version,
    config: &Config,
) {
    query_params.add("version", current_version.to_string());
    query_params.add("seed", config.seed.to_string());
}

/// List any available update that matches the current variant
fn list_updates(
    manifest: &Manifest,
    variant: &str,
    json: bool,
    ignore_waves: bool,
    seed: u32,
) -> Result<()> {
    let updates = applicable_updates(manifest, variant, ignore_waves, seed);
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&updates).context(error::UpdateSerializeSnafu)?
        );
    } else {
        for u in updates {
            eprintln!("{}", &fmt_full_version(u));
        }
    }
    Ok(())
}

/// Struct to hold the specified command line argument values
#[allow(clippy::struct_excessive_bools)]
struct Arguments {
    subcommand: String,
    log_level: LevelFilter,
    json: bool,
    ignore_waves: bool,
    force_version: Option<Version>,
    all: bool,
    reboot: bool,
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
    let mut variant = None;

    let mut iter = args.skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_ref() {
            "--log-level" => {
                let log_level_str = iter
                    .next()
                    .unwrap_or_else(|| usage_msg("Did not give argument to --log-level"));
                log_level =
                    Some(LevelFilter::from_str(&log_level_str).unwrap_or_else(|_| {
                        usage_msg(format!("Invalid log level '{log_level_str}'"))
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
        log_level: log_level.unwrap_or(LevelFilter::Info),
        json,
        ignore_waves,
        force_version: update_version,
        all,
        reboot,
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
            serde_json::to_string_pretty(&object).context(error::UpdateSerializeSnafu)?
        );
    } else {
        println!("{string}");
    }
    Ok(())
}

fn initiate_reboot() -> Result<()> {
    // Set up signal handler for termination signals
    let mut signals = Signals::new([SIGTERM]).context(error::SignalSnafu)?;
    let signals_handle = signals.handle();
    thread::spawn(move || {
        for _sig in signals.forever() {
            // Ignore termination signals in case updog gets terminated
            // before getting to exit normally by itself after invoking
            // `shutdown -r` to complete the update.
        }
    });
    if let Err(err) = process::Command::new("shutdown")
        .arg("-r")
        .status()
        .context(error::RebootFailureSnafu)
    {
        // Kill the signal handling thread
        signals_handle.close();
        return Err(err);
    }
    Ok(())
}

/// Our underlying HTTP client, reqwest, supports proxies by reading the `HTTPS_PROXY` and `NO_PROXY`
/// environment variables. Bottlerocket services can source proxy.env before running, but updog is
/// not a service, so we read these values from the config file and add them to the environment
/// here.
fn set_https_proxy_environment_variables(
    https_proxy: &Option<String>,
    no_proxy: &Option<Vec<String>>,
) {
    let proxy = match https_proxy {
        Some(s) if !s.is_empty() => s.clone(),
        // without https_proxy, no_proxy does nothing, so we are done
        _ => return,
    };

    std::env::set_var("HTTPS_PROXY", proxy);
    if let Some(no_proxy) = no_proxy {
        if !no_proxy.is_empty() {
            let no_proxy_string = no_proxy.join(",");
            debug!("setting NO_PROXY={}", no_proxy_string);
            std::env::set_var("NO_PROXY", &no_proxy_string);
        }
    }
}

#[allow(clippy::too_many_lines)]
fn main_inner() -> Result<()> {
    // Parse and store the arguments passed to the program
    let arguments = parse_args(std::env::args());

    // SimpleLogger will send errors to stderr and anything less to stdout.
    SimpleLogger::init(arguments.log_level, LogConfig::default()).context(error::LoggerSnafu)?;

    let command =
        serde_plain::from_str::<Command>(&arguments.subcommand).unwrap_or_else(|_| usage());

    let config = load_config()?;
    set_https_proxy_environment_variables(&config.https_proxy, &config.no_proxy);
    let current_release = BottlerocketRelease::new().context(error::ReleaseVersionSnafu)?;
    let variant = arguments.variant.unwrap_or(current_release.variant_id);
    let transport = HttpQueryTransport::new();
    // get a shared pointer to the transport's query_params so we can add metrics information to
    // the transport's HTTP calls.
    let mut query_params = transport.query_params();
    set_common_query_params(&mut query_params, &current_release.version_id, &config);
    let repository = load_repository(transport, &config)?;
    let manifest = load_manifest(&repository)?;
    let ignore_waves = arguments.ignore_waves || config.ignore_waves;
    match command {
        Command::CheckUpdate | Command::Whats => {
            if arguments.all {
                return list_updates(
                    &manifest,
                    &variant,
                    arguments.json,
                    ignore_waves,
                    config.seed,
                );
            }

            let update = update_required(
                &manifest,
                &current_release.version_id,
                &variant,
                ignore_waves,
                config.seed,
                &config.version_lock,
                arguments.force_version,
            )?
            .context(error::UpdateNotAvailableSnafu)?;

            output(arguments.json, update, &fmt_full_version(update))?;
        }
        Command::Update | Command::UpdateImage => {
            if let Some(u) = update_required(
                &manifest,
                &current_release.version_id,
                &variant,
                ignore_waves,
                config.seed,
                &config.version_lock,
                arguments.force_version,
            )? {
                eprintln!("Starting update to {}", u.version);
                query_params.add("target", u.version.to_string());
                retrieve_migrations(
                    &repository,
                    &mut query_params,
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
                    u,
                    &format!("Update applied: {}", fmt_full_version(u)),
                )?;
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
        Command::UpdateRevert => {
            revert_update_flags()?;
        }
        Command::Prepare => {
            // TODO unimplemented
        }
    }

    Ok(())
}

fn load_manifest(repository: &tough::Repository) -> Result<Manifest> {
    let target = "manifest.json";
    let target = target
        .try_into()
        .context(error::TargetNameSnafu { target })?;
    Manifest::from_json(
        repository
            .read_target(&target)
            .context(error::ManifestLoadSnafu)?
            .context(error::ManifestNotFoundSnafu)?,
    )
    .context(error::ManifestParseSnafu)
}

fn main() -> ! {
    std::process::exit(match main_inner() {
        Ok(()) => 0,
        Err(err) => {
            eprintln!("{err}");
            if let Some(var) = std::env::var_os("RUST_BACKTRACE") {
                if var != "0" {
                    if let Some(backtrace) = err.backtrace() {
                        eprintln!("\n{backtrace:?}");
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
    use update_metadata::Images;

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
            !manifest.updates.is_empty(),
            "Failed to parse update manifest"
        );

        assert!(
            !manifest.migrations.is_empty(),
            "Failed to parse migrations"
        );
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
        assert!(!manifest.updates.is_empty());
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
            version_lock: "latest".to_string(),
            ignore_waves: false,
            https_proxy: None,
            no_proxy: None,
        };
        let version = Version::parse("1.18.0").unwrap();
        let variant = String::from("bottlerocket-aws-eks");

        assert!(
            update_required(
                &manifest,
                &version,
                &variant,
                config.ignore_waves,
                config.seed,
                &config.version_lock,
                None
            )
            .unwrap()
            .is_none(),
            "Updog tried to exceed max_version"
        );
    }

    #[test]
    fn older_versions() {
        // A manifest with two updates, both less than 0.1.3.
        // Use a architecture specific JSON payload, otherwise updog will ignore the update
        let path = format!("tests/data/example_3_{TARGET_ARCH}.json");
        let manifest: Manifest = serde_json::from_reader(File::open(path).unwrap()).unwrap();
        let config = Config {
            metadata_base_url: String::from("foo"),
            targets_base_url: String::from("bar"),
            seed: 1487,
            version_lock: "latest".to_string(),
            ignore_waves: false,
            https_proxy: None,
            no_proxy: None,
        };

        let version = Version::parse("0.1.3").unwrap();
        let variant = String::from("aws-k8s-1.15");
        let update = update_required(
            &manifest,
            &version,
            &variant,
            config.ignore_waves,
            config.seed,
            &config.version_lock,
            None,
        )
        .unwrap();

        assert!(update.is_some(), "Updog ignored max version");
        assert!(
            update.unwrap().version == Version::parse("0.1.2").unwrap(),
            "Updog didn't choose the most recent valid version"
        );
    }

    #[test]
    fn test_multiple() {
        // A manifest with four updates; two valid, one which exceeds the max
        // version, and one which is for the opposite target architecture. This asserts that
        // upgrading from the version 1.10.0 results in updating to 1.15.0
        // instead of 1.13.0 (lower), 1.25.0 (too high), or 1.16.0 (wrong arch).
        let path = format!("tests/data/multiple_{TARGET_ARCH}.json");
        let manifest: Manifest = serde_json::from_reader(File::open(path).unwrap()).unwrap();
        let config = Config {
            metadata_base_url: String::from("foo"),
            targets_base_url: String::from("bar"),
            seed: 123,
            version_lock: "latest".to_string(),
            ignore_waves: false,
            https_proxy: None,
            no_proxy: None,
        };

        let version = Version::parse("1.10.0").unwrap();
        let variant = String::from("bottlerocket-aws-eks");
        let result = update_required(
            &manifest,
            &version,
            &variant,
            config.ignore_waves,
            config.seed,
            &config.version_lock,
            None,
        )
        .unwrap();

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
        // version, and one which is for the opposite target architecture. This tests forces
        // a downgrade to 1.13.0, instead of 1.15.0 like it would be in the
        // above test, test_multiple.
        let path = format!("tests/data/multiple_{TARGET_ARCH}.json");
        let manifest: Manifest = serde_json::from_reader(File::open(path).unwrap()).unwrap();
        let config = Config {
            metadata_base_url: String::from("foo"),
            targets_base_url: String::from("bar"),
            seed: 123,
            version_lock: "latest".to_string(),
            ignore_waves: false,
            https_proxy: None,
            no_proxy: None,
        };

        let version = Version::parse("1.10.0").unwrap();
        let forced = Version::parse("1.13.0").unwrap();
        let variant = String::from("bottlerocket-aws-eks");
        let result = update_required(
            &manifest,
            &version,
            &variant,
            config.ignore_waves,
            config.seed,
            &config.version_lock,
            Some(forced),
        )
        .unwrap();

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
            .context(error::UpdateSerializeSnafu)
            .is_ok());
    }

    #[test]
    /// Make sure that `update_required()` doesn't return true unless the client's
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
        let first_wave_seed = 0;
        let config = Config {
            metadata_base_url: String::from("foo"),
            targets_base_url: String::from("bar"),
            seed: first_wave_seed,
            version_lock: "latest".to_string(),
            ignore_waves: false,
            https_proxy: None,
            no_proxy: None,
        };

        // Two waves; the 1st wave that starts immediately, and the final wave which starts in one hour
        let time = Utc::now();
        update.waves.insert(0, time);
        update.waves.insert(1024, time + TestDuration::hours(1));
        update.waves.insert(2048, time + TestDuration::hours(1));
        manifest.updates.push(update);

        assert!(
            update_required(
                &manifest,
                &current_version,
                &variant,
                config.ignore_waves,
                config.seed,
                &config.version_lock,
                None,
            )
            .unwrap()
            .is_some(),
            "1st wave doesn't appear ready"
        );

        assert!(
            update_required(
                &manifest,
                &current_version,
                &variant,
                config.ignore_waves,
                2000,
                &config.version_lock,
                None,
            )
            .unwrap()
            .is_none(),
            "Later wave incorrectly sees update"
        );
    }
}
