use crate::error::Result;
use crate::io::{DigestAdapter, MaxSizeAdapter};
use reqwest::{Client, Url};
use std::io::Read;

// Test mock that allows fetching from file:/// URLs relative to crate root
#[cfg(test)]
fn fetch(_: &Client, url: Url) -> Result<impl Read> {
    use std::fs::File;
    use std::path::PathBuf;

    assert!(
        url.scheme() == "file",
        "non-file URL schemes not supported in tests"
    );
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(url.path());
    Ok(File::open(path).unwrap())
}

#[cfg(not(test))]
fn fetch(client: &Client, url: Url) -> Result<impl Read> {
    use crate::error;
    use snafu::ResultExt;

    client
        .get(url.clone())
        .send()
        .context(error::Request { url })
}

pub(crate) fn fetch_max_size(client: &Client, url: Url, max_size: usize) -> Result<impl Read> {
    Ok(MaxSizeAdapter::new(fetch(client, url)?, max_size))
}

pub(crate) fn fetch_sha256(
    client: &Client,
    url: Url,
    size: usize,
    sha256: &[u8],
) -> Result<impl Read> {
    Ok(DigestAdapter::sha256(
        MaxSizeAdapter::new(fetch(client, url)?, size),
        sha256,
    ))
}
