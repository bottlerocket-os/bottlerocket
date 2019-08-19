//! This is a tool to run migrations built with the migration-helpers library.
//!
//! Given a data store and a version to migrate it to, it will find and run appropriate migrations
//! for the version on a copy of the data store, then symlink-flip it into place.

#[macro_use]
extern crate log;

use nix::{dir::Dir, fcntl::OFlag, sys::stat::Mode, unistd::fsync};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use snafu::{ensure, OptionExt, ResultExt};
use std::env;
use std::fs;
use std::os::unix::fs::symlink;
use std::os::unix::io::AsRawFd;
use std::path::{Path, PathBuf};
use std::process::{self, Command};
use std::str::FromStr;

mod args;
mod error;
mod version;

use args::Args;
use error::Result;
use version::{Direction, Version, VersionComponent, MIGRATION_FILENAME_RE};

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

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

    // TODO: starting with simple stderr logging, replace when we have a better idea.
    stderrlog::new()
        .module(module_path!())
        .timestamp(stderrlog::Timestamp::Millisecond)
        .verbosity(args.verbosity)
        .color(args.color)
        .init()
        .context(error::Logger)?;

    // We don't handle data store format (major version) migrations because they could change
    // anything about our storage; they're handled by more free-form binaries run by a separate
    // startup service.
    let current_version = Version::from_datastore_path(&args.datastore_path)?;
    if current_version.major != args.migrate_to_version.major {
        return error::MajorVersionMismatch {
            given: args.migrate_to_version.major,
            found: current_version.major,
        }
        .fail();
    }

    let direction = Direction::from_versions(current_version, args.migrate_to_version)
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
        current_version,
        args.migrate_to_version,
    )?;

    let (copy_path, copy_id) = copy_datastore(&args.datastore_path, args.migrate_to_version)?;
    run_migrations(direction, &migrations, &copy_path)?;
    flip_to_new_minor_version(args.migrate_to_version, &copy_path, &copy_id)?;

    Ok(())
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

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
    from: Version,
    to: Version,
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
        let version = Version::from_str(version_match.as_str())
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
        let applicable = if to > from && version > from && version <= to {
            info!(
                "Found applicable forward migration '{}': {} < ({}) <= {}",
                file_name, from, version, to
            );
            true
        } else if to < from && version > to && version <= from {
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
fn find_migrations<P>(paths: &[P], from: Version, to: Version) -> Result<Vec<PathBuf>>
where
    P: AsRef<Path>,
{
    let mut candidates = Vec::new();
    for path in paths {
        candidates.extend(find_migrations_on_disk(path)?);
    }
    select_migrations(from, to, &candidates)
}

/// Copies the data store at the given path to a new directory in the same parent direction, with
/// the new copy being named appropriately for the given new version.
fn copy_datastore<P: AsRef<Path>>(from: P, new_version: Version) -> Result<(PathBuf, String)> {
    // First, we need a random ID to append; this helps us avoid timing issues, and lets us
    // leave failed migrations for inspection, while being able to try again in a new path.
    // Note: we use this rather than mktemp::new_path_in because that has a delete destructor.
    // Note: consider using this as a more general migration ID?
    let copy_id = thread_rng().sample_iter(&Alphanumeric).take(16).collect();

    let to = from
        .as_ref()
        .with_file_name(format!("{}_{}", new_version, copy_id));
    ensure!(
        !to.exists(),
        error::NewVersionAlreadyExists {
            version: new_version,
            path: to
        }
    );

    info!(
        "Copying datastore from {} to work location {}",
        from.as_ref().display(),
        to.display()
    );

    let mut copy_options = fs_extra::dir::CopyOptions::new();
    // Set copy_inside true; if we're moving from v0.0 to v0.1, and this isn't set, it tries to
    // copy "v0.0" itself inside "v0.1".
    copy_options.copy_inside = true;
    // Note: this copies file permissions but not directory permissions; OK?
    fs_extra::dir::copy(&from, &to, &copy_options).context(error::DataStoreCopy)?;

    Ok((to, copy_id))
}

/// Runs the given migrations in their given order on the given data store.  The given direction
/// is passed to each migration so it knows which direction we're migrating.
fn run_migrations<P1, P2>(direction: Direction, migrations: &[P1], datastore_path: P2) -> Result<()>
where
    P1: AsRef<Path>,
    P2: AsRef<Path>,
{
    for migration in migrations {
        let mut command = Command::new(migration.as_ref());

        // Point each migration in the right direction, and at the given data store.
        command.arg(direction.to_string());
        command.args(&[
            "--datastore-path".to_string(),
            datastore_path.as_ref().display().to_string(),
        ]);

        info!("Running migration command: {:?}", command);

        let output = command
            .output()
            .context(error::StartMigration { command })?;

        debug!(
            "Migration stdout: {}",
            std::str::from_utf8(&output.stdout).unwrap_or("<invalid UTF-8>")
        );
        debug!(
            "Migration stderr: {}",
            std::str::from_utf8(&output.stderr).unwrap_or("<invalid UTF-8>")
        );

        ensure!(output.status.success(), error::MigrationFailure { output });
    }
    Ok(())
}

/// Atomically flips version symlinks to point to the given "to" datastore so that it becomes live.
///
/// This includes pointing the new minor version to the given `to_datastore` (which includes
/// `copy_id` in its name), then pointing the major version to the new minor version, and finally
/// fsyncing the directory to disk.
///
/// `copy_id` is the identifier for this migration attempt, as created by copy_datastore, which we
/// use internally in this function for consistency.
fn flip_to_new_minor_version<P, S>(version: Version, to_datastore: P, copy_id: S) -> Result<()>
where
    P: AsRef<Path>,
    S: AsRef<str>,
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
    // We use the same copy_id so that mistakes/errors here will be obviously related to a
    // given copy attempt.
    let temp_link = to_dir.join(copy_id.as_ref());
    // Build the path to the major version link; this is what we're atomically swapping from
    // pointing at the old minor version to pointing at the new minor version.
    // Example: /path/to/datastore/v1
    // FIXME: duplicating knowledge of formatting here
    let major_version_link = to_dir.join(format!("v{}", version.major));
    // Build the path to the minor version link.  If this already exists, it's because we've
    // previously tried to migrate to this version.  We point it at the full `to_datastore` path.
    // Example: /path/to/datastore/v1.5
    let minor_version_link = to_dir.join(format!("{}", version));

    // Get the final component of the paths we're linking to, so we can use relative links instead
    // of absolute, for understandability.
    let to_target = to_datastore
        .as_ref()
        .file_name()
        .context(error::DataStoreLinkToRoot {
            path: to_datastore.as_ref(),
        })?;
    let minor_target = minor_version_link
        .file_name()
        .context(error::DataStoreLinkToRoot {
            path: to_datastore.as_ref(),
        })?;

    info!(
        "Flipping {} to point to {}",
        minor_version_link.display(),
        to_target.to_string_lossy(),
    );

    // Create a symlink from the minor version to the new data store.  We create it at a temporary
    // path so we can atomically swap it into the real path with a rename call.
    // This will point at, for example, /path/to/datastore/v1.5_0123456789abcdef
    symlink(&to_target, &temp_link).context(error::LinkCreate { path: &temp_link })?;
    // Atomically swap the link into place, so that the minor version link points to the new data
    // store copy.
    fs::rename(&temp_link, &minor_version_link).context(error::LinkSwap {
        link: &minor_version_link,
    })?;

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
        let m00_1 = Path::new("migrate_v0.0_001");
        let m01_1 = Path::new("migrate_v0.1_001");
        let m01_2 = Path::new("migrate_v0.1_002");
        let m02_1 = Path::new("migrate_v0.2_001");
        let m03_1 = Path::new("migrate_v0.3_001");
        let m04_1 = Path::new("migrate_v0.4_001");
        let m04_2 = Path::new("migrate_v0.4_002");
        let all_migrations = vec![&m00_1, &m01_1, &m01_2, &m02_1, &m03_1, &m04_1, &m04_2];

        // Versions for use in testing
        let v00 = Version::new(0, 0);
        let v01 = Version::new(0, 1);
        let v02 = Version::new(0, 2);
        let v03 = Version::new(0, 3);
        let v04 = Version::new(0, 4);
        let v05 = Version::new(0, 5);

        // Test going forward one minor version
        assert_eq!(
            select_migrations(v01, v02, &all_migrations).unwrap(),
            vec![m02_1]
        );

        // Test going backward one minor version
        assert_eq!(
            select_migrations(v02, v01, &all_migrations).unwrap(),
            vec![m02_1]
        );

        // Test going forward a few minor versions
        assert_eq!(
            select_migrations(v01, v04, &all_migrations).unwrap(),
            vec![m02_1, m03_1, m04_1, m04_2]
        );

        // Test going backward a few minor versions
        assert_eq!(
            select_migrations(v04, v01, &all_migrations).unwrap(),
            vec![m04_2, m04_1, m03_1, m02_1]
        );

        // Test no matching migrations
        assert!(select_migrations(v04, v05, &all_migrations)
            .unwrap()
            .is_empty());
    }
}
