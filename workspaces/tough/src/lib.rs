//! Tough is a client library for [TUF repositories].
//!
//! [TUF repositories]: https://theupdateframework.github.io/

#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

mod datastore;
pub mod error;
mod fetch;
mod io;
mod serde;

use crate::datastore::Datastore;
use crate::error::Result;
use crate::fetch::{fetch_max_size, fetch_sha256};
use crate::serde::{Role, Root, Signed, Snapshot, Timestamp};
use chrono::{DateTime, Utc};
use reqwest::Client;
use reqwest::Url;
use snafu::{ensure, OptionExt, ResultExt};
use std::collections::BTreeMap;
use std::io::Read;
use std::path::Path;

/// A TUF repository.
///
/// You can create a `Repository` using the `load` method.
#[derive(Debug, Clone)]
pub struct Repository {
    client: Client,
    consistent_snapshot: bool,
    earliest_expiration: DateTime<Utc>,
    earliest_expiration_role: Role,
    target_base_url: Url,
    targets: BTreeMap<String, Target>,
}

impl Repository {
    /// Load and verify TUF repository metadata.
    ///
    /// `root` is a [`Read`]er for the trusted root metadata file, which you must ship with your
    /// software using an out-of-band process. It should be a copy of the most recent root.json
    /// from your repository. (It's okay if it becomes out of date later; the client establishes
    /// trust up to the most recent root.json file.)
    ///
    /// `datastore` is a [`Path`] to a directory on a persistent filesystem. This directory's
    /// contents store the most recently fetched root, timestamp, and snapshot metadata files. The
    /// directory must exist prior to calling this method, and **the directory and its contents
    /// must only be writable by your software.**
    ///
    /// `max_root_size` and `max_timestamp_size` are the maximum size for the root.json and
    /// timestamp.json files, respectively, downloaded from the repository. These must be
    /// sufficiently large such that future updates to your repository's key management strategy
    /// will still be supported, but sufficiently small such that you are protected against an
    /// endless data attack (defined by TUF as an attacker responding to clients with extremely
    /// large files that interfere with the client's system).
    ///
    /// `metadata_base_url` and `target_base_url` are the HTTP(S) base URLs for where the client
    /// can find metadata (such as root.json) and targets (as listed in targets.json). This method
    /// returns an error if the URLs do not end in slashes.
    pub fn load<R: Read, P: AsRef<Path>>(
        root: R,
        datastore: P,
        max_root_size: usize,
        max_timestamp_size: usize,
        metadata_base_url: &str,
        target_base_url: &str,
    ) -> Result<Self> {
        let client = Client::new();

        let metadata_base_url = parse_url(metadata_base_url)?;
        let target_base_url = parse_url(target_base_url)?;

        let datastore = Datastore::new(datastore)?;

        // 0. Load the trusted root metadata file + 1. Update the root metadata file
        let root = load_root(&client, root, &datastore, max_root_size, &metadata_base_url)?;

        // 2. Download the timestamp metadata file
        let timestamp = load_timestamp(
            &client,
            &root,
            &datastore,
            max_timestamp_size,
            &metadata_base_url,
        )?;

        // 3. Download the snapshot metadata file
        let snapshot = load_snapshot(&client, &root, &timestamp, &datastore, &metadata_base_url)?;

        // 4. Download the targets metadata file
        let targets = load_targets(&client, &root, &snapshot, &datastore, &metadata_base_url)?;

        let expires_iter = [
            (root.signed.expires, Role::Root),
            (timestamp.signed.expires, Role::Timestamp),
            (snapshot.signed.expires, Role::Snapshot),
            (targets.signed.expires, Role::Targets),
        ];
        let (earliest_expiration, earliest_expiration_role) =
            expires_iter.iter().min_by_key(|tup| tup.0).unwrap();

        Ok(Self {
            client,
            consistent_snapshot: root.signed.consistent_snapshot,
            earliest_expiration: earliest_expiration.to_owned(),
            earliest_expiration_role: *earliest_expiration_role,
            target_base_url,
            targets: targets
                .signed
                .targets
                .into_iter()
                .map(|(key, value)| (key, value.into()))
                .collect(),
        })
    }

