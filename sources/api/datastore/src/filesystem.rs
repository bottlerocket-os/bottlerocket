//! This implementation of the DataStore trait relies on the filesystem for data and metadata
//! storage.
//!
//! Data is kept in files with paths resembling the keys, e.g. a/b/c for a.b.c, and metadata is
//! kept in a suffixed file next to the data, e.g. a/b/c.meta for metadata "meta" about a.b.c

use log::{debug, error, trace};
use percent_encoding::{percent_decode_str, utf8_percent_encode, AsciiSet, NON_ALPHANUMERIC};
use snafu::{ensure, OptionExt, ResultExt};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;
use std::path::{self, Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

use super::key::{Key, KeyType};
use super::{error, Committed, DataStore, Result};

const METADATA_KEY_PREFIX: &str = ".";

// This describes the set of characters we encode when making the filesystem path for a given key.
// Any non-ASCII characters, plus these ones, will be encoded.
// We start off very strict (anything not alphanumeric) and remove characters we'll allow.
// To make inspecting the filesystem easier, we allow any filesystem-safe characters that are
// allowed in a Key.
const ENCODE_CHARACTERS: &AsciiSet = &NON_ALPHANUMERIC.remove(b'_').remove(b'-');

#[derive(Debug)]
pub struct FilesystemDataStore {
    live_path: PathBuf,
    pending_base_path: PathBuf,
}

impl FilesystemDataStore {
    pub fn new<P: AsRef<Path>>(base_path: P) -> FilesystemDataStore {
        FilesystemDataStore {
            live_path: base_path.as_ref().join("live"),
            pending_base_path: base_path.as_ref().join("pending"),
        }
    }

    /// Returns the appropriate filesystem path for pending or live data.
    fn base_path(&self, committed: &Committed) -> PathBuf {
        match committed {
            Committed::Pending { tx } => {
                let encoded = encode_path_component(tx);
                self.pending_base_path.join(encoded)
            }
            Committed::Live => self.live_path.clone(),
        }
    }

    /// Returns the appropriate path on the filesystem for the given data key.
    fn data_path(&self, key: &Key, committed: &Committed) -> Result<PathBuf> {
        let base_path = self.base_path(committed);

        // Encode key segments so they're filesystem-safe
        let encoded: Vec<_> = key.segments().iter().map(encode_path_component).collect();
        // Join segments with filesystem separator to get path underneath data store
        let path_suffix = encoded.join(path::MAIN_SEPARATOR_STR);

        // Make path from base + prefix
        // FIXME: canonicalize requires that the full path exists.  We know our Key is checked
        // for acceptable characters, so join should be safe enough, but come back to this.
        // let path = fs::canonicalize(self.base_path.join(path_suffix))?;
        let path = base_path.join(path_suffix);

        // Confirm no path traversal outside of base
        ensure!(
            path != *base_path && path.starts_with(base_path),
            error::PathTraversalSnafu { name: key.name() }
        );

        Ok(path)
    }

    /// Returns the appropriate path on the filesystem for the given metadata key.
    fn metadata_path(
        &self,
        metadata_key: &Key,
        data_key: &Key,
        committed: &Committed,
    ) -> Result<PathBuf> {
        let path = self.data_path(data_key, committed)?;

        // We want to add to the existing file name, not create new path components (directories),
        // so we use a string type rather than a path type.
        let mut path_str = path.into_os_string();

        // Key names have quotes as necessary to identify segments with special characters, so
        // we don't think "a.b" is actually two segments, for example.
        // Metadata keys only have a single segment, and we encode that as a single path
        // component, so we don't need the quotes in the filename.
        let raw_key_name = metadata_key
            .segments()
            .get(0)
            .context(error::InternalSnafu {
                msg: "metadata key with no segments",
            })?;

        let encoded_meta = encode_path_component(raw_key_name);
        path_str.push(METADATA_KEY_PREFIX);
        path_str.push(encoded_meta);

        Ok(path_str.into())
    }

    /// Deletes the given path from the filesystem.  Also removes the parent directory if empty
    /// (repeatedly, up to the base path), so as to have consistent artifacts on the filesystem
    /// after adding and removing keys.
    ///
    /// If the path doesn't exist, we still return Ok for idempotency, but if it exists and we
    /// fail to remove it, we return Err.
    ///
    /// If we fail to remove an empty directory, we log an error, but still return Ok.  (The
    /// error for trying to remove an empty directory is not specific, and we don't want to rely
    /// on platform-specific error codes or the error description.  We could check the directory
    /// contents ourself, but it would be more complex and subject to timing issues.)
    fn delete_key_path<P>(&mut self, path: P, committed: &Committed) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();

        // Remove the file.  If it doesn't exist, we're still OK.
        match fs::remove_file(path) {
            Ok(()) => {}
            Err(e) => {
                if e.kind() != io::ErrorKind::NotFound {
                    return Err(e).context(error::DeleteKeySnafu { path });
                }
            }
        }

        // Remove the directory if it's empty, i.e. if the setting we removed was the last setting
        // in that prefix.  Continue up the tree until the base, in case it was the only thing in
        // that subtree.
        let base = self.base_path(committed);
        if let Some(parent) = path.parent() {
            // Note: ancestors() includes 'parent' itself
            for parent in parent.ancestors() {
                // Stop at the base directory; we don't expect anything here or above to be empty,
                // but stop as a safeguard.
                if parent == base {
                    break;
                }
                if let Err(e) = fs::remove_dir(parent) {
                    // If the directory doesn't exist, continue up the tree.  Modulo timing issues,
                    // this means the key didn't exist either, which means a previous attempt to remove
                    // the directory failed or we got an unset request for a bogus key.  Either way, we
                    // can clean up and make things consistent.
                    if e.kind() == io::ErrorKind::NotFound {
                        continue;

                    // "Directory not empty" doesn't have its own ErrorKind, so we have to check a
                    // platform-specific error number or the error description, neither of which is
                    // ideal.  Still, we can at least log an error in the case we know.  Don't
                    // fail, though, because we've still accomplished our main purpose.
                    } else if e.raw_os_error() != Some(39) {
                        error!(
                            "Failed to delete directory '{}' we believe is empty: {}",
                            parent.display(),
                            e
                        );
                    }
                    // We won't be able to delete parent directories if this one still exists.
                    break;
                }
            }
        }
        Ok(())
    }
}

