//! This module aims to make it as easy as possible to migrate a data store between minor
//! versions.  Migration authors just implement one trait, and can then use helper methods to take
//! care of everything else in their main function.
//!
//! Note that you must still name your migration binary according to spec for it to be handled
//! properly by the migration runner.

// Note that migrations must be run serially; technically, this is because the data store isn't
// locked, and also because migration authors are given an interface for ordering via migration
// name, and running in parallel would violate that.

#![deny(rust_2018_idioms)]

mod args;
mod datastore;
pub mod error;

use std::collections::HashMap;
use std::env;
use std::fmt;

use apiserver::datastore::{Committed, Value};
pub use apiserver::datastore::{DataStore, FilesystemDataStore, Key, KeyType};

use args::parse_args;
use datastore::{get_input_data, set_output_data};
pub use error::Result;

/// The data store implementation currently in use.  Used by the simpler `migrate` interface; can
/// be overridden by using the `run_migration` interface.
type DataStoreImplementation = FilesystemDataStore;

/// Migrations must implement this trait, and can then use the migrate method to let this module
/// do the rest of the work.
///
/// Migrations must implement forward and backward methods so changes can be rolled back as
/// necessary.
///
/// Migrations must not assume any key will exist because they're run on pending data as well as
/// live, and pending may be empty.  For the same reason, migrations must not add a key in all
/// cases if it's missing, because you could add a key to the pending data when the user didn't
/// have any pending data.  Instead, make sure you're adding a key to an existing structure.
pub trait Migration {
    /// Migrates data forward from the prior version to the version specified in the migration
    /// name.
    fn forward(&mut self, input: MigrationData) -> Result<MigrationData>;

    /// Migrates data backward from the version specified in the migration name to the prior
    /// version.
    fn backward(&mut self, input: MigrationData) -> Result<MigrationData>;
}

/// Mapping of metadata key to arbitrary value.  Each data key can have a Metadata describing its
/// metadata keys.
pub type Metadata = HashMap<Key, Value>;

/// MigrationData holds all data that can be migrated in a migration, and serves as the input and
/// output format of migrations.  A serde Value type is used to hold the arbitrary data of each
/// key because we can't represent types when they could change in the migration.
#[derive(Debug)]
pub struct MigrationData {
    /// Mapping of data keys to their arbitrary values.
    pub data: HashMap<Key, Value>,
    /// Mapping of data keys to their metadata.
    pub metadata: HashMap<Key, Metadata>,
}

/// Returns the default settings for a given path so you can easily replace a given section of the
/// datastore with new defaults.  For example, you could request "settings" to get all new default
/// settings, or "settings.serviceX.subsection" to scope it down.
pub fn defaults_for<S: AsRef<str>>(_path: S) -> Result<Value> {
    unimplemented!()
}

/// Ensures we can use the migrated data in the new data store.  Can use this result to stop the
/// migration process before saving any data.
fn validate_migrated_data(_migrated: &MigrationData) -> Result<()> {
    // No validations yet.
    // You can check the migrated data and throw error::Validation if anything seems wrong.
    Ok(())
}

/// If you need a little more control over a migration than with migrate, or you're using this
/// module as a library, you can call run_migration directly with the arguments that would
/// normally be parsed from the migration binary's command line.
pub fn run_migration<D: DataStore>(
    mut migration: impl Migration,
    datastore: &mut D,
    migration_type: MigrationType,
) -> Result<()> {
    for committed in &[Committed::Live, Committed::Pending] {
        let input = get_input_data(datastore, *committed)?;

        let migrated = match migration_type {
            MigrationType::Forward => migration.forward(input),
            MigrationType::Backward => migration.backward(input),
        }?;

        validate_migrated_data(&migrated)?;

        set_output_data(datastore, &migrated, *committed)?;
    }
    Ok(())
}

/// Represents the type of migration, so we know which Migration trait method to call.
#[derive(Debug, Copy, Clone)]
pub enum MigrationType {
    Forward,
    Backward,
}

impl fmt::Display for MigrationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MigrationType::Forward => write!(f, "forward"),
            MigrationType::Backward => write!(f, "backward"),
        }
    }
}

/// This is the primary entry point for migration authors.  When you've implemented the Migration
/// trait, you should just be able to pass it to this function from your main function and let it
/// take care of the rest.  The migration runner will pass in the appropriate datastore path and
/// migration type.
pub fn migrate(migration: impl Migration) -> Result<()> {
    let args = parse_args(env::args())?;
    let mut datastore = DataStoreImplementation::new(args.datastore_path);
    run_migration(migration, &mut datastore, args.migration_type)
}