    fn check_expired(&self) -> Result<()> {
        ensure!(
            Utc::now() < self.earliest_expiration,
            error::ExpiredMetadata {
                role: self.earliest_expiration_role
            }
        );
        Ok(())
    }

    /// Returns the list of targets present in the repository.
    pub fn targets(&self) -> &BTreeMap<String, Target> {
        &self.targets
    }

    /// Fetches a target from the repository.
    ///
    /// If the repository metadata is expired or there is an issue making the request, `Err` is
    /// returned.
    ///
    /// If the requested target is not listed in the repository metadata, `Ok(None)` is returned.
    ///
    /// Otherwise, a reader is returned, which provides access to the full target contents before
    /// its checksum is validated. If the maximum size is reached or there is a checksum mismatch,
    /// the reader returns a [`std::io::Error`]. **Consumers of this library must not use data from
    /// the reader if it returns an error.**
    pub fn read_target(&self, name: &str) -> Result<Option<impl Read>> {
        // 5. Verify the desired target against its targets metadata.
        //
        // 5.1. If there is no targets metadata about this target, abort the update cycle and
        //   report that there is no such target.
        //
        // 5.2. Otherwise, download the target (up to the number of bytes specified in the targets
        //   metadata), and verify that its hashes match the targets metadata. (We download up to
        //   this number of bytes, because in some cases, the exact number is unknown. This may
        //   happen, for example, if an external program is used to compute the root hash of a tree
        //   of targets files, and this program does not provide the total size of all of these
        //   files.) If consistent snapshots are not used (see Section 7), then the filename used
        //   to download the target file is of the fixed form FILENAME.EXT (e.g., foobar.tar.gz).
        //   Otherwise, the filename is of the form HASH.FILENAME.EXT (e.g.,
        //   c14aeb4ac9f4a8fc0d83d12482b9197452f6adf3eb710e3b1e2b79e8d14cb681.foobar.tar.gz), where
        //   HASH is one of the hashes of the targets file listed in the targets metadata file
        //   found earlier in step 4. In either case, the client MUST write the file to
        //   non-volatile storage as FILENAME.EXT.
        //
        // (This implementation currently assumes that the exact number of bytes a target contains
        // is known. This implementation does not download
        self.check_expired()?;
        Ok(if let Some(target) = self.targets.get(name) {
            let file = if self.consistent_snapshot {
                format!("{}.{}", hex::encode(&target.sha256), name)
            } else {
                name.to_owned()
            };

            Some(fetch_sha256(
                &self.client,
                self.target_base_url.join(&file).context(error::JoinUrl {
                    path: file,
                    url: self.target_base_url.to_owned(),
                })?,
                target.length,
                &target.sha256,
            )?)
        } else {
            None
        })
    }
}

/// A target from a repository.
#[derive(Debug, Clone)]
pub struct Target {
    /// Custom metadata for this target from the repository.
    pub custom: BTreeMap<String, serde_json::Value>,
    /// The SHA-256 checksum for this target.
    pub sha256: Vec<u8>,
    /// The maximum size in bytes for this target. This is not necessarily the exact size.
    pub length: usize,
}

impl From<crate::serde::Target> for Target {
    fn from(target: crate::serde::Target) -> Self {
        Self {
            custom: target.custom,
            sha256: target.hashes.sha256.into_vec(),
            length: target.length,
        }
    }
}

pub(crate) fn parse_url(url: &str) -> Result<Url> {
    ensure!(
        url.ends_with('/'),
        error::BaseUrlMissingTrailingSlash { url }
    );
    Url::parse(url).context(error::ParseUrl { url })
}