// Filesystem helpers

/// Encodes a string so that it's safe to use as a filesystem path component.
fn encode_path_component<S: AsRef<str>>(segment: S) -> String {
    let encoded = utf8_percent_encode(segment.as_ref(), ENCODE_CHARACTERS);
    encoded.to_string()
}

/// Decodes a path component, removing the encoding that's applied to make it filesystem-safe.
fn decode_path_component<S, P>(segment: S, path: P) -> Result<String>
where
    S: AsRef<str>,
    P: AsRef<Path>,
{
    let segment = segment.as_ref();

    percent_decode_str(segment)
        .decode_utf8()
        // Get back a plain String.
        .map(|cow| cow.into_owned())
        // decode_utf8 will only fail if someone messed with the filesystem contents directly
        // and created a filename that contains percent-encoded bytes that are invalid UTF-8.
        .ok()
        .context(error::CorruptionSnafu {
            path: path.as_ref(),
            msg: format!("invalid UTF-8 in encoded segment '{}'", segment),
        })
}

/// Helper for reading a key from the filesystem.  Returns Ok(None) if the file doesn't exist
/// rather than erroring.
fn read_file_for_key(key: &Key, path: &Path) -> Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(s) => Ok(Some(s)),
        Err(e) => {
            if e.kind() == io::ErrorKind::NotFound {
                return Ok(None);
            }

            Err(e).context(error::KeyReadSnafu { key: key.name() })
        }
    }
}

/// Helper for writing a file that makes the directory tree beforehand, so we can handle
/// arbitrarily dotted keys without needing to create fixed structure first.
fn write_file_mkdir<S: AsRef<str>>(path: PathBuf, data: S) -> Result<()> {
    // create key prefix directory if necessary
    let dirname = path.parent().with_context(|| error::InternalSnafu {
        msg: format!(
            "Given path to write without proper prefix: {}",
            path.display()
        ),
    })?;
    fs::create_dir_all(dirname).context(error::IoSnafu { path: dirname })?;

    fs::write(&path, data.as_ref().as_bytes()).context(error::IoSnafu { path: &path })
}

/// KeyPath represents the filesystem path to a data or metadata key, relative to the base path of
/// the live or pending data store.  For example, the data key "settings.a.b" would be
/// "settings/a/b" and the metadata key "meta1" for "settings.a.b" would be "settings/a/b.meta1".
///
/// It allows access to the data_key and (if it's a metadata key) the metadata_key based on the
/// path.
///
/// This structure can be useful when it doesn't matter where the key is physically stored, but
/// you still need to deal with the interaction between key name and filename, e.g. when
/// abstracting over data and metadata keys during a search.
// Note: this may be useful in other parts of the FilesystemDataStore code too.  It may also be
// useful enough to use its ideas to extend the Key type directly, instead.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct KeyPath {
    data_key: Key,
    metadata_key: Option<Key>,
}

