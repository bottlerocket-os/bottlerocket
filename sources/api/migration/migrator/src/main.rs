//! migrator is a tool to run migrations built with the migration-helpers library.
//!
//! It must be given:
//! * a data store to migrate
//! * a version to migrate it to
//! * where to find migration binaries
//!
//! Given those, it will:
//! * confirm that the given data store has the appropriate versioned symlink structure
//! * find the version of the given data store
//! * find migrations between the two versions
//! * if there are migrations:
//!   * run the migrations; the transformed data becomes the new data store
//! * if there are *no* migrations:
//!   * just symlink to the old data store
//! * do symlink flips so the new version takes the place of the original
//!
//! To understand motivation and more about the overall process, look at the migration system
//! documentation, one level up.

#![deny(rust_2018_idioms)]

#[macro_use]
extern crate log;

use nix::{dir::Dir, fcntl::OFlag, sys::stat::Mode, unistd::fsync};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use semver::Version;
use simplelog::{Config as LogConfig, TermLogger, TerminalMode};
use snafu::{ensure, OptionExt, ResultExt};
use std::collections::HashSet;
use std::env;
use std::fs::{self, Permissions};
use std::os::unix::fs::{symlink, PermissionsExt};
use std::os::unix::io::AsRawFd;
use std::path::{Path, PathBuf};
use std::process::{self, Command};

use update_metadata::MIGRATION_FILENAME_RE;
mod args;
mod direction;
mod error;

use args::Args;
use direction::Direction;
use error::Result;

// Returning a Result from main makes it print a Debug representation of the error, but with Snafu
// we have nice Display representations of the error, so we wrap "main" (run) and print any error.
// https://github.com/shepmaster/snafu/issues/110
fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = Args::from_env(env::args());

    // TerminalMode::Mixed will send errors to stderr and anything less to stdout.
    TermLogger::init(args.log_level, LogConfig::default(), TerminalMode::Mixed)
        .context(error::Logger)?;

    // Get the directory we're working in.
    let datastore_dir = args
        .datastore_path
        .parent()
        .context(error::DataStoreLinkToRoot {
            path: &args.datastore_path,
        })?;

    let current_version = get_current_version(&datastore_dir)?;

    let direction = Direction::from_versions(&current_version, &args.migrate_to_version)
        .unwrap_or_else(|| {
            info!(
                "Requested version {} matches version of given datastore at '{}'; nothing to do",
                args.migrate_to_version,
                args.datastore_path.display()
            );
            process::exit(0);
        });

    let migrations = find_migrations(
        &args.migration_directories,
        &current_version,
        &args.migrate_to_version,
    )?;

    if migrations.is_empty() {
        // Not all new OS versions need to change the data store format.  If there's been no
        // change, we can just link to the last version rather than making a copy.
        // (Note: we link to the fully resolved directory, args.datastore_path,  so we don't
        // have a chain of symlinks that could go past the maximum depth.)
        flip_to_new_version(&args.migrate_to_version, &args.datastore_path)?;
    } else {
        let copy_path = run_migrations(
            direction,
            &migrations,
            &args.datastore_path,
            &args.migrate_to_version,
        )?;
        flip_to_new_version(&args.migrate_to_version, &copy_path)?;
    }

    Ok(())
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

fn get_current_version<P>(datastore_dir: P) -> Result<Version>
where
    P: AsRef<Path>,
{
    let datastore_dir = datastore_dir.as_ref();

    // Find the current patch version link, which contains our full version number
    let current = datastore_dir.join("current");
    let major =
        datastore_dir.join(fs::read_link(&current).context(error::LinkRead { link: current })?);
    let minor = datastore_dir.join(fs::read_link(&major).context(error::LinkRead { link: major })?);
    let patch = datastore_dir.join(fs::read_link(&minor).context(error::LinkRead { link: minor })?);

    // Pull out the basename of the path, which contains the version
    let version_os_str = patch
        .file_name()
        .context(error::DataStoreLinkToRoot { path: &patch })?;
    let mut version_str = version_os_str
        .to_str()
        .context(error::DataStorePathNotUTF8 { path: &patch })?;

    // Allow 'v' at the start so the links have clearer names for humans
    if version_str.starts_with('v') {
        version_str = &version_str[1..];
    }

    Version::parse(version_str).context(error::InvalidDataStoreVersion { path: &patch })
}

