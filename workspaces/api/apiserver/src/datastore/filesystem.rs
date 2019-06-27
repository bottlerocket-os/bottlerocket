//! This implementation of the DataStore trait relies on the filesystem for data and metadata
//! storage.
//!
//! Data is kept in files with paths resembling the keys, e.g. a/b/c for a.b.c, and metadata is
//! kept in a suffixed file next to the data, e.g. a/b/c.meta for metadata "meta" about a.b.c

use snafu::{ensure, OptionExt, ResultExt};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;
use std::path::{self, Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

use super::key::{Key, KeyType, KEY_SEPARATOR};
use super::serialization::to_pairs;
use super::{error, serialize_scalar, Committed, DataStore, Result, ScalarError};
use crate::model::Metadata;

const METADATA_KEY_PREFIX: char = '.';

#[derive(Debug)]
pub struct FilesystemDataStore {
    live_path: PathBuf,
    pending_path: PathBuf,
}

impl FilesystemDataStore {
    pub fn new<P: AsRef<Path>>(base_path: P) -> FilesystemDataStore {
        FilesystemDataStore {
            live_path: base_path.as_ref().join("live"),
            pending_path: base_path.as_ref().join("pending"),
        }
    }

    /// Creates a new FilesystemDataStore at the given path, with data and metadata coming from
    /// defaults.toml at compile time.
    pub fn populate_default<P: AsRef<Path>>(base_path: P) -> Result<()> {
        // Read and parse defaults
        let defaults_str = include_str!("../../defaults.toml");
        let mut defaults_val: toml::Value =
            toml::from_str(defaults_str).context(error::DefaultsFormatting)?;

        // Check if we have metadata
        let table = defaults_val
            .as_table_mut()
            .context(error::DefaultsNotTable)?;
        let maybe_metadata_val = table.remove("metadata");

        // Write defaults to datastore
        trace!("Serializing defaults and writing to datastore");
        let defaults =
            to_pairs(&defaults_val).context(error::Serialization { given: "defaults" })?;
        let mut datastore = FilesystemDataStore::new(base_path);
        datastore.set_keys(&defaults, Committed::Live)?;

        // If we had metadata, write it out
        if let Some(metadata_val) = maybe_metadata_val {
            trace!("Serializing metadata and writing to datastore");
            let metadatas: Vec<Metadata> = metadata_val
                .try_into()
                .context(error::DefaultsMetadataNotTable)?;
            for metadata in metadatas {
                let Metadata { key, md, val } = metadata;
                let data_key = Key::new(KeyType::Data, key)?;
                let md_key = Key::new(KeyType::Data, md)?;
                let value = serialize_scalar::<_, ScalarError>(&val).with_context(|| {
                    error::SerializeScalar {
                        given: format!("metadata value '{}'", val),
                    }
                })?;

                datastore.set_metadata(&md_key, &data_key, value)?;
            }
        }

        Ok(())
    }

    /// Returns the appropriate filesystem path for pending or live data.
    fn base_path(&self, committed: Committed) -> &PathBuf {
        match committed {
            Committed::Pending => &self.pending_path,
            Committed::Live => &self.live_path,
        }
    }

    /// Returns the appropriate path on the filesystem for the given data key.
    fn data_path(&self, key: &Key, committed: Committed) -> Result<PathBuf> {
        let base_path = self.base_path(committed);

        // turn dot-separated key into slash-separated path suffix
        let path_suffix = key.replace(KEY_SEPARATOR, &path::MAIN_SEPARATOR.to_string());

        // Make path from base + prefix
        // FIXME: canonicalize requires that the full path exists.  We know our Key is checked
        // for acceptable characters, so join should be safe enough, but come back to this.
        // let path = fs::canonicalize(self.base_path.join(path_suffix))?;
        let path = base_path.join(path_suffix);

        // Confirm no path traversal outside of base
        ensure!(
            path != *base_path && path.starts_with(base_path),
            error::PathTraversal { name: key.as_ref() }
        );

        Ok(path)
    }

    /// Returns the appropriate path on the filesystem for the given metadata key.
    fn metadata_path(
        &self,
        metadata_key: &Key,
        data_key: &Key,
        committed: Committed,
    ) -> Result<PathBuf> {
        let data_path = self.data_path(data_key, committed)?;
        let data_path_str = data_path.to_str().expect("Key paths must be UTF-8");

        let segments: Vec<&str> = data_path_str.rsplitn(2, path::MAIN_SEPARATOR).collect();
        let (basename, dirname) = match segments.len() {
            2 => (segments[0], segments[1]),
            _ => panic!("Grave error with path generation; invalid base path?"),
        };

        let filename = basename.to_owned() + &METADATA_KEY_PREFIX.to_string() + metadata_key;
        Ok(Path::new(dirname).join(filename))
    }
}

// Filesystem helpers

/// Helper for reading a key from the filesystem.  Returns Ok(None) if the file doesn't exist
/// rather than erroring.
fn read_file_for_key(key: &Key, path: &Path) -> Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(s) => Ok(Some(s)),
        Err(e) => {
            if e.kind() == io::ErrorKind::NotFound {
                return Ok(None);
            }

            Err(e).context(error::KeyRead { key: key.as_ref() })
        }
    }
}