/// Steps 0 and 1 of the client application, which load the current root metadata file based on a
/// trusted root metadata file.
fn load_root<R: Read>(
    client: &Client,
    root: R,
    datastore: &Datastore,
    max_root_size: usize,
    metadata_base_url: &Url,
) -> Result<Signed<Root>> {
    // 0. Load the trusted root metadata file. We assume that a good, trusted copy of this file was
    //    shipped with the package manager or software updater using an out-of-band process. Note
    //    that the expiration of the trusted root metadata file does not matter, because we will
    //    attempt to update it in the next step.
    //
    // If a cached root.json is present in the datastore, prefer that over the `root` reader
    // provided to this function (unless it's corrupt).
    let mut root: Signed<Root> =
        if let Some(Ok(root)) = datastore.reader("root.json")?.map(serde_json::from_reader) {
            root
        } else {
            serde_json::from_reader(root).context(error::ParseTrustedMetadata)?
        };
    root.verify(&root).context(error::VerifyTrustedMetadata)?;

    // Used in step 1.9
    let original_timestamp_keys = root.signed.keys(Role::Timestamp);
    let original_snapshot_keys = root.signed.keys(Role::Snapshot);

    // 1. Update the root metadata file. Since it may now be signed using entirely different keys,
    //    the client must somehow be able to establish a trusted line of continuity to the latest
    //    set of keys. To do so, the client MUST download intermediate root metadata files, until
    //    the latest available one is reached.
    loop {
        // 1.1. Let N denote the version number of the trusted root metadata file.
        //
        // 1.2. Try downloading version N+1 of the root metadata file, up to some X number of bytes
        //   (because the size is unknown). The value for X is set by the authors of the
        //   application using TUF. For example, X may be tens of kilobytes. The filename used to
        //   download the root metadata file is of the fixed form VERSION_NUMBER.FILENAME.EXT
        //   (e.g., 42.root.json). If this file is not available, then go to step 1.8.
        let path = format!("{}.root.json", u64::from(root.signed.version) + 1);
        match fetch_max_size(
            client,
            metadata_base_url.join(&path).context(error::JoinUrl {
                path,
                url: metadata_base_url.to_owned(),
            })?,
            max_root_size,
        ) {
            Ok(reader) => {
                let new_root: Signed<Root> = serde_json::from_reader(reader)
                    .context(error::ParseMetadata { role: Role::Root })?;

                // 1.3. Check signatures. Version N+1 of the root metadata file MUST have been
                //   signed by: (1) a threshold of keys specified in the trusted root metadata file
                //   (version N), and (2) a threshold of keys specified in the new root metadata
                //   file being validated (version N+1). If version N+1 is not signed as required,
                //   discard it, abort the update cycle, and report the signature failure. On the
                //   next update cycle, begin at step 0 and version N of the root metadata file.
                new_root.verify(&root)?;
                new_root.verify(&new_root)?;

                // 1.4. Check for a rollback attack. The version number of the trusted root
                //   metadata file (version N) must be less than or equal to the version number of
                //   the new root metadata file (version N+1). Effectively, this means checking
                //   that the version number signed in the new root metadata file is indeed N+1. If
                //   the version of the new root metadata file is less than the trusted metadata
                //   file, discard it, abort the update cycle, and report the rollback attack. On
                //   the next update cycle, begin at step 0 and version N of the root metadata
                //   file.
                ensure!(
                    root.signed.version <= new_root.signed.version,
                    error::OlderMetadata {
                        role: Role::Root,
                        current_version: root.signed.version,
                        new_version: new_root.signed.version
                    }
                );

                // Off-spec: 1.4 specifies that the version number of the trusted root metadata
                // file must be less than or equal to the version number of the new root metadata
                // file. If they are equal, this will create an infinite loop, so we ignore the new
                // root metadata file but do not report an error.
                if root.signed.version == new_root.signed.version {
                    break;
                }

                // 1.5. Note that the expiration of the new (intermediate) root metadata file does
                //   not matter yet, because we will check for it in step 1.8.
                //
                // 1.6. Set the trusted root metadata file to the new root metadata file.
                root = new_root;

                // 1.7. Repeat steps 1.1 to 1.7.
                continue;
            }
            Err(_) => break, // If this file is not available, then go to step 1.8.
        }
    }

    // 1.8. Check for a freeze attack. The latest known time should be lower than the expiration
    //   timestamp in the trusted root metadata file (version N). If the trusted root metadata file
    //   has expired, abort the update cycle, report the potential freeze attack. On the next
    //   update cycle, begin at step 0 and version N of the root metadata file.
    root.check_expired()?;

    // 1.9. If the timestamp and / or snapshot keys have been rotated, then delete the trusted
    //   timestamp and snapshot metadata files. This is done in order to recover from fast-forward
    //   attacks after the repository has been compromised and recovered. A fast-forward attack
    //   happens when attackers arbitrarily increase the version numbers of: (1) the timestamp
    //   metadata, (2) the snapshot metadata, and / or (3) the targets, or a delegated targets,
    //   metadata file in the snapshot metadata.
    if original_timestamp_keys != root.signed.keys(Role::Timestamp)
        || original_snapshot_keys != root.signed.keys(Role::Snapshot)
    {
        datastore.remove("timestamp.json")?;
        datastore.remove("snapshot.json")?;
    }

    Ok(root)
}

