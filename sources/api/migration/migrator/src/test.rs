//! Provides an end-to-end test of `migrator` via the `run` function. This module is conditionally
//! compiled for cfg(test) only.
use crate::args::Args;
use crate::run;
use chrono::{DateTime, Utc};
use semver::Version;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Provides the path to a folder where test data files reside.
fn test_data() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop();
    p.join("migrator").join("tests").join("data")
}

/// Returns the filepath to a `root.json` file stored in tree for testing. This file declares
/// an expiration date of `1970-01-01` to ensure success with an expired TUF repository.
fn root() -> PathBuf {
    test_data()
        .join("expired-root.json")
        .canonicalize()
        .unwrap()
}

/// Returns the filepath to a private key, stored in tree and used only for testing.
fn pem() -> PathBuf {
    test_data().join("snakeoil.pem").canonicalize().unwrap()
}

/// The name of a test migration. The prefix `b-` ensures we are not alphabetically sorting.
const FIRST_MIGRATION: &str = "b-first-migration";

/// The name of a test migration. The prefix `a-` ensures we are not alphabetically sorting.
const SECOND_MIGRATION: &str = "a-second-migration";

/// Creates a script that will serve as a migration during testing. The script writes its migrations
/// name to a file named `result.txt` in the parent directory of the datastore. `pentacle` does not
/// retain the name of the executing binary or script, so we take the `migration_name` as input,
/// and 'hardcode' it into the script.
fn create_test_migration<S: AsRef<str>>(migration_name: S) -> String {
    format!(
        r#"#!/usr/bin/env bash
set -eo pipefail
migration_name="{}"
datastore_parent_dir="$(dirname "${{3}}")"
outfile="${{datastore_parent_dir}}/result.txt"
echo "${{migration_name}}:" "${{@}}" >> "${{outfile}}"
"#,
        migration_name.as_ref()
    )
}

/// Holds the lifetime of a `TempDir` inside which a datastore directory and links are held for
/// testing.
struct TestDatastore {
    tmp: TempDir,
    datastore: PathBuf,
}

impl TestDatastore {
    /// Creates a `TempDir`, sets up the datastore links needed to represent the `from_version`
    /// and returns a `TestDatastore` populated with this information.
    fn new(from_version: Version) -> Self {
        let tmp = TempDir::new().unwrap();
        let datastore = storewolf::create_new_datastore(tmp.path(), Some(from_version)).unwrap();
        TestDatastore { tmp, datastore }
    }
}

/// Represents a TUF repository, which is held in a tempdir.
struct TestRepo {
    /// This field preserves the lifetime of the TempDir even though we never read it. When
    /// `TestRepo` goes out of scope, `TempDir` will remove the temporary directory.
    _tuf_dir: TempDir,
    metadata_path: PathBuf,
    targets_path: PathBuf,
}

/// LZ4 compresses `source` bytes to a new file at `destination`.
fn compress(source: &[u8], destination: &Path) {
    let output_file = File::create(destination).unwrap();
    let mut encoder = lz4::EncoderBuilder::new()
        .level(4)
        .build(output_file)
        .unwrap();
    encoder.write_all(source).unwrap();
    let (_output, result) = encoder.finish();
    result.unwrap()
}

/// Creates a test repository with a couple of versions defined in the manifest and a couple of
/// migrations. See the test description for for more info.
fn create_test_repo() -> TestRepo {
    // This is where the signed TUF repo will exist when we are done. It is the
    // root directory of the `TestRepo` we will return when we are done.
    let test_repo_dir = TempDir::new().unwrap();
    let metadata_path = test_repo_dir.path().join("metadata");
    let targets_path = test_repo_dir.path().join("targets");

    // This is where we will stage the TUF repository targets prior to signing them. We are using
    // symlinks from `tuf_indir` to `tuf_outdir/targets` so we keep both in the same `TempDir`.
    let tuf_indir = test_repo_dir.path();

    // Create a Manifest and save it to the tuftool_indir for signing.
    let mut manifest = update_metadata::Manifest::default();
    // insert the following migrations to the manifest. note that the first migration would sort
    // later than the second migration alphabetically. this is to help ensure that migrations
    // are running in their listed order (rather than sorted order as in previous
    // implementations).
    manifest.migrations.insert(
        (Version::new(0, 99, 0), Version::new(0, 99, 1)),
        vec![FIRST_MIGRATION.into(), SECOND_MIGRATION.into()],
    );
    update_metadata::write_file(tuf_indir.join("manifest.json").as_path(), &manifest).unwrap();

    // Create an script that we can use as the 'migration' that migrator will run. This script will
    // write its name and arguments to a file named result.txt in the directory that is the parent
    // of --source-datastore. result.txt can then be used to see what migrations ran, and in what
    // order. Note that tests are sensitive to the order and number of arguments passed. If
    // --source-datastore is given at a different position then the tests will fail and the script
    // will need to be updated.
    let migration_a = create_test_migration(FIRST_MIGRATION);
    let migration_b = create_test_migration(SECOND_MIGRATION);

    // Save lz4 compressed copies of the migration script into the tuftool_indir.
    compress(migration_a.as_bytes(), &tuf_indir.join(FIRST_MIGRATION));
    compress(migration_b.as_bytes(), &tuf_indir.join(SECOND_MIGRATION));

    // Create and sign the TUF repository.
    let mut editor = tough::editor::RepositoryEditor::new(root()).unwrap();
    let long_ago: DateTime<Utc> = DateTime::parse_from_rfc3339("1970-01-01T00:00:00Z")
        .unwrap()
        .into();
    let one = std::num::NonZeroU64::new(1).unwrap();
    editor
        .targets_version(one)
        .unwrap()
        .targets_expires(long_ago)
        .unwrap()
        .snapshot_version(one)
        .snapshot_expires(long_ago)
        .timestamp_version(one)
        .timestamp_expires(long_ago);

    fs::read_dir(tuf_indir)
        .unwrap()
        .filter(|dir_entry_result| {
            if let Ok(dir_entry) = dir_entry_result {
                return dir_entry.path().is_file();
            }
            false
        })
        .for_each(|dir_entry_result| {
            let dir_entry = dir_entry_result.unwrap();
            editor
                .add_target(
                    dir_entry.file_name().to_str().unwrap(),
                    tough::schema::Target::from_path(dir_entry.path()).unwrap(),
                )
                .unwrap();
        });
    let signed_repo = editor
        .sign(&[Box::new(tough::key_source::LocalKeySource { path: pem() })])
        .unwrap();
    signed_repo
        .link_targets(
            tuf_indir,
            &targets_path,
            tough::editor::signed::PathExists::Fail,
        )
        .unwrap();
    signed_repo.write(&metadata_path).unwrap();

    TestRepo {
        _tuf_dir: test_repo_dir,
        metadata_path,
        targets_path,
    }
}

