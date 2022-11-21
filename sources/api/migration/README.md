# Data store migration

## Overview

We're not perfect, so we need the ability to fix problems in our design, but it's critical to maintain user configuration through those changes.
To achieve both, the ability to migrate configuration between versions is required.

The mechanism is reminiscent of database migrations - individual bits of code that manipulate specific data, and that are labeled to indicate what version they apply to.

## When to migrate data

Migrations run during the boot process when it is detected that the on-disk data store doesn't match the version of the booting image.
The system will download any missing, appropriate migrations before rebooting.

One downside of this approach is that migration failure requires another reboot to flip back to a matching version.

In return, this provides a consistent process: always check and migrate the data store before it's used.
Additionally, the pre-reboot process is simpler.
The process can also handle some cases of unexpected version mismatches at boot, for example if something goes wrong during an update.

### Offline migration

Since migrations run at boot the network connection may not be available yet.
This means it is impossible to check for fixes to migrations or download missing migrations.

When downloading an OS update, the TUF metadata and the Bottlerocket update manifest are retrieved for that particular variant and architecture.
The manifest describes the migrations necessary between various updates and the TUF metadata ensures its integrity.
During this update preparation process, all necessary migrations are downloaded and an update will refuse to start otherwise.

### Rejected designs

* Before reboot - This would optimistically prepare the data store before rebooting into a new version.
    * Pro: The flip to the new format would still have to happen after the reboot, but would be fast and nearly trivial.
    * Con: We must prevent changes via the API when migration starts so as to prevent data loss.
    * Con: We would still need the ability to migrate at boot if we discover inconsistent versions.

## Integration with update system