impl KeyPath {
    /// Given a DirEntry, gives you a KeyPath if it's a valid path to a key.  Specifically, we return
    /// Ok(Some(Key)) if it seems like a datastore key.  Returns Ok(None) if it doesn't seem like a
    /// datastore key, e.g. a directory, or if it's a file otherwise invalid as a key.  Returns Err if
    /// we weren't able to check.
    fn from_entry<P: AsRef<Path>>(
        entry: &DirEntry,
        strip_path_prefix: P,
    ) -> Result<Option<KeyPath>> {
        if !entry.file_type().is_file() {
            trace!("Skipping non-file entry: {}", entry.path().display());
            return Ok(None);
        }

        let key_path_raw = entry
            .path()
            .strip_prefix(strip_path_prefix)
            .context(error::PathSnafu)?;
        // If from_path doesn't think this is an OK key, we'll return Ok(None), otherwise the KeyPath
        Ok(Self::from_path(key_path_raw).ok())
    }

    fn from_path(path: &Path) -> Result<KeyPath> {
        let path_str = path.to_str().context(error::CorruptionSnafu {
            msg: "Non-UTF8 path",
            path,
        })?;

        // Split the data and metadata parts.
        // Any dots in key names are encoded.
        let mut keys = path_str.splitn(2, '.');
        let data_key_raw = keys.next().context(error::InternalSnafu {
            msg: "KeyPath given empty path",
        })?;
        // Turn the data path into a dotted key
        let data_segments = data_key_raw
            .split(path::MAIN_SEPARATOR)
            .map(|s| decode_path_component(s, path))
            .collect::<Result<Vec<_>>>()?;
        let data_key = Key::from_segments(KeyType::Data, &data_segments)?;

        // If we have a metadata portion, make that a Key too
        let metadata_key = match keys.next() {
            Some(meta_key_str) => Some(Key::new(KeyType::Meta, meta_key_str)?),
            None => None,
        };

        Ok(KeyPath {
            data_key,
            metadata_key,
        })
    }

    fn key_type(&self) -> KeyType {
        match self.metadata_key {
            Some(_) => KeyType::Meta,
            None => KeyType::Data,
        }
    }
}

/// Helper to walk through the filesystem to find populated keys of the given type, starting with
/// the given prefix.  Each item in the returned set is a KeyPath representing a data or metadata
/// key.
// Note: if we needed to list all possible keys, a walk would only work if we had empty files to
// represent unset values, which could be ugly.
// Another option would be to use a procedural macro to step through a structure to list possible
// keys; this would be similar to serde, but would need to step through Option fields.
fn find_populated_key_paths<S: AsRef<str>>(
    datastore: &FilesystemDataStore,
    key_type: KeyType,
    prefix: S,
    committed: &Committed,
) -> Result<HashSet<KeyPath>> {
    // Find the base path for our search, and confirm it exists.
    let base = datastore.base_path(committed);
    if !base.exists() {
        match committed {
            // No live keys; something must be wrong because we create a default datastore.
            Committed::Live => {
                return error::CorruptionSnafu {
                    msg: "Live datastore missing",
                    path: base,
                }
                .fail()
            }
            // No pending keys, OK, return empty set.
            Committed::Pending { .. } => {
                trace!(
                    "Returning empty list because pending path doesn't exist: {}",
                    base.display()
                );
                return Ok(HashSet::new());
            }
        }
    }

    // Walk through the filesystem.
    let walker = WalkDir::new(&base)
        .follow_links(false) // shouldn't be links...
        .same_file_system(true); // shouldn't be filesystems to cross...

    let mut key_paths = HashSet::new();
    trace!(
        "Starting walk of filesystem to list {:?} key paths under {}",
        key_type,
        base.display()
    );

    // For anything we find, confirm it matches the user's filters, and add it to results.
    for entry in walker {
        let entry = entry.context(error::ListKeysSnafu)?;
        if let Some(kp) = KeyPath::from_entry(&entry, &base)? {
            if !kp.data_key.name().starts_with(prefix.as_ref()) {
                trace!(
                    "Discarded {:?} key whose data_key '{}' doesn't start with prefix '{}'",
                    kp.key_type(),
                    kp.data_key,
                    prefix.as_ref()
                );
                continue;
            } else if kp.key_type() != key_type {
                continue;
            }

            trace!("Found {:?} key at {}", key_type, entry.path().display());
            key_paths.insert(kp);
        }
    }

    Ok(key_paths)
}