/// Returns a list of all migrations found on disk.
///
/// TODO: This does not yet handle migrations that have been replaced by newer versions - we only
/// look in one fixed location. We need to get the list of migrations from update metadata, and
/// only return those.  That may also obviate the need for select_migrations.
fn find_migrations_on_disk<P>(dir: P) -> Result<Vec<PathBuf>>
where
    P: AsRef<Path>,
{
    let dir = dir.as_ref();
    let mut result = Vec::new();

    trace!("Looking for potential migrations in {}", dir.display());
    let entries = fs::read_dir(dir).context(error::ListMigrations { dir })?;
    for entry in entries {
        let entry = entry.context(error::ReadMigrationEntry)?;
        let path = entry.path();

        // Just check that it's a file; other checks to determine whether we should actually run
        // a file we find are done by select_migrations.
        let file_type = entry
            .file_type()
            .context(error::PathMetadata { path: &path })?;
        if !file_type.is_file() {
            debug!(
                "Skipping non-file in migration directory: {}",
                path.display()
            );
            continue;
        }

        trace!("Found potential migration: {}", path.display());
        result.push(path);
    }

    Ok(result)
}

/// Returns the sublist of the given migrations that should be run, in the returned order, to move
/// from the 'from' version to the 'to' version.
fn select_migrations<P: AsRef<Path>>(
    from: &Version,
    to: &Version,
    paths: &[P],
) -> Result<Vec<PathBuf>> {
    // Intermediate result where we also store the version and name, needed for sorting
    let mut sortable: Vec<(Version, String, PathBuf)> = Vec::new();

    for path in paths {
        let path = path.as_ref();

        // We pull the applicable version and the migration name out of the filename.
        let file_name = path
            .file_name()
            .context(error::Internal {
                msg: "Found '/' as migration",
            })?
            .to_str()
            .context(error::MigrationNameNotUTF8 { path: &path })?;
        let captures = match MIGRATION_FILENAME_RE.captures(&file_name) {
            Some(captures) => captures,
            None => {
                debug!(
                    "Skipping non-migration (bad name) in migration directory: {}",
                    path.display()
                );
                continue;
            }
        };

        let version_match = captures.name("version").context(error::Internal {
            msg: "Migration name matched regex but we don't have a 'version' capture",
        })?;
        let version = Version::parse(version_match.as_str())
            .context(error::InvalidMigrationVersion { path: &path })?;

        let name_match = captures.name("name").context(error::Internal {
            msg: "Migration name matched regex but we don't have a 'name' capture",
        })?;
        let name = name_match.as_str().to_string();

        // We don't want to include migrations for the "from" version we're already on.
        // Note on possible confusion: when going backward it's the higher version that knows
        // how to undo its changes and take you to the lower version.  For example, the v2
        // migration knows what changes it made to go from v1 to v2 and therefore how to go
        // back from v2 to v1.  See tests.
        let applicable = if to > from && version > *from && version <= *to {
            info!(
                "Found applicable forward migration '{}': {} < ({}) <= {}",
                file_name, from, version, to
            );
            true
        } else if to < from && version > *to && version <= *from {
            info!(
                "Found applicable backward migration '{}': {} >= ({}) > {}",
                file_name, from, version, to
            );
            true
        } else {
            debug!(
                "Migration '{}' doesn't apply when going from {} to {}",
                file_name, from, to
            );
            false
        };

        if applicable {
            sortable.push((version, name, path.to_path_buf()));
        }
    }

    // Sort the migrations using the metadata we stored -- version first, then name so that
    // authors have some ordering control if necessary.
    sortable.sort_unstable();

    // For a Backward migration process, reverse the order.
    if to < from {
        sortable.reverse();
    }

    debug!(
        "Sorted migrations: {:?}",
        sortable
            .iter()
            // Want filename, which always applies for us, but fall back to name just in case
            .map(|(_version, name, path)| path
                .file_name()
                .map(|osstr| osstr.to_string_lossy().into_owned())
                .unwrap_or_else(|| name.to_string()))
            .collect::<Vec<String>>()
    );

    // Get rid of the name; only needed it as a separate component for sorting
    let result: Vec<PathBuf> = sortable
        .into_iter()
        .map(|(_version, _name, path)| path)
        .collect();

    Ok(result)
}

