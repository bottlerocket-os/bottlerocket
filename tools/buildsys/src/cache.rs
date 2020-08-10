/*!
Many of the inputs to package builds are not source files tracked within the git
repository, but large binary artifacts such as tar archives that are independently
distributed by an upstream project.

This module provides the ability to retrieve and validate these external files,
given the (name, url, hash) data that uniquely identifies each file.

It implements a two-tier approach to retrieval: files are first pulled from the
"lookaside" cache and only fetched from the upstream site if that access fails.

*/
pub(crate) mod error;
use error::Result;

use super::manifest;
use sha2::{Digest, Sha512};
use snafu::{ensure, OptionExt, ResultExt};
use std::fs::{self, File};
use std::io::{self, BufWriter};
use std::path::{Path, PathBuf};

static LOOKASIDE_CACHE: &str = "https://cache.bottlerocket.aws";

pub(crate) struct LookasideCache;

impl LookasideCache {
    /// Fetch files stored out-of-tree and ensure they match the stored hash.
    pub(crate) fn fetch(files: &[manifest::ExternalFile]) -> Result<Self> {
        for f in files {
            let url_file_name = Self::extract_file_name(&f.url)?;
            let path = &f.path.as_ref().unwrap_or_else(|| &url_file_name);
            ensure!(
                path.components().count() == 1,
                error::ExternalFileName { path }
            );

            let hash = &f.sha512;
            if path.is_file() {
                match Self::verify_file(path, hash) {
                    Ok(_) => continue,
                    Err(e) => {
                        eprintln!("{}", e);
                        fs::remove_file(path).context(error::ExternalFileDelete { path })?;
                    }
                }
            }

            let name = path.display();
            let tmp = PathBuf::from(format!(".{}", name));

            // first check the lookaside cache
            let url = format!("{}/{}/{}/{}", LOOKASIDE_CACHE.to_string(), name, hash, name);
            match Self::fetch_file(&url, &tmp, hash) {
                Ok(_) => {
                    fs::rename(&tmp, path).context(error::ExternalFileRename { path: &tmp })?;
                    continue;
                }
                Err(e) => {
                    eprintln!("{}", e);
                }
            }

            // next check with upstream, if permitted
            if std::env::var("BUILDSYS_UPSTREAM_SOURCE_FALLBACK") == Ok("true".to_string()) {
                println!("Fetching {:?} from upstream source", url_file_name);
                Self::fetch_file(&f.url, &tmp, hash)?;
                fs::rename(&tmp, path).context(error::ExternalFileRename { path: &tmp })?;
            }
        }

        Ok(Self)
    }

    /// Retrieves a file from the specified URL and write it to the given path,
    /// then verifies the contents against the SHA-512 hash provided.
    fn fetch_file<P: AsRef<Path>>(url: &str, path: P, hash: &str) -> Result<()> {
        let path = path.as_ref();
        let mut resp = reqwest::blocking::get(url).context(error::ExternalFileRequest { url })?;
        let status = resp.status();
        ensure!(
            status.is_success(),
            error::ExternalFileFetch { url, status }
        );

        let f = File::create(path).context(error::ExternalFileOpen { path })?;
        let mut f = BufWriter::new(f);
        resp.copy_to(&mut f)
            .context(error::ExternalFileSave { path })?;
        drop(f);

        match Self::verify_file(path, hash) {
            Ok(_) => Ok(()),
            Err(e) => {
                fs::remove_file(path).context(error::ExternalFileDelete { path })?;
                Err(e)
            }
        }
    }

    fn extract_file_name(url: &str) -> Result<PathBuf> {
        let parsed = reqwest::Url::parse(url).context(error::ExternalFileUrl { url })?;
        let name = parsed
            .path_segments()
            .context(error::ExternalFileName { path: url })?
            .last()
            .context(error::ExternalFileName { path: url })?;
        Ok(name.into())
    }

    /// Reads a file from disk and compares it to the expected SHA-512 hash.
    fn verify_file<P: AsRef<Path>>(path: P, hash: &str) -> Result<()> {
        let path = path.as_ref();
        let mut f = File::open(path).context(error::ExternalFileOpen { path })?;
        let mut d = Sha512::new();

        io::copy(&mut f, &mut d).context(error::ExternalFileLoad { path })?;
        let digest = hex::encode(d.finalize());

        ensure!(digest == hash, error::ExternalFileVerify { path, hash });
        Ok(())
    }
}