// TODO: maybe add/strip single newline at end, so file is easier to read
impl DataStore for FilesystemDataStore {
    fn key_populated(&self, key: &Key, committed: &Committed) -> Result<bool> {
        let path = self.data_path(key, committed)?;

        Ok(path.exists())
    }

    /// Returns the set of all data keys that are currently populated in the datastore, that
    /// start with the given prefix.
    fn list_populated_keys<S: AsRef<str>>(
        &self,
        prefix: S,
        committed: &Committed,
    ) -> Result<HashSet<Key>> {
        let key_paths = find_populated_key_paths(self, KeyType::Data, prefix, committed)?;
        let keys = key_paths.into_iter().map(|kp| kp.data_key).collect();
        Ok(keys)
    }

    /// Finds all metadata keys that are currently populated in the datastore whose data keys
    /// start with the given prefix.  If you specify metadata_key_name, only metadata keys with
    /// that name will be returned.
    ///
    /// Returns a mapping of the data keys to the set of populated metadata keys for each.
    ///
    /// Note: The data keys do not need to be populated themselves; sometimes metadata is used
    /// to help generate the data, for example.  (Committed status is then irrelevant, too.)
    fn list_populated_metadata<S1, S2>(
        &self,
        prefix: S1,
        metadata_key_name: &Option<S2>,
    ) -> Result<HashMap<Key, HashSet<Key>>>
    where
        S1: AsRef<str>,
        S2: AsRef<str>,
    {
        // Find metadata key paths on disk
        let key_paths = find_populated_key_paths(self, KeyType::Meta, prefix, &Committed::Live)?;

        // For each file on disk, check the user's conditions, and add it to our output
        let mut result = HashMap::new();
        for key_path in key_paths {
            let data_key = key_path.data_key;
            let meta_key = key_path.metadata_key.context(error::InternalSnafu {
                msg: format!("Found meta key path with no dot: {}", data_key),
            })?;

            // If the user requested specific metadata, move to the next key unless it matches.
            if let Some(name) = metadata_key_name {
                if name.as_ref() != meta_key.name() {
                    continue;
                }
            }

            // Insert into output if we met the requested conditions; don't add an entry for
            // the data key unless we did find some metadata.
            let data_entry = result.entry(data_key).or_insert_with(HashSet::new);
            data_entry.insert(meta_key);
        }
        Ok(result)
    }

    fn get_key(&self, key: &Key, committed: &Committed) -> Result<Option<String>> {
        let path = self.data_path(key, committed)?;
        read_file_for_key(key, &path)
    }

    fn set_key<S: AsRef<str>>(&mut self, key: &Key, value: S, committed: &Committed) -> Result<()> {
        let path = self.data_path(key, committed)?;
        write_file_mkdir(path, value)
    }

    fn unset_key(&mut self, key: &Key, committed: &Committed) -> Result<()> {
        let path = self.data_path(key, committed)?;
        self.delete_key_path(path, committed)
    }

    fn get_metadata_raw(&self, metadata_key: &Key, data_key: &Key) -> Result<Option<String>> {
        let path = self.metadata_path(metadata_key, data_key, &Committed::Live)?;
        read_file_for_key(metadata_key, &path)
    }

    fn set_metadata<S: AsRef<str>>(
        &mut self,
        metadata_key: &Key,
        data_key: &Key,
        value: S,
    ) -> Result<()> {
        let path = self.metadata_path(metadata_key, data_key, &Committed::Live)?;
        write_file_mkdir(path, value)
    }

    fn unset_metadata(&mut self, metadata_key: &Key, data_key: &Key) -> Result<()> {
        let path = self.metadata_path(metadata_key, data_key, &Committed::Live)?;
        self.delete_key_path(path, &Committed::Live)
    }

    /// We commit by copying pending keys to live, then removing pending.  Something smarter (lock,
    /// atomic flip, etc.) will be required to make the server concurrent.
    fn commit_transaction<S>(&mut self, transaction: S) -> Result<HashSet<Key>>
    where
        S: Into<String> + AsRef<str>,
    {
        let pending = Committed::Pending {
            tx: transaction.into(),
        };
        // Get data for changed keys
        let pending_data = self.get_prefix("settings.", &pending)?;

        // Nothing to do if no keys are present in pending
        if pending_data.is_empty() {
            return Ok(Default::default());
        }

        // Save Keys for return value
        let pending_keys: HashSet<Key> = pending_data.keys().cloned().collect();

        // Apply changes to live
        debug!("Writing pending keys to live");
        self.set_keys(&pending_data, &Committed::Live)?;

        // Remove pending
        debug!("Removing old pending keys");
        let path = self.base_path(&pending);
        fs::remove_dir_all(&path).context(error::IoSnafu { path })?;

        Ok(pending_keys)
    }