/// Given the versions we're migrating from and to, this will return an ordered list of paths to
/// migration binaries we should run to complete the migration on a data store.
// This separation allows for easy testing of select_migrations.
fn find_migrations<P>(paths: &[P], from: &Version, to: &Version) -> Result<Vec<PathBuf>>
where
    P: AsRef<Path>,
{
    let mut candidates = Vec::new();
    for path in paths {
        candidates.extend(find_migrations_on_disk(path)?);
    }
    select_migrations(from, to, &candidates)
}

/// Generates a random ID, affectionately known as a 'rando', that can be used to avoid timing
/// issues and identify unique migration attempts.
fn rando() -> String {
    thread_rng().sample_iter(&Alphanumeric).take(16).collect()
}

/// Generates a path for a new data store, given the path of the existing data store,
/// the new version number, and a random "copy id" to append.
fn new_datastore_location<P>(from: P, new_version: &Version) -> Result<PathBuf>
where
    P: AsRef<Path>,
{
    let to = from
        .as_ref()
        .with_file_name(format!("v{}_{}", new_version, rando()));
    ensure!(
        !to.exists(),
        error::NewVersionAlreadyExists {
            version: new_version.clone(),
            path: to
        }
    );

    info!(
        "New data store is being built at work location {}",
        to.display()
    );
    Ok(to)
}

/// Runs the given migrations in their given order.  The given direction is passed to each
/// migration so it knows which direction we're migrating.
///
/// The given data store is used as a starting point; each migration is given the output of the
/// previous migration, and the final output becomes the new data store.
fn run_migrations<P1, P2>(
    direction: Direction,
    migrations: &[P1],
    source_datastore: P2,
    new_version: &Version,
) -> Result<PathBuf>
where
    P1: AsRef<Path>,
    P2: AsRef<Path>,
{
    // We start with the given source_datastore, updating this after each migration to point to the
    // output of the previous one.
    let mut source_datastore = source_datastore.as_ref();
    // We create a new data store (below) to serve as the target of each migration.  (Start at
    // source just to have the right type; we know we have migrations at this point.)
    let mut target_datastore = source_datastore.to_owned();
    // Any data stores we create that aren't the final one, i.e. intermediate data stores, will be
    // removed at the end.  (If we fail and return early, they're left for debugging purposes.)
    let mut intermediate_datastores = HashSet::new();

    for migration in migrations {
        // Ensure the migration is executable.
        fs::set_permissions(migration.as_ref(), Permissions::from_mode(0o755)).context(
            error::SetPermissions {
                path: migration.as_ref(),
            },
        )?;

        let mut command = Command::new(migration.as_ref());

        // Point each migration in the right direction, and at the given data store.
        command.arg(direction.to_string());
        command.args(&[
            "--source-datastore".to_string(),
            source_datastore.display().to_string(),
        ]);

        // Create a new output location for this migration.
        target_datastore = new_datastore_location(&source_datastore, &new_version)?;
        intermediate_datastores.insert(target_datastore.clone());

        command.args(&[
            "--target-datastore".to_string(),
            target_datastore.display().to_string(),
        ]);

        info!("Running migration command: {:?}", command);

        let output = command
            .output()
            .context(error::StartMigration { command })?;

        if !output.stdout.is_empty() {
            debug!(
                "Migration stdout: {}",
                std::str::from_utf8(&output.stdout).unwrap_or("<invalid UTF-8>")
            );
        } else {
            debug!("No migration stdout");
        }
        if !output.stderr.is_empty() {
            let stderr = std::str::from_utf8(&output.stderr).unwrap_or("<invalid UTF-8>");
            // We want to see migration stderr on the console, so log at error level.
            error!("Migration stderr: {}", stderr);
        } else {
            debug!("No migration stderr");
        }

        ensure!(output.status.success(), error::MigrationFailure { output });

        source_datastore = &target_datastore;
    }

    // Remove the intermediate data stores
    intermediate_datastores.remove(&target_datastore);
    for intermediate_datastore in intermediate_datastores {
        // Even if we fail to remove an intermediate data store, we've still migrated
        // successfully, and we don't want to fail the upgrade - just let someone know for
        // later cleanup.
        trace!("Removing intermediate data store at {}", intermediate_datastore.display());
        if let Err(e) = fs::remove_dir_all(&intermediate_datastore) {
            error!(
                "Failed to remove intermediate data store at '{}': {}",
                intermediate_datastore.display(),
                e
            );
        }
    }

    Ok(target_datastore)
}