/// Helper for writing a file that makes the directory tree beforehand, so we can handle
/// arbitrarily dotted keys without needing to create fixed structure first.
fn write_file_mkdir<S: AsRef<str>>(path: PathBuf, data: S) -> Result<()> {
    // create key prefix directory if necessary
    let dirname = path.parent().with_context(|| error::Internal {
        msg: format!(
            "Given path to write without proper prefix: {}",
            path.display()
        ),
    })?;
    fs::create_dir_all(dirname).context(error::Io { path: dirname })?;

    fs::write(&path, data.as_ref().as_bytes()).context(error::Io { path: &path })
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
    fn new(path: &Path) -> Result<KeyPath> {
        let path_str = path.to_str().context(error::Corruption {
            msg: "Non-UTF8 path",
            path,
        })?;

        let mut segments = path_str.splitn(2, '.');

        // Split the data and metadata parts
        let data_key_raw = segments.next().context(error::Internal {
            msg: "KeyPath given empty path",
        })?;
        // Turn the data path into a dotted key
        let data_key_str = data_key_raw.replace("/", KEY_SEPARATOR);
        let data_key = Key::new(KeyType::Data, data_key_str)?;

        // If we have a metadata portion, make that a Key too
        let metadata_key = match segments.next() {
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

/// Given a DirEntry, gives you a KeyPath if it's a valid path to a key.  Specifically, we return
/// Ok(Some(Key)) if it seems like a datastore key.  Returns Ok(None) if it doesn't seem like a
/// datastore key, e.g. a directory, or if it's a file otherwise invalid as a key.  Returns Err if
/// we weren't able to check.
fn key_path_for_entry<P: AsRef<Path>>(
    entry: &DirEntry,
    strip_path_prefix: P,
) -> Result<Option<KeyPath>> {
    if !entry.file_type().is_file() {
        trace!("Skipping non-file entry: {}", entry.path().display());
        return Ok(None);
    }

    let path = entry.path();
    let key_path_raw = path.strip_prefix(strip_path_prefix).context(error::Path)?;
    // If KeyPath doesn't think this is an OK key, we'll return Ok(None), otherwise the KeyPath
    Ok(KeyPath::new(key_path_raw).ok())
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
    committed: Committed,
) -> Result<HashSet<KeyPath>> {
    // Find the base path for our search, and confirm it exists.
    let base = datastore.base_path(committed);
    if !base.exists() {
        match committed {
            // No live keys; something must be wrong because we create a default datastore.
            Committed::Live => {
                return error::Corruption {
                    msg: "Live datastore missing",
                    path: base,
                }
                .fail()
            }
            // No pending keys, OK, return empty set.
            Committed::Pending => {
                trace!(
                    "Returning empty list because pending path doesn't exist: {}",
                    base.display()
                );
                return Ok(HashSet::new());
            }
        }
    }

    // Walk through the filesystem.
    let walker = WalkDir::new(base)
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
        let entry = entry.context(error::ListKeys)?;
        if let Some(kp) = key_path_for_entry(&entry, &base)? {
            if !kp.data_key.as_ref().starts_with(prefix.as_ref()) {
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
    fn key_populated(&self, key: &Key, committed: Committed) -> Result<bool> {
        let path = self.data_path(key, committed)?;

        Ok(path.exists())
    }

    /// Returns the set of all data keys that are currently populated in the datastore, that
    /// start with the given prefix.
    fn list_populated_keys<S: AsRef<str>>(
        &self,
        prefix: S,
        committed: Committed,
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
        let key_paths = find_populated_key_paths(self, KeyType::Meta, prefix, Committed::Live)?;

        // For each file on disk, check the user's conditions, and add it to our output
        let mut result = HashMap::new();
        for key_path in key_paths {
            let data_key = key_path.data_key;
            let meta_key = key_path.metadata_key.context(error::Internal {
                msg: format!("Found meta key path with no dot: {}", data_key),
            })?;

            // If the user requested specific metadata, move to the next key unless it matches.
            if let Some(name) = metadata_key_name {
                if name.as_ref() != meta_key.as_ref() {
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

    fn get_key(&self, key: &Key, committed: Committed) -> Result<Option<String>> {
        let path = self.data_path(key, committed)?;
        read_file_for_key(&key, &path)
    }

    fn set_key<S: AsRef<str>>(&mut self, key: &Key, value: S, committed: Committed) -> Result<()> {
        let path = self.data_path(key, committed)?;
        write_file_mkdir(path, value)
    }

    fn get_metadata_raw(&self, metadata_key: &Key, data_key: &Key) -> Result<Option<String>> {
        let path = self.metadata_path(metadata_key, data_key, Committed::Live)?;
        read_file_for_key(&metadata_key, &path)
    }

    fn set_metadata<S: AsRef<str>>(
        &mut self,
        metadata_key: &Key,
        data_key: &Key,
        value: S,
    ) -> Result<()> {
        let path = self.metadata_path(metadata_key, data_key, Committed::Live)?;
        write_file_mkdir(path, value)
    }

    /// We commit by copying pending keys to live, then removing pending.  Something smarter (lock,
    /// atomic flip, etc.) will be required to make the server concurrent.
    fn commit(&mut self) -> Result<HashSet<Key>> {
        // Get data for changed keys
        let pending_data = self.get_prefix("settings.", Committed::Pending)?;

        // Turn String keys of pending data into Key keys, for return
        let try_pending_keys: Result<HashSet<Key>> = pending_data
            .keys()
            .map(|s| Key::new(KeyType::Data, s))
            .collect();
        let pending_keys = try_pending_keys?;

        // Apply changes to live
        debug!("Writing pending keys to live");
        self.set_keys(&pending_data, Committed::Live)?;

        // Remove pending
        debug!("Removing old pending keys");
        fs::remove_dir_all(&self.pending_path).context(error::Io {
            path: &self.pending_path,
        })?;

        Ok(pending_keys)
    }
}

#[cfg(test)]
mod test {
    use super::{Committed, FilesystemDataStore, Key, KeyType};

    #[test]
    fn data_path() {
        let f = FilesystemDataStore::new("/base");
        let key = Key::new(KeyType::Data, "a.b.c").unwrap();

        let pending = f.data_path(&key, Committed::Pending).unwrap();
        assert_eq!(pending.into_os_string(), "/base/pending/a/b/c");

        let live = f.data_path(&key, Committed::Live).unwrap();
        assert_eq!(live.into_os_string(), "/base/live/a/b/c");
    }

    #[test]
    fn metadata_path() {
        let f = FilesystemDataStore::new("/base");
        let data_key = Key::new(KeyType::Data, "a.b.c").unwrap();
        let md_key = Key::new(KeyType::Meta, "my-metadata").unwrap();

        let pending = f
            .metadata_path(&md_key, &data_key, Committed::Pending)
            .unwrap();
        assert_eq!(pending.into_os_string(), "/base/pending/a/b/c.my-metadata");

        let live = f
            .metadata_path(&md_key, &data_key, Committed::Live)
            .unwrap();
        assert_eq!(live.into_os_string(), "/base/live/a/b/c.my-metadata");
    }
}