/// Step 2 of the client application, which loads the timestamp metadata file.
fn load_timestamp(
    client: &Client,
    root: &Signed<Root>,
    datastore: &Datastore,
    max_timestamp_size: usize,
    metadata_base_url: &Url,
) -> Result<Signed<Timestamp>> {
    // 2. Download the timestamp metadata file, up to Y number of bytes (because the size is
    //    unknown.) The value for Y is set by the authors of the application using TUF. For
    //    example, Y may be tens of kilobytes. The filename used to download the timestamp metadata
    //    file is of the fixed form FILENAME.EXT (e.g., timestamp.json).
    let path = "timestamp.json";
    let reader = fetch_max_size(
        client,
        metadata_base_url.join(path).context(error::JoinUrl {
            path,
            url: metadata_base_url.to_owned(),
        })?,
        max_timestamp_size,
    )?;
    let timestamp: Signed<Timestamp> =
        serde_json::from_reader(reader).context(error::ParseMetadata {
            role: Role::Timestamp,
        })?;

    // 2.1. Check signatures. The new timestamp metadata file must have been signed by a threshold
    //   of keys specified in the trusted root metadata file. If the new timestamp metadata file is
    //   not properly signed, discard it, abort the update cycle, and report the signature failure.
    timestamp.verify(root)?;

    // 2.2. Check for a rollback attack. The version number of the trusted timestamp metadata file,
    //   if any, must be less than or equal to the version number of the new timestamp metadata
    //   file. If the new timestamp metadata file is older than the trusted timestamp metadata
    //   file, discard it, abort the update cycle, and report the potential rollback attack.
    //
    // (Unless it's corrupt.)
    if let Some(Ok(old_timestamp)) = datastore
        .reader("timestamp.json")?
        .map(serde_json::from_reader::<_, Signed<Timestamp>>)
    {
        ensure!(
            old_timestamp.signed.version <= timestamp.signed.version,
            error::OlderMetadata {
                role: Role::Timestamp,
                current_version: old_timestamp.signed.version,
                new_version: timestamp.signed.version
            }
        );
    }

    // 2.3. Check for a freeze attack. The latest known time should be lower than the expiration
    //   timestamp in the new timestamp metadata file. If so, the new timestamp metadata file
    //   becomes the trusted timestamp metadata file. If the new timestamp metadata file has
    //   expired, discard it, abort the update cycle, and report the potential freeze attack.
    timestamp.check_expired()?;

    // Now that everything seems okay, write the timestamp file to the datastore.
    datastore.create("timestamp.json", &timestamp)?;

    Ok(timestamp)
}

