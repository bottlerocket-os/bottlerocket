/*!
This module provides a very simple parser for RPM spec files.

It does not attempt to expand macros or perform any meaningful validation. Its
only purpose is to extract Source and Patch declarations so they can be passed
to Cargo as files to watch for changes.

*/
pub(crate) mod error;
use error::Result;

use snafu::ResultExt;
use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

pub(crate) struct SpecInfo {
    pub(crate) sources: Vec<PathBuf>,
    pub(crate) patches: Vec<PathBuf>,
}

impl SpecInfo {
    /// Returns a list of 'Source' and 'Patch' lines found in a spec file.
    pub(crate) fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let (sources, patches) = Self::parse(path)?;
        let sources = Self::filter(&sources);
        let patches = Self::filter(&patches);
        Ok(Self { sources, patches })
    }

    /// "Parse" a spec file, extracting values of potential interest.
    fn parse<P: AsRef<Path>>(path: P) -> Result<(Vec<String>, Vec<String>)> {
        let path = path.as_ref();
        let f = File::open(path).context(error::SpecFileReadSnafu { path })?;
        let f = BufReader::new(f);

        let mut sources = Vec::new();
        let mut patches = Vec::new();

        for line in f.lines() {
            let line = line.context(error::SpecFileReadSnafu { path })?;

            let mut tokens = line.split_whitespace().collect::<VecDeque<&str>>();
            if let Some(t) = tokens.pop_front() {
                if t.starts_with("Source") {
                    if let Some(s) = tokens.pop_front() {
                        sources.push(s.into());
                    }
                } else if t.starts_with("Patch") {
                    if let Some(p) = tokens.pop_front() {
                        patches.push(p.into());
                    }
                }
            }
        }

        Ok((sources, patches))
    }

    /// Emitting a non-existent file for `rerun-if-changed` will cause Cargo
    /// to always repeat the build. Therefore we exclude "files" that do not
    /// exist or that point outside the package directory. We also exclude
    /// anything that appears to be an unexpanded macro.
    fn filter(input: &[String]) -> Vec<PathBuf> {
        input
            .iter()
            .filter(|s| !s.contains("%{"))
            .map(PathBuf::from)
            .filter(|p| p.components().count() == 1)
            .filter(|p| p.file_name().is_some())
            .collect()
    }
}