As mentioned above in [Offline Migration](#offline-migration), the manifest lists the migrations needed to move between versions.
This manifest is made available for the migrator later along with TUF metadata to verify the migrations.

There may be times when we discover issues with migrations after release and need to replace them.
These fixed migrations are listed in the manifest and downloaded along with the image.
Since this metadata is the source of truth for the migration list, any old migrations will simply be ignored.

## How to update and flip the data store

To start, the migration system copies the data store, updates it by running migrations, and (when ready) flips symlinks to the new version.
This duplicates the data, but currently the data store only contains a small amount of data.
New OS versions that have no data store changes are simply a new symlink rather than a copy.

### Data store symlink structure

Applications access the data store at `/var/lib/bottlerocket/datastore/current` so they don't have to understand the versioning scheme.

`current` is a link to the major version, which is a link to the minor version, etc.
Here's a full example setup for version 1.5:
```shell
/var/lib/bottlerocket/datastore/current
   -> v1
   -> v1.5
   -> v1.5.2
   -> v1.5.2_0123456789abcdef
```
The migration system appends a random identifier to the final level so it can track migration attempts and help prevent timing issues.

Old versions can be kept for quick rollbacks, but automated cleanup will be necessary to prevent filling the disk.

Note that the version applies to both the `live` and `pending` trees, which both live inside the directory described above.
Both live and pending data is migrated to prevent user configuration information from being lost.

## How to run migrations

The migration system follows these steps:

### Find outgoing and incoming versions

The outgoing version is represented by the link described above in [How to update and flip the data store](#how-to-update-and-flip-the-data-store).
The incoming version is listed in the file `/etc/os-release` in the incoming image.

### Find migrations

Next, the migration system finds applicable migration binaries for moving from the outgoing version to the incoming version.
Migration names include the version where they first apply, so all applicable migrations after the outgoing version and up to and including the incoming version are used by the migration system.

Migrations are in a known location (e.g. `/var/lib/bottlerocket-migrations`) and all migrations are also available on the update server for download.
Migration lists (metadata) are available from both sources to ensure no migrations are missing.

### Run migrations

The migration system then runs the migrations in order.
This uses a relatively simple numerical ordering of the migration binary filenames, because the names include the applicable version and a name for ordering multiple migrations within a version.
(See [Structure](#structure).)

As mentioned above in [How to update and flip the data store](#how-to-update-and-flip-the-data-store), migrations are run on an in-memory copy of the current data store, and are run on the live and pending trees within the copy.

### Handling failure

Upon failure of a migration, rollbacks aren't needed because the migration system is operating on a copy of the data store.

If the migrator fails, boot services will be marked as failed, which means the current partition set won't be marked as successful.
A reboot will automatically switch back to the other partition set containing the version that supports the current data store.

## How to write migrations

### Structure

Migrations are Rust code adhering to a Migration interface.
This is defined in the `migration-helpers` library; see [Helpers](#helpers).

The interface requires handling forward and backward migrations through specific methods.
Each method will give data and metadata maps as input, and require data and metadata maps as output.
(These structures are nested, rather than using dotted keys, to make it easier to handle sub-trees and reduce dependency on existing data store code.)

Migration code should not assume that any given keys exist, because migrations will be run on live data (where all keys will likely exist) and on pending data (where none, some, or all keys may exist).
Plus, different variants of Bottlerocket may not have the same keys.

To write a migration, start a Rust project at `/migrations/<applicable version>/migrate-<name>/Cargo.toml`

Migrations run in the order that they are found in the manifest.
The name takes the format `migrate_v<applicable version>_<name>`.
Cargo does not allow naming binaries this way, so the migration build process renames them appropriately when installing them into the image.

### Helpers

There is a standard structure for migration code that handles common things like argument parsing, which allows for a common CLI interface for the migration system.

There is also a Rust module that handles common migration types, such as adding, removing, and replacing settings.

### Rejected designs

Regarding ordering:

* We could use metadata in the migration project, and in the resulting binary, to represent the applicable version and ordering.  It's unclear how to do this, and filenames are obvious; we can figure out a metadata-based system if our needs become more complex.

Regarding migration function parameters:

* We could use dotted keys instead of nested maps, but nested maps make it easier to handle sub-trees and reduce dependency on existing data store code.
* We could use real structures from the data model, but because we have to model incoming and outgoing parameters, we'd have to have both formats.  This would require versioned structs, or something similar, which is complex.

## Rollback

As mentioned in [Structure](#structure), migrations will be required to implement forward and backward migration functions.
Therefore, rollbacks can largely be treated the same way, just running the backward function instead.
(Migration binaries using `migration-helpers` have a flag for direction.)

In situations where no user configuration has changed, or if the user wants to discard changes, a quick rollback can be accomplished by flipping the symlink(s) back, as mentioned in [How to update and flip the data store](#how-to-update-and-flip-the-data-store).

## Example use cases

### New application

Say the project adds a new open-source application, like rngd.
This requires adding data regarding settings, services, and configuration-files of the application to the data model.

In this case, there is the existing helper `AddSettingsMigration`.
It doesn't need to do anything on upgrade, because the new key will be populated by its default value.
On downgrade, it removes the setting, so that the old data store model doesn't see an unexpected key and reject the data.

### Application version upgrade

If an important application is upgraded, its available and required settings may change.
This means updating the data model to include any new or changed settings, and writing migrations to transform data from the old settings to the new.
This can likely be handled by existing helpers `AddSettingsMigration`, `RemoveSettingsMigration`, `ReplaceStringMigration`, and `ReplaceTemplateMigration`.

## Open questions and future directions

### Data store implementation change

If we find that the filesystem data store implementation is insufficient, we may, for example, want to move to a database like SQLite, or to an improved filesystem data store implementation.
If it can use the same symlink-flip system, we can handle it with a migration: make a migration that doesn't use the standard migration-helpers library, but uses custom code to affect the larger change.
If it needs an entirely new structure, we need to add capabilities to the migrator first.

### Saving old data

If migrating data requires throwing away part of the old data, we should have a solution for saving that old data so it can be used during the backward migration / rollback.

To give an example, say we start out with a "hostname" setting.
Later, we decide that we always want to take hostname from DHCP responses, so we remove the setting.
If we roll back, we'd want to re-insert the user-specified hostname we had before.
We can't do this unless we saved the value somewhere.

The system does not currently have a provision for this, but we can imagine a few options:
* The data store could have versioned keys, storing all previous values
* There could be a snapshot of the data store from before migrations
* There could be a "recycle bin" outside of the data store that stores any removed data in a separate versioned store, making it available for rollbacks

### Variants

Bottlerocket variants allow the creation of Bottlerocket images with varying system components.
Do we want to support moving between variants?
If so, and we want to maintain user settings through the transition, we need to understand how to define and order migrations between variants.

### Migrations

The migration system could deserialize the function outputs into the incoming model types to confirm that the structure is valid; we should prototype this idea because it would add safety.