/// Step 3 of the client application, which loads the snapshot metadata file.
fn load_snapshot(
    client: &Client,
    root: &Signed<Root>,
    timestamp: &Signed<Timestamp>,
    datastore: &Datastore,
    metadata_base_url: &Url,
) -> Result<Signed<Snapshot>> {
    // 3. Download snapshot metadata file, up to the number of bytes specified in the timestamp
    //    metadata file. If consistent snapshots are not used (see Section 7), then the filename
    //    used to download the snapshot metadata file is of the fixed form FILENAME.EXT (e.g.,
    //    snapshot.json). Otherwise, the filename is of the form VERSION_NUMBER.FILENAME.EXT (e.g.,
    //    42.snapshot.json), where VERSION_NUMBER is the version number of the snapshot metadata
    //    file listed in the timestamp metadata file. In either case, the client MUST write the
    //    file to non-volatile storage as FILENAME.EXT.
    let snapshot_meta = timestamp
        .signed
        .meta
        .get("snapshot.json")
        .context(error::MetaMissing {
            file: "snapshot.json",
            role: Role::Timestamp,
        })?;
    let path = if root.signed.consistent_snapshot {
        format!("{}.snapshot.json", snapshot_meta.version)
    } else {
        "snapshot.json".to_owned()
    };
    let reader = fetch_sha256(
        client,
        metadata_base_url.join(&path).context(error::JoinUrl {
            path,
            url: metadata_base_url.to_owned(),
        })?,
        snapshot_meta.length,
        &snapshot_meta.hashes.sha256,
    )?;
    let snapshot: Signed<Snapshot> =
        serde_json::from_reader(reader).context(error::ParseMetadata {
            role: Role::Snapshot,
        })?;

    // 3.1. Check against timestamp metadata. The hashes and version number of the new snapshot
    //   metadata file MUST match the hashes and version number listed in timestamp metadata. If
    //   hashes and version do not match, discard the new snapshot metadata, abort the update
    //   cycle, and report the failure.
    //
    // (We already checked the hash in `fetch_sha256` above.)
    ensure!(
        snapshot.signed.version == snapshot_meta.version,
        error::VersionMismatch {
            role: Role::Snapshot,
            fetched: snapshot.signed.version,
            expected: snapshot_meta.version
        }
    );

    // 3.2. Check signatures. The new snapshot metadata file MUST have been signed by a threshold
    //   of keys specified in the trusted root metadata file. If the new snapshot metadata file is
    //   not signed as required, discard it, abort the update cycle, and report the signature
    //   failure.
    snapshot.verify(&root)?;

    // 3.3. Check for a rollback attack.
    //
    // 3.3.1. Note that the trusted snapshot metadata file may be checked for authenticity, but its
    //   expiration does not matter for the following purposes.
    if let Some(Ok(old_snapshot)) = datastore
        .reader("snapshot.json")?
        .map(serde_json::from_reader::<_, Signed<Snapshot>>)
    {
        // 3.3.2. The version number of the trusted snapshot metadata file, if any, MUST be less
        //   than or equal to the version number of the new snapshot metadata file. If the new
        //   snapshot metadata file is older than the trusted metadata file, discard it, abort the
        //   update cycle, and report the potential rollback attack.
        ensure!(
            old_snapshot.signed.version <= snapshot.signed.version,
            error::OlderMetadata {
                role: Role::Snapshot,
                current_version: old_snapshot.signed.version,
                new_version: snapshot.signed.version
            }
        );

        // 3.3.3. The version number of the targets metadata file, and all delegated targets
        //   metadata files (if any), in the trusted snapshot metadata file, if any, MUST be less
        //   than or equal to its version number in the new snapshot metadata file. Furthermore,
        //   any targets metadata filename that was listed in the trusted snapshot metadata file,
        //   if any, MUST continue to be listed in the new snapshot metadata file. If any of these
        //   conditions are not met, discard the new snaphot metadadata file, abort the update
        //   cycle, and report the failure.
        if let Some(old_targets_meta) = old_snapshot.signed.meta.get("targets.json") {
            let targets_meta =
                snapshot
                    .signed
                    .meta
                    .get("targets.json")
                    .context(error::MetaMissing {
                        file: "targets.json",
                        role: Role::Snapshot,
                    })?;
            ensure!(
                old_targets_meta.version <= targets_meta.version,
                error::OlderMetadata {
                    role: Role::Targets,
                    current_version: old_targets_meta.version,
                    new_version: targets_meta.version,
                }
            );
        }
    }

    // 3.4. Check for a freeze attack. The latest known time should be lower than the expiration
    //   timestamp in the new snapshot metadata file. If so, the new snapshot metadata file becomes
    //   the trusted snapshot metadata file. If the new snapshot metadata file is expired, discard
    //   it, abort the update cycle, and report the potential freeze attack.
    snapshot.check_expired()?;

    // Now that everything seems okay, write the timestamp file to the datastore.
    datastore.create("snapshot.json", &snapshot)?;

    Ok(snapshot)
}

