use crate::error::{self, Result};
use crate::io::{DigestAdapter, MaxSizeAdapter};
use crate::transport::Transport;
use snafu::ResultExt;
use std::io::Read;
use url::Url;

pub(crate) fn fetch_max_size<T: Transport>(
    transport: &T,
    url: Url,
    max_size: u64,
    specifier: &'static str,
) -> Result<impl Read> {
    Ok(MaxSizeAdapter::new(
        transport
            .fetch(url.clone())
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
            .context(error::Transport { url })?,
        specifier,
        max_size,
    ))
}

pub(crate) fn fetch_sha256<T: Transport>(
    transport: &T,
    url: Url,
    size: u64,
    specifier: &'static str,
    sha256: &[u8],
) -> Result<impl Read> {
    Ok(DigestAdapter::sha256(
        MaxSizeAdapter::new(
            transport
                .fetch(url.clone())
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
                .context(error::Transport { url: url.clone() })?,
            specifier,
            size,
        ),
        sha256,
        url,
    ))
}
