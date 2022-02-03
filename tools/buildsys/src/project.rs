/*!
This module handles iterating through project directories to discover source
files that should be passed to Cargo to watch for changes.

For now, it's a thin wrapper around `walkdir` with a filter applied to ignore
files that shouldn't trigger rebuilds.

*/
pub(crate) mod error;
use error::Result;

use snafu::ResultExt;
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

pub(crate) struct ProjectInfo {
    pub(crate) files: Vec<PathBuf>,
}

impl ProjectInfo {
    /// Traverse the list of directories and produce a list of files to track.
    pub(crate) fn crawl<P: AsRef<Path>>(dirs: &[P]) -> Result<Self> {
        let mut files = Vec::new();

        for dir in dirs {
            let walker = WalkDir::new(dir)
                .follow_links(false)
                .same_file_system(true)
                .into_iter();

            files.extend(
                walker
                    .filter_entry(|e| !Self::ignored(e))
                    .flat_map(|e| e.context(error::DirectoryWalkSnafu))
                    .map(|e| e.into_path())
                    .filter(|e| e.is_file()),
            );
        }

        Ok(ProjectInfo { files })
    }

    /// Exclude hidden files and build artifacts from the list.
    fn ignored(entry: &DirEntry) -> bool {
        entry
            .file_name()
            .to_str()
            .map(|s| s.starts_with('.') || s == "target" || s == "vendor" || s == "README.md")
            .unwrap_or(false)
    }
}