/// Step 4 of the client application, which loads the targets metadata file.
fn load_targets(
    client: &Client,
    root: &Signed<Root>,
    snapshot: &Signed<Snapshot>,
    datastore: &Datastore,
    metadata_base_url: &Url,
) -> Result<Signed<crate::serde::Targets>> {
    // 4. Download the top-level targets metadata file, up to either the number of bytes specified
    //    in the snapshot metadata file, or some Z number of bytes. The value for Z is set by the
    //    authors of the application using TUF. For example, Z may be tens of kilobytes. If
    //    consistent snapshots are not used (see Section 7), then the filename used to download the
    //    targets metadata file is of the fixed form FILENAME.EXT (e.g., targets.json).  Otherwise,
    //    the filename is of the form VERSION_NUMBER.FILENAME.EXT (e.g., 42.targets.json), where
    //    VERSION_NUMBER is the version number of the targets metadata file listed in the snapshot
    //    metadata file. In either case, the client MUST write the file to non-volatile storage as
    //    FILENAME.EXT.
    let targets_meta = snapshot
        .signed
        .meta
        .get("targets.json")
        .context(error::MetaMissing {
            file: "targets.json",
            role: Role::Timestamp,
        })?;
    let path = if root.signed.consistent_snapshot {
        format!("{}.targets.json", targets_meta.version)
    } else {
        "targets.json".to_owned()
    };
    let reader = fetch_sha256(
        client,
        metadata_base_url.join(&path).context(error::JoinUrl {
            path,
            url: metadata_base_url.to_owned(),
        })?,
        targets_meta.length,
        &targets_meta.hashes.sha256,
    )?;
    let targets: Signed<crate::serde::Targets> =
        serde_json::from_reader(reader).context(error::ParseMetadata {
            role: Role::Targets,
        })?;

    // 4.1. Check against snapshot metadata. The hashes (if any), and version number of the new
    //   targets metadata file MUST match the trusted snapshot metadata. This is done, in part, to
    //   prevent a mix-and-match attack by man-in-the-middle attackers. If the new targets metadata
    //   file does not match, discard it, abort the update cycle, and report the failure.
    //
    // (We already checked the hash in `fetch_sha256` above.)
    ensure!(
        targets.signed.version == targets_meta.version,
        error::VersionMismatch {
            role: Role::Targets,
            fetched: targets.signed.version,
            expected: targets_meta.version
        }
    );

    // 4.2. Check for an arbitrary software attack. The new targets metadata file MUST have been
    //   signed by a threshold of keys specified in the trusted root metadata file. If the new
    //   targets metadata file is not signed as required, discard it, abort the update cycle, and
    //   report the failure.
    targets.verify(&root)?;

    // 4.3. Check for a rollback attack. The version number of the trusted targets metadata file,
    //   if any, MUST be less than or equal to the version number of the new targets metadata file.
    //   If the new targets metadata file is older than the trusted targets metadata file, discard
    //   it, abort the update cycle, and report the potential rollback attack.
    if let Some(Ok(old_targets)) = datastore
        .reader("targets.json")?
        .map(serde_json::from_reader::<_, Signed<crate::serde::Targets>>)
    {
        ensure!(
            old_targets.signed.version <= targets.signed.version,
            error::OlderMetadata {
                role: Role::Targets,
                current_version: old_targets.signed.version,
                new_version: targets.signed.version
            }
        );
    }

    // 4.4. Check for a freeze attack. The latest known time should be lower than the expiration
    //   timestamp in the new targets metadata file. If so, the new targets metadata file becomes
    //   the trusted targets metadata file. If the new targets metadata file is expired, discard
    //   it, abort the update cycle, and report the potential freeze attack.
    targets.check_expired()?;

    // 4.5. Perform a preorder depth-first search for metadata about the desired target, beginning
    //   with the top-level targets role.
    //
    // (This library does not yet handle delegated roles, so we just use the parsed targets from
    // targets.json.)

    Ok(targets)
}