/// Atomically flips version symlinks to point to the given "to" datastore so that it becomes live.
///
/// This includes:
/// * pointing the new patch version to the given `to_datastore`
/// * pointing the minor version to the patch version
/// * pointing the major version to the minor version
/// * pointing the 'current' link to the major version
/// * fsyncing the directory to disk
fn flip_to_new_version<P>(version: &Version, to_datastore: P) -> Result<()>
where
    P: AsRef<Path>,
{
    // Get the directory we're working in.
    let to_dir = to_datastore
        .as_ref()
        .parent()
        .context(error::DataStoreLinkToRoot {
            path: to_datastore.as_ref(),
        })?;
    // We need a file descriptor for the directory so we can fsync after the symlink swap.
    let raw_dir = Dir::open(
        to_dir,
        // Confirm it's a directory
        OFlag::O_DIRECTORY,
        // (mode doesn't matter for opening a directory)
        Mode::empty(),
    )
    .context(error::DataStoreDirOpen { path: &to_dir })?;

    // Get a unique temporary path in the directory; we need this to atomically swap.
    let temp_link = to_dir.join(rando());
    // Build the path to the 'current' link; this is what we're atomically swapping from
    // pointing at the old major version to pointing at the new major version.
    // Example: /path/to/datastore/current
    let current_version_link = to_dir.join("current");
    // Build the path to the major version link; this is what we're atomically swapping from
    // pointing at the old minor version to pointing at the new minor version.
    // Example: /path/to/datastore/v1
    let major_version_link = to_dir.join(format!("v{}", version.major));
    // Build the path to the minor version link; this is what we're atomically swapping from
    // pointing at the old patch version to pointing at the new patch version.
    // Example: /path/to/datastore/v1.5
    let minor_version_link = to_dir.join(format!("v{}.{}", version.major, version.minor));
    // Build the path to the patch version link.  If this already exists, it's because we've
    // previously tried to migrate to this version.  We point it at the full `to_datastore`
    // path.
    // Example: /path/to/datastore/v1.5.2
    let patch_version_link = to_dir.join(format!(
        "v{}.{}.{}",
        version.major, version.minor, version.patch
    ));

    // Get the final component of the paths we're linking to, so we can use relative links instead
    // of absolute, for understandability.
    let to_target = to_datastore
        .as_ref()
        .file_name()
        .context(error::DataStoreLinkToRoot {
            path: to_datastore.as_ref(),
        })?;
    let patch_target = patch_version_link
        .file_name()
        .context(error::DataStoreLinkToRoot {
            path: to_datastore.as_ref(),
        })?;
    let minor_target = minor_version_link
        .file_name()
        .context(error::DataStoreLinkToRoot {
            path: to_datastore.as_ref(),
        })?;
    let major_target = major_version_link
        .file_name()
        .context(error::DataStoreLinkToRoot {
            path: to_datastore.as_ref(),
        })?;

    // =^..^=   =^..^=   =^..^=   =^..^=

    info!(
        "Flipping {} to point to {}",
        patch_version_link.display(),
        to_target.to_string_lossy(),
    );

    // Create a symlink from the patch version to the new data store.  We create it at a temporary
    // path so we can atomically swap it into the real path with a rename call.
    // This will point at, for example, /path/to/datastore/v1.5.2_0123456789abcdef
    symlink(&to_target, &temp_link).context(error::LinkCreate { path: &temp_link })?;
    // Atomically swap the link into place, so that the patch version link points to the new data
    // store copy.
    fs::rename(&temp_link, &patch_version_link).context(error::LinkSwap {
        link: &patch_version_link,
    })?;

    // =^..^=   =^..^=   =^..^=   =^..^=

    info!(
        "Flipping {} to point to {}",
        minor_version_link.display(),
        patch_target.to_string_lossy(),
    );

    // Create a symlink from the minor version to the new patch version.
    // This will point at, for example, /path/to/datastore/v1.5.2
    symlink(&patch_target, &temp_link).context(error::LinkCreate { path: &temp_link })?;
    // Atomically swap the link into place, so that the minor version link points to the new patch
    // version.
    fs::rename(&temp_link, &minor_version_link).context(error::LinkSwap {
        link: &minor_version_link,
    })?;

    // =^..^=   =^..^=   =^..^=   =^..^=

    info!(
        "Flipping {} to point to {}",
        major_version_link.display(),
        minor_target.to_string_lossy(),
    );

    // Create a symlink from the major version to the new minor version.
    // This will point at, for example, /path/to/datastore/v1.5
    symlink(&minor_target, &temp_link).context(error::LinkCreate { path: &temp_link })?;
    // Atomically swap the link into place, so that the major version link points to the new minor
    // version.
    fs::rename(&temp_link, &major_version_link).context(error::LinkSwap {
        link: &major_version_link,
    })?;

    // =^..^=   =^..^=   =^..^=   =^..^=

    info!(
        "Flipping {} to point to {}",
        current_version_link.display(),
        major_target.to_string_lossy(),
    );

    // Create a symlink from 'current' to the new major version.
    // This will point at, for example, /path/to/datastore/v1
    symlink(&major_target, &temp_link).context(error::LinkCreate { path: &temp_link })?;
    // Atomically swap the link into place, so that 'current' points to the new major version.
    fs::rename(&temp_link, &current_version_link).context(error::LinkSwap {
        link: &current_version_link,
    })?;

    // =^..^=   =^..^=   =^..^=   =^..^=

    // fsync the directory so the links point to the new version even if we crash right after
    // this.  If fsync fails, warn but continue, because we likely can't swap the links back
    // without hitting the same failure.
    fsync(raw_dir.as_raw_fd()).unwrap_or_else(|e| {
        warn!(
            "fsync of data store directory '{}' failed, update may disappear if we crash now: {}",
            to_dir.display(),
            e
        )
    });

    Ok(())
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[allow(unused_variables)]
    fn select_migrations_works() {
        // Migration paths for use in testing
        let m00_1 = Path::new("migrate_v0.0.0_001");
        let m01_1 = Path::new("migrate_v0.0.1_001");
        let m01_2 = Path::new("migrate_v0.0.1_002");
        let m02_1 = Path::new("migrate_v0.0.2_001");
        let m03_1 = Path::new("migrate_v0.0.3_001");
        let m04_1 = Path::new("migrate_v0.0.4_001");
        let m04_2 = Path::new("migrate_v0.0.4_002");
        let all_migrations = vec![&m00_1, &m01_1, &m01_2, &m02_1, &m03_1, &m04_1, &m04_2];

        // Versions for use in testing
        let v00 = Version::new(0, 0, 0);
        let v01 = Version::new(0, 0, 1);
        let v02 = Version::new(0, 0, 2);
        let v03 = Version::new(0, 0, 3);
        let v04 = Version::new(0, 0, 4);
        let v05 = Version::new(0, 0, 5);

        // Test going forward one minor version
        assert_eq!(
            select_migrations(&v01, &v02, &all_migrations).unwrap(),
            vec![m02_1]
        );

        // Test going backward one minor version
        assert_eq!(
            select_migrations(&v02, &v01, &all_migrations).unwrap(),
            vec![m02_1]
        );

        // Test going forward a few minor versions
        assert_eq!(
            select_migrations(&v01, &v04, &all_migrations).unwrap(),
            vec![m02_1, m03_1, m04_1, m04_2]
        );

        // Test going backward a few minor versions
        assert_eq!(
            select_migrations(&v04, &v01, &all_migrations).unwrap(),
            vec![m04_2, m04_1, m03_1, m02_1]
        );

        // Test no matching migrations
        assert!(select_migrations(&v04, &v05, &all_migrations)
            .unwrap()
            .is_empty());
    }
}
