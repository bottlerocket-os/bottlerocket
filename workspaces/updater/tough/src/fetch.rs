use crate::error::Result;
use crate::io::{DigestAdapter, MaxSizeAdapter};
use reqwest::{Client, Url};
use snafu::ensure;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

fn fetch(client: &Client, url: Url) -> Result<Box<dyn Read>> {
    use crate::error;
    use snafu::ResultExt;

    if url.scheme() == "file" {
        let path = PathBuf::from(url.path());
        let file = File::open(&path).context(error::FileRead { path })?;
        Ok(Box::new(file) as Box<dyn Read>)
    } else {
        let response = client
            .get(url.clone())
            .send()
            .context(error::Request { url: url.clone() })?;
        ensure!(
            !response.status().is_client_error() && !response.status().is_server_error(),
            error::ResponseStatus {
                code: response.status(),
                url
            }
        );
        Ok(Box::new(response))
    }
}

pub(crate) fn fetch_max_size(
    client: &Client,
    url: Url,
    max_size: u64,
    specifier: &'static str,
) -> Result<impl Read> {
    Ok(MaxSizeAdapter::new(
        fetch(client, url)?,
        specifier,
        max_size,
    ))
}

pub(crate) fn fetch_sha256(
    client: &Client,
    url: Url,
    size: u64,
    specifier: &'static str,
    sha256: &[u8],
) -> Result<impl Read> {
    Ok(DigestAdapter::sha256(
        MaxSizeAdapter::new(fetch(client, url.clone())?, specifier, size),
        sha256,
        url,
    ))
}
