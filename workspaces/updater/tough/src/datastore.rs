use crate::error::{self, Result};
use serde::Serialize;
use snafu::ResultExt;
use std::fs::{self, File};
use std::io::{ErrorKind, Read};
use std::path::Path;
use std::sync::{Arc, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};

#[derive(Debug, Clone)]
pub(crate) struct Datastore<'a>(Arc<RwLock<&'a Path>>);

impl<'a> Datastore<'a> {
    pub(crate) fn new(path: &'a Path) -> Self {
        Self(Arc::new(RwLock::new(path)))
    }

    // Because we are not actually changing the underlying data in the lock, we can ignore when a
    // lock is poisoned.

    fn read(&self) -> RwLockReadGuard<'_, &'a Path> {
        self.0.read().unwrap_or_else(PoisonError::into_inner)
    }

    fn write(&self) -> RwLockWriteGuard<'_, &'a Path> {
        self.0.write().unwrap_or_else(PoisonError::into_inner)
    }

    pub(crate) fn reader(&self, file: &str) -> Result<Option<impl Read>> {
        let path = self.read().join(file);
        match File::open(&path) {
            Ok(file) => Ok(Some(file)),
            Err(err) => match err.kind() {
                ErrorKind::NotFound => Ok(None),
                _ => Err(err).context(error::DatastoreOpen { path: &path }),
            },
        }
    }

    pub(crate) fn create<T: Serialize>(&self, file: &str, value: &T) -> Result<()> {
        let path = self.write().join(file);
        serde_json::to_writer_pretty(
            File::create(&path).context(error::DatastoreCreate { path: &path })?,
            value,
        )
        .context(error::DatastoreSerialize {
            what: format!("{} in datastore", file),
            path,
        })
    }

    pub(crate) fn remove(&self, file: &str) -> Result<()> {
        let path = self.write().join(file);
        match fs::remove_file(&path) {
            Ok(()) => Ok(()),
            Err(err) => match err.kind() {
                ErrorKind::NotFound => Ok(()),
                _ => Err(err).context(error::DatastoreRemove { path: &path }),
            },
        }
    }
}