/// Tests the migrator program end-to-end using the `run` function. Creates a TUF repo in a
/// tempdir which includes a  `manifest.json` with a couple of migrations:
/// ```
///     "(0.99.0, 0.99.1)": [
///       "b-first-migration",
///       "a-second-migration"
///     ]
/// ```
///
/// The two 'migrations' are instances of the same bash script (see `create_test_repo`) which
/// writes its name (i.e. the migration name) and its arguments to a file at `./result.txt`
/// (i.e. since migrations run in the context of the datastore directory, `result.txt` is
/// written one directory above the datastore.) We can then inspect the contents of `result.txt`
/// to see that the expected migrations ran in the correct order.
#[test]
fn migrate_forward() {
    let from_version = Version::parse("0.99.0").unwrap();
    let to_version = Version::parse("0.99.1").unwrap();
    let test_datastore = TestDatastore::new(from_version);
    let test_repo = create_test_repo();
    let args = Args {
        datastore_path: test_datastore.datastore.clone(),
        log_level: log::LevelFilter::Info,
        migration_directory: test_repo.targets_path.clone(),
        migrate_to_version: to_version,
        root_path: root(),
        metadata_directory: test_repo.metadata_path.clone(),
    };
    run(&args).unwrap();
    // the migrations should write to a file named result.txt.
    let output_file = test_datastore.tmp.path().join("result.txt");
    let contents = std::fs::read_to_string(&output_file).unwrap();
    let lines: Vec<&str> = contents.split('\n').collect();
    assert_eq!(lines.len(), 3);
    let first_line = *lines.get(0).unwrap();
    let want = format!("{}: --forward", FIRST_MIGRATION);
    let got: String = first_line.chars().take(want.len()).collect();
    assert_eq!(got, want);
    let second_line = *lines.get(1).unwrap();
    let want = format!("{}: --forward", SECOND_MIGRATION);
    let got: String = second_line.chars().take(want.len()).collect();
    assert_eq!(got, want);
}

/// This test ensures that migrations run when migrating from a newer to an older version.
/// See `migrate_forward` for a description of how these tests work.
#[test]
fn migrate_backward() {
    let from_version = Version::parse("0.99.1").unwrap();
    let to_version = Version::parse("0.99.0").unwrap();
    let test_datastore = TestDatastore::new(from_version);
    let test_repo = create_test_repo();
    let args = Args {
        datastore_path: test_datastore.datastore.clone(),
        log_level: log::LevelFilter::Info,
        migration_directory: test_repo.targets_path.clone(),
        migrate_to_version: to_version,
        root_path: root(),
        metadata_directory: test_repo.metadata_path.clone(),
    };
    run(&args).unwrap();
    let output_file = test_datastore.tmp.path().join("result.txt");
    let contents = std::fs::read_to_string(&output_file).unwrap();
    let lines: Vec<&str> = contents.split('\n').collect();
    assert_eq!(lines.len(), 3);
    let first_line = *lines.get(0).unwrap();
    let want = format!("{}: --backward", SECOND_MIGRATION);
    let got: String = first_line.chars().take(want.len()).collect();
    assert_eq!(got, want);
    let second_line = *lines.get(1).unwrap();
    let want = format!("{}: --backward", FIRST_MIGRATION);
    let got: String = second_line.chars().take(want.len()).collect();
    assert_eq!(got, want);
}