    fn delete_transaction<S>(&mut self, transaction: S) -> Result<HashSet<Key>>
    where
        S: Into<String> + AsRef<str>,
    {
        let pending = Committed::Pending {
            tx: transaction.into(),
        };
        // Get changed keys so we can return the list
        let pending_data = self.get_prefix("settings.", &pending)?;

        // Pull out just the keys so we can log them and return them
        let pending_keys = pending_data.into_keys().collect();
        debug!("Found pending keys: {:?}", &pending_keys);

        // Delete pending from the filesystem, same as a commit
        let path = self.base_path(&pending);
        debug!("Removing transaction directory {}", path.display());
        if let Err(e) = fs::remove_dir_all(&path) {
            // If path doesn't exist, it's fine, we'll just return an empty list.
            if e.kind() != io::ErrorKind::NotFound {
                return Err(e).context(error::IoSnafu { path });
            }
        }

        Ok(pending_keys)
    }

    /// We store transactions as subdirectories of the pending data store, so to list them we list
    /// the names of the subdirectories.
    fn list_transactions(&self) -> Result<HashSet<String>> {
        // Any directory under pending should be a transaction name.
        let walker = WalkDir::new(&self.pending_base_path)
            .min_depth(1)
            .max_depth(1);

        let mut transactions = HashSet::new();
        trace!(
            "Starting walk of filesystem to list transactions under {}",
            self.pending_base_path.display(),
        );

        for entry in walker {
            let entry = match entry {
                Ok(entry) => entry,
                Err(e) => {
                    if let Some(io_error) = e.io_error() {
                        // If there's no pending directory, that's OK, just return empty set.
                        if io_error.kind() == io::ErrorKind::NotFound {
                            break;
                        }
                    }
                    return Err(e).context(error::ListKeysSnafu);
                }
            };

            if entry.file_type().is_dir() {
                // The directory name should be valid UTF-8, encoded by encode_path_component,
                // or the data store has been corrupted.
                let file_name = entry.file_name().to_str().context(error::CorruptionSnafu {
                    msg: "Non-UTF8 path",
                    path: entry.path(),
                })?;
                let transaction = decode_path_component(file_name, entry.path())?;
                transactions.insert(transaction);
            }
        }

        Ok(transactions)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn data_path() {
        let f = FilesystemDataStore::new("/base");
        let key = Key::new(KeyType::Data, "a.b.c").unwrap();

        let tx = "test transaction";
        let pending = f
            .data_path(&key, &Committed::Pending { tx: tx.into() })
            .unwrap();
        assert_eq!(
            pending.into_os_string(),
            "/base/pending/test%20transaction/a/b/c"
        );

        let live = f.data_path(&key, &Committed::Live).unwrap();
        assert_eq!(live.into_os_string(), "/base/live/a/b/c");
    }

    #[test]
    fn metadata_path() {
        let f = FilesystemDataStore::new("/base");
        let data_key = Key::new(KeyType::Data, "a.b.c").unwrap();
        let md_key = Key::new(KeyType::Meta, "my-metadata").unwrap();

        let tx = "test transaction";
        let pending = f
            .metadata_path(&md_key, &data_key, &Committed::Pending { tx: tx.into() })
            .unwrap();
        assert_eq!(
            pending.into_os_string(),
            "/base/pending/test%20transaction/a/b/c.my-metadata"
        );

        let live = f
            .metadata_path(&md_key, &data_key, &Committed::Live)
            .unwrap();
        assert_eq!(live.into_os_string(), "/base/live/a/b/c.my-metadata");
    }

    #[test]
    fn encode_path_component_works() {
        assert_eq!(encode_path_component("a-b_42"), "a-b_42");
        assert_eq!(encode_path_component("a.b"), "a%2Eb");
        assert_eq!(encode_path_component("a/b"), "a%2Fb");
        assert_eq!(encode_path_component("a b%c<d>e"), "a%20b%25c%3Cd%3Ee");
    }

    #[test]
    fn decode_path_component_works() {
        assert_eq!(decode_path_component("a-b_42", "").unwrap(), "a-b_42");
        assert_eq!(decode_path_component("a%2Eb", "").unwrap(), "a.b");
        assert_eq!(decode_path_component("a%2Fb", "").unwrap(), "a/b");
        assert_eq!(
            decode_path_component("a%20b%25c%3Cd%3Ee", "").unwrap(),
            "a b%c<d>e"
        );

        // Invalid UTF-8
        decode_path_component("%C3%28", "").unwrap_err();
    }
}
