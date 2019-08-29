# Data store migration

## Overview

We're not perfect, so we need the ability to fix problems in our design, but it's critical to maintain user configuration through those changes.
To achieve both, we need the ability to migrate configuration between versions.

The mechanism proposed is reminiscent of database migrations - individual bits of code that manipulate specific data, and that are labeled to indicate what version they apply to.

## What exactly is versioned

This document covers versioning of the data store specifically.
Two things will be versioned: the data store format itself, and the content format within the data store.

### data store format

This refers to the major structure of the data store, i.e. the fact that we're using the filesystem to store data, how the data is laid out on the filesystem, etc.

This will start out at “v1”.
If we were to make a major change, e.g. moving to SQLite, we would move to v2.

The data store format is not expected to change frequently.
Most of this document covers content format changes; see [Handling data store format changes](#handling-data-store-format-changes) below for discussion of handling data store format changes.

### content format

This refers to the available keys and the format of data within those keys.

This will start out at 0.
When combined with the data store format, that means we will start at “v1.0”.
A content format update would put us at “v1.1”.
A data store format update would then put us at “v2.0”.

The content format is expected to change relatively frequently, because we will likely get requests for adding customization, and will be updating and adding applications occasionally.

Note: adding new keys will usually not require a migration; default settings will be populated for new keys automatically.

### Rejected options

#### Versioning prefixes:

"prefix" means the initial, common substring of a set of keys.
For example, `settings.a.b.c` and `settings.a.x.y` would both be matched by the prefix `settings.a`.

We could attach a version number to prefixes and migrate them separately.
This would provide the benefit of scoping migrations to a specific subset of data, e.g. only “settings.docker” to handle a Docker upgrade.
However, this would be confusing because parent and child nodes could have different versions.
It would be difficult to understand what migrations should run, which have been run, etc.

### For reference: Other versioned artifacts

Other documents should cover:

* Versioning of the API, and how that versioning relates to data store versioning
* Versioning of specific components like Docker or the Linux kernel; this is more complex because we may want to support multiple versions

## When to migrate data

We will run migrations during the boot process when we detect that the on-disk data store doesn't match the format of the booting image.

This can slow boot, but the impact can be partially mitigated by having recent migrations stored in the image, and having the system download any missing, appropriate migrations before rebooting. (The image can be mounted read-only after being written to disk so we can check what’s missing.)

Another downside is that migration failure would require another reboot to flip back to a matching version.

In return, we get a consistent process where we always check and migrate the data store before it's used, and we have a simpler pre-reboot process.

We can also handle some cases of unexpected version mismatches at boot, for example if something goes wrong during an update.

### Offline migration

Since we’re running migrations at boot, we may not yet have a network connection.
This means we can’t check for fixes to migrations or download missing migrations.

When downloading an OS update, we necessarily download the TUF metadata describing that update.
This metadata also describes the migrations necessary between various updates.
During this update preparation process, we will also download all necessary migrations, and refuse to start an update otherwise.

This means we can be confident in running the migrations we have, even if there may be newer versions online.
If there are any fixed migrations, there will necessarily be a new content format version to handle fixing any known issues with migrations to the prior version.
This means we can be confident that running cached migrations offline won’t leave a machine stranded; it will just need to update again.

### Migrations location

We expect most migrations to be stored in the incoming image in a known location.
Since we’re running migrations at boot, we can easily access and run these from the current partition.

Not all migrations will be in the image, though, because of the case described above in [Offline Migration](#offline-migration) where fixed migrations are listed in the metadata and had to be downloaded.
At the time of download, we’re still in the old image, and the new image must be considered read-only or verified boot would fail.

Therefore, we must also look for migrations in a known location on persistent storage where they were downloaded.

### Rejected options

* Before reboot - This would optimistically prepare the data store before rebooting into a new version.
    * Pro: The flip to the new format would still have to happen after the reboot, but would be fast and nearly trivial.
    * Con: We must prevent changes via the API when migration starts so as to prevent data loss.
    * Con: We would still need the ability to migrate at boot if we discover inconsistent versions.

## Integration with update system

As mentioned above in [Offline Migration](#offline-migration), update metadata will list the migrations needed to move between versions.
This metadata will be made available for the migrator to use later to verify the migrations.

As mentioned above in [When to migrate data](#when-to-migrate-data), expected migrations will be stored directly in the image so most updates can be fulfilled without further downloads.
However, there will be times when we discover issues with migrations after release and need to replace them.
These fixed migrations can be listed in the update metadata and downloaded along with the image.
Since this metadata is the source of truth for the migration list, any old migrations in the image will simply be ignored.

Note: The tool downloading migrations does not have to be the same as the one downloading images, nor does it have to be part of the migrator.
Its job is closest to the update downloader, though, and so should be described in further detail in the update system design.

## How to update and flip the data store

To start, the migration system would copy the data store, update it by running migrations, and (when ready) flip a link to the new format.
This duplicates the data, but at the start, we aren't expecting to store large quantities of data in the data store.
We could mitigate this downside by deduplicating data through clever inner linking or other techniques.

### Data store symlink structure

Applications access the data store at `/var/lib/thar/datastore/current` so they don't have to understand the versioning scheme.

`current` is a link to the data store format (major) version, e.g. `v1`.
If we change data store format versions, it's this symlink we'd flip; see [Handling data store format changes](#handling-data-store-format-changes) below for details.

`v1` is a link to the content format (minor) version, e.g. `v1.5`.
Any time we change content format versions, it's this symlink we'd flip.

`v1.5` is a link to the real data store directory.
This has a random identifier appended so that we can run migrations on a copy without affecting live data.
The link is created by the migrator.

Here's a full example setup for version 1.5:
```
/var/lib/thar/datastore/current
   -> v1
   -> v1.5
   -> v1.5_0123456789abcdef
```

The old version's directory can be kept for quick rollbacks; automated cleanup will be necessary to prevent filling the disk.

Note that the version applies to both the live and pending trees, which live inside the directory described above - we have to migrate both live and pending data or we could lose customer information.

## How to run migrations

The migration system will follow these steps.

### Find outgoing and incoming versions

The outgoing version is represented by the link described above in [How to update and flip the data store](#how-to-update-and-flip-the-data-store).
The incoming version is listed in the file `/usr/share/thar/data-store-version` in the incoming image.

### Find migrations

Next we find the migration binaries that are applicable when moving from the outgoing version to the incoming version.
Migration names include the version where they first apply, so we take any migrations applicable after the outgoing version, up to and including the incoming version.

Migrations from recent versions will be in a known location (e.g. `/var/lib/thar/datastore/migrations`) in the incoming image, and all migrations will also be available on the update server for download.
Migration lists (metadata) will be available from both sources to ensure we're not missing any migrations.

### Run migrations

The migration system then runs the migrations in order.
This can use a relatively simple numerical ordering of the migration binary filenames, because the names include the applicable version and optional integer for ordering multiple migrations within a version.
(See [Structure](#structure).)

As mentioned above in [How to update and flip the data store](#how-to-update-and-flip-the-data-store), migrations are run on a copy of the current data store, and are run on the live and pending trees within the copy.

### Handling failure

Upon failure of a migration, we don't need to run any rollbacks because we were operating on a copy of the data store.

However, if migrating at boot, we need to flip the partition table back to the version that supports our current data store, and we should reboot back into the supported version.

## How to write migrations

### Structure

Migrations are Rust code adhering to a Migration interface.
This is defined in the `migration-helpers` library; see [Helpers](#helpers).

The interface will require handling forward and backward migrations through specific methods.
Each method will give data and metadata maps as input, and require data and metadata maps as output.
(These structures will be nested, rather than using dotted keys, to make it easier to handle sub-trees and reduce dependency on existing data store code.)

Migration code should not assume that any given keys exist, because migrations will be run on live data (where all keys will likely exist) and on pending data (where none, some, or all keys may exist).

The migration system could deserialize the function outputs into the incoming model types to confirm that the structure is valid; we should prototype this idea because it would add safety.

To write a migration, start a Rust project at `/migrations/<applicable version>/<int or name>/Cargo.toml`

The filename of the migration will be used by the migration system to order migrations.
The name will take the format `migrate_v<applicable version>_<int or name>`, similar to the directory name it's stored in.
Cargo does not allow naming binaries this way, so the migration build process will rename them appropriately when installing them into the image.
Migration authors can simply name their crate with the `<int or name>` referenced above.

### Helpers

We will have a standard structure for migration code that handles common things like argument parsing, so that we can have a common CLI interface for the migration system to run migrations.

An expected common use case is to discard some existing data and use the new defaults.
This is expected to be common for system-level data in the data store, e.g. services and configuration files, which aren't modifiable by the user.
We'll make it easy to generate default values from the incoming image, so the new defaults can be used as the migration output.
(This is not needed for entirely new keys, which will be taken from defaults automatically.)

### Rejected options

Regarding ordering:

* We could use metadata in the migration project, and in the resulting binary, to represent the applicable version and ordering.  It's unclear how to do this, and filenames are obvious; we can figure out a metadata-based system if our needs become more complex.


Regarding migration function parameters:

* We could use dotted keys instead of nested maps, but nested maps make it easier to handle sub-trees and reduce dependency on existing data store code.
* We could use real structures from the data model, but because we have to model incoming and outgoing parameters, we'd have to have both formats.  This would require versioned structs, or something similar, which is complex.

## Rollback

As mentioned in [Structure](#structure), migrations will be required to implement forward and backward migration functions.
Therefore, rollbacks can largely be treated the same way, just running the backward function instead.
(Migration binaries using `migration-helpers` have a flag for direction.)

If we're confident no user configuration has changed, or if the user wants to discard changes, we can do a quick rollback by flipping the symlink(s) back, as mentioned in [How to update and flip the data store](#how-to-update-and-flip-the-data-store).

## Handling data store format changes

Not all of the above design applies to handling data store format changes because they're necessarily bigger changes that we can't predict.
If we move everything to version 2, there could be a dramatically different storage system that requires entirely different migration types, for example.

### How to migrate data store format

We still have clear version numbers changing in a clear way, so we can use the basic plan from [How to run migrations](#how-to-run-migrations) above.

We would be migrating at the same time as in [When to migrate data](#when-to-migrate-data) above.

We would likely still flip symlinks as in [How to update and flip the data store](#how-to-update-and-flip-the-data-store) above, but we have to leave open the possibility that the data would move entirely and the symlinks are no longer appropriate.
Therefore, managing this would be left to the individual migration code.

The primary changes are to [How to write migrations](#how-to-write-migrations). We would still want the capability to migrate forward and backward, but we can't guarantee that the map-based parameters are relevant.
The migrations would have a looser interface (“void”) and would manipulate the existing and new data stores however they deem appropriate.
(We'd probably still want the same argument handling so the migration system can run them consistently.)

### Rejected options

We could use the OS version as a surrogate for data store format version, but because the data store format version will change rarely, there will be many times when we have a new OS version but no data store format change.
It would be harder to determine when there was actually a change, and harder to find appropriate migrations.
Because we expect to change data store format rarely, it's better to stay simple until we have more specific needs.

## Example use cases

### New application

Say we add a new open-source application, like rngd.
We can add data regarding settings, services, and configuration-files of the application to our data model.
In this case, we wouldn't need to write migrations because data store keys that are entirely new will be automatically populated from defaults by storewolf.

### Docker version upgrade

If we upgrade Docker, its available and required settings may change.
This means we'd have to update the data model to include any new or changed settings, and we'd write migrations to transform data from the old settings to the new.

### Data store implementation change

If we find that the filesystem data store implementation is insufficient, we may, for example, want to move to a database like SQLite, or to an improved filesystem data store implementation.
In this case, we'd use data store format migration(s) to insert the data into the new system and (if still applicable) flip symlinks so we know to use it.

## Open questions

### Saving old data

If migrating data requires throwing away part of the old data, we should have a solution for saving that old data so it can be used during the backward migration / rollback.

To give an example, say we start out with a "hostname" setting.
Later, we decide that we always want to take hostname from DHCP responses, so we remove the setting.
If we roll back, we'd want to re-insert the user-specified hostname we had before.
We can't do this unless we saved the value somewhere.

The system does not currently have a provision for this, but we can imagine a few options:
* The data store could have versioned keys, storing all previous values
* There could be a snapshot of the data store from before migrations

### Multiple versions of an application

We expect that some customers will want to run different versions of Docker or of the Linux kernel, for example.
If we want to support multiple versions of an application in our source tree, what does that mean for data migration?  If a user wants to switch from Linux 4.14 to 4.18, how do we migrate settings?

The main issue is that this presents another dimension for migrations.
Consider the case where the user is updating an instance and also wants change their kernel from 4.14 to 4.18.
We have to know to run version migrations, and also to run kernel-application-related migrations; how do we choose an order?  To prevent broken migrations, you'd need a total ordering across both dimensions.

We also don't currently have a concrete plan for configurable builds and feature selection, so it's unclear how to tie this in.
