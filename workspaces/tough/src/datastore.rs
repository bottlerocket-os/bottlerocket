use crate::error::{self, Result};
use serde::Serialize;
use snafu::ResultExt;
use std::fs::{self, File};
use std::io::{ErrorKind, Read};
use std::path::{Path, PathBuf};

pub(crate) struct Datastore(PathBuf);

impl Datastore {
    pub(crate) fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        Ok(Self(path.as_ref().to_owned()))
    }

    pub(crate) fn reader(&self, file: &str) -> Result<Option<impl Read>> {
        let path = self.0.join(file);
        match File::open(&path) {
            Ok(file) => Ok(Some(file)),
            Err(err) => match err.kind() {
                ErrorKind::NotFound => Ok(None),
                _ => Err(err).context(error::DatastoreOpen { path: &path }),
            },
        }
    }

    pub(crate) fn create<T: Serialize>(&self, file: &str, value: &T) -> Result<()> {
        let path = self.0.join(file);
        serde_json::to_writer_pretty(
            File::create(&path).context(error::DatastoreCreate { path: &path })?,
            value,
        )
        .context(error::JsonSerialization {
            what: format!("{} in datastore", file),
        })
    }

    pub(crate) fn remove(&self, file: &str) -> Result<()> {
        let path = self.0.join(file);
        match fs::remove_file(&path) {
            Ok(()) => Ok(()),
            Err(err) => match err.kind() {
                ErrorKind::NotFound => Ok(()),
                _ => Err(err).context(error::DatastoreRemove { path: &path }),
            },
        }
    }
}
