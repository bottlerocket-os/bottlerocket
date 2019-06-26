//! This implementation of the DataStore trait relies on the filesystem for data and metadata
//! storage.
//!
//! Data is kept in files with paths resembling the keys, e.g. a/b/c for a.b.c, and metadata is
//! kept in a suffixed file next to the data, e.g. a/b/c.meta for metadata "meta" about a.b.c

use glob::glob;
use lazy_static::lazy_static;
use regex::Regex;
use snafu::{ensure, OptionExt, ResultExt};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;
use std::path::{self, Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

use super::key::{Key, KeyType, KEY_SEGMENT, KEY_SEGMENT_STR, KEY_SEPARATOR};
use super::serialization::to_pairs;
use super::{error, serialize_scalar, Committed, DataStore, Result, ScalarError};
use crate::model::Metadata;

const METADATA_KEY_PREFIX: char = '.';

lazy_static! {
    /// Pattern to validate the filename for a metadata key.
    // One data segment, a dot, and one metadata segment.
    static ref METADATA_FILENAME: Regex = Regex::new(
        &format!(r"^(?P<data>{segment}).(?P<metadata>{segment})$", segment=KEY_SEGMENT_STR)
    ).unwrap();
}

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

// Filesystem read/write/copy helpers

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

/// Given a DirEntry, returns Ok(Some(Key)) if it seems like a datastore key of the given
/// key_type.  Returns Ok(None) if it doesn't seem like a datastore key, e.g. a directory, or is
/// the wrong key_type.  Returns Err if we weren't able to check or if it doesn't seem like
/// something that should be in the datastore directory at all.
fn key_for_entry<P: AsRef<Path>>(
    entry: &DirEntry,
    key_type: KeyType,
    strip_path_prefix: P,
) -> Result<Option<Key>> {
    if !entry.file_type().is_file() {
        trace!("Skipping non-file entry: {}", entry.path().display());
        return Ok(None);
    }

    let check_path = |p: Option<_>| -> Result<_> {
        p.context(error::Corruption {
            msg: "Non-UTF8 path",
            path: entry.path(),
        })
    };

    // Check if the filename is valid as the requested key_type; if not, we return Ok(None) to
    // represent that we successfully figured out that it's not the requested type.
    let filename = check_path(entry.file_name().to_str())?;
    let name_pattern = match key_type {
        KeyType::Data => &*KEY_SEGMENT,
        KeyType::Meta => &*METADATA_FILENAME,
    };
    if !name_pattern.is_match(filename) {
        trace!(
            "Skipping file not valid as {:?}: {}",
            key_type,
            entry.path().display()
        );
        return Ok(None);
    }

    let path = entry.path();
    let key_path = path.strip_prefix(strip_path_prefix).context(error::Path)?;
    let key_path_str = check_path(key_path.to_str())?;

    let key_name = key_path_str.replace("/", KEY_SEPARATOR);
    trace!(
        "Made key name '{}' from path: {}",
        key_name,
        entry.path().display()
    );
    let key = Key::new(key_type, key_name)?;
    Ok(Some(key))
}

// TODO: maybe add/strip single newline at end, so file is easier to read
impl DataStore for FilesystemDataStore {
    fn key_populated(&self, key: &Key, committed: Committed) -> Result<bool> {
        let path = self.data_path(key, committed)?;

        Ok(path.exists())
    }

    /// We walk the filesystem to list populated keys.
    ///
    /// If we were to need to list all possible keys, a walk would only work if we had empty files
    /// to represent unset values, which could be ugly.
    ///
    /// Another option would be to use a procedural macro to step through a structure to list
    /// possible keys; this would be similar to serde, but would need to step through Option fields.
    fn list_populated_keys<S: AsRef<str>>(
        &self,
        prefix: S,
        committed: Committed,
    ) -> Result<HashSet<Key>> {
        let prefix = prefix.as_ref();

        let base = self.base_path(committed);
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

        let walker = WalkDir::new(base)
            .follow_links(false) // shouldn't be links...
            .same_file_system(true); // shouldn't be filesystems to cross...

        let mut keys: HashSet<Key> = HashSet::new();
        trace!(
            "Starting walk of filesystem to list keys, path: {}",
            base.display()
        );
        for entry in walker {
            let entry = entry.context(error::ListKeys)?;
            if let Some(key) = key_for_entry(&entry, KeyType::Data, &base)? {
                keys.insert(key);
            }
        }

        trace!("Removing keys not beginning with '{}'", prefix);
        // Note: Can't start walk at prefix because it may not be a valid path - e.g. could ask for
        // prefix of "sett" to get settings.  Could reconsider that behavior to optimize here.
        keys.retain(|k| k.starts_with(&prefix));

        Ok(keys)
    }

    /// To find populated metadata, we first find populated data with the given prefix, since
    /// metadata must be attached to data.  For each found data key, we list files starting with
    /// the same path but with a metadata extension.  We return a mapping of data keys to the
    /// metadata keys we found for each.
    fn list_populated_metadata<S: AsRef<str>>(
        &self,
        prefix: S,
    ) -> Result<HashMap<Key, HashSet<Key>>> {
        // Metadata can only be attached to populated data, so first we find data keys.
        let data_keys = self.list_populated_keys(prefix, Committed::Live)?;

        let mut result = HashMap::new();
        for data_key in data_keys {
            // For each found data key, we find the metadata keys next to it on disk.
            let mut result_for_key = HashSet::new();
            let data_path = self.data_path(&data_key, Committed::Live)?;
            let meta_path_glob = format!("{}.*", data_path.display());
            for meta_path in glob(&meta_path_glob).with_context(|| error::GlobPattern {
                glob: meta_path_glob.clone(),
            })? {
                // For each found metadata file, we insert a Key into the result.
                // We have the full path, so we need to pull out the filename, then the metadata
                // key portion of the filename using the structure of the key naming regex.
                let meta_path = meta_path.with_context(|| error::GlobIo {
                    glob: meta_path_glob.clone(),
                })?;
                let meta_file = meta_path.file_name().context(error::Internal {
                    msg: format!("Metadata path has no file name: {}", meta_path.display()),
                })?;
                let meta_file_str = meta_file.to_str().context(error::NonUnicodeFile {
                    file: meta_file.to_string_lossy(),
                    context: "listing populated metadata",
                })?;
                let meta_file_captures =
                    METADATA_FILENAME
                        .captures(meta_file_str)
                        .context(error::Internal {
                            msg: format!(
                                "Unable to capture metadata name from filename '{}'",
                                meta_file_str
                            ),
                        })?;
                let meta_name = meta_file_captures
                    .name("metadata")
                    .context(error::Internal {
                        msg: format!("Did not find metadata name in filename '{}'", meta_file_str),
                    })?.as_str();

                let meta_key = Key::new(KeyType::Meta, meta_name)
                    .ok()
                    .with_context(|| error::Internal {
                        msg: format!("Couldn't build Key from listed metadata: {}", meta_file_str),
                    })?;
                result_for_key.insert(meta_key);
            }

            // Only want to return data keys for which we found metadata
            if !result_for_key.is_empty() {
                result.insert(data_key, result_for_key);
            }
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
