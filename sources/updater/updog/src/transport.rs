use async_trait::async_trait;
use bytes::Bytes;
use futures::StreamExt;
use futures::TryStreamExt;
use futures_core::Stream;
use log::error;
use std::io::{ErrorKind, Read};
use std::pin::Pin;
use std::sync::{Arc, RwLock};
use tokio_util::compat::FuturesAsyncReadCompatExt;
use tokio_util::io::SyncIoBridge;
use tough::{HttpTransport, Transport, TransportError};
use url::Url;

/// A shared pointer to a list of query params that the transport will add to HTTP calls.
#[derive(Debug, Clone, Default)]
pub(crate) struct QueryParams(Arc<RwLock<Vec<(String, String)>>>);

/// A `tough` `Transport` that allows us to add query parameters to HTTP calls.
#[derive(Debug, Clone)]
#[allow(clippy::module_name_repetitions)]
pub(crate) struct HttpQueryTransport {
    pub inner: HttpTransport,
    parameters: QueryParams,
}

impl QueryParams {
    pub(crate) fn add_params_to_url(&self, mut url: Url) -> Url {
        let mut params = match self.0.write() {
            Err(e) => {
                // a thread died while holding a lock to the params. unlikely to occur.
                error!("unable to add query params to HTTP call: {}", e);
                return url;
            }
            Ok(lock_result) => lock_result,
        };
        params.sort_by(|(a, _), (b, _)| a.cmp(b));
        url.query_pairs_mut().extend_pairs(params.iter());
        url
    }

    pub(crate) fn add<S1, S2>(&self, key: S1, val: S2)
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        let mut params = match self.0.write() {
            Err(e) => {
                // a thread died while holding a lock to the params. unlikely to occur.
                error!(
                    "unable to add query param '{}={}': {}",
                    key.into(),
                    val.into(),
                    e
                );
                return;
            }
            Ok(lock_result) => lock_result,
        };
        params.push((key.into(), val.into()));
    }
}

impl HttpQueryTransport {
    pub fn new() -> Self {
        Self {
            inner: HttpTransport::default(),
            parameters: QueryParams::default(),
        }
    }

    /// Obtain a shared pointer to the query params for this transport.
    pub fn query_params(&self) -> QueryParams {
        QueryParams(Arc::clone(&self.parameters.0))
    }
}

pub(crate) type TransportStream = Pin<Box<dyn Stream<Item = Result<Bytes, TransportError>> + Send>>;

#[async_trait]
impl Transport for HttpQueryTransport {
    /// Send a GET request to the URL. The returned `TransportStream` will retry as necessary per
    /// the `ClientSettings`.
    async fn fetch(&self, url: Url) -> Result<TransportStream, TransportError> {
        self.inner
            .fetch(self.parameters.add_params_to_url(url))
            .await
    }
}

pub(crate) fn reader_from_stream<S>(stream: S) -> impl Read
where
    S: Stream<Item = Result<Bytes, tough::error::Error>> + Send + Unpin,
{
    let mapped_err = stream.map(|next| next.map_err(|e| std::io::Error::new(ErrorKind::Other, e)));
    let async_read = mapped_err.into_async_read().compat();
    SyncIoBridge::new(async_read)
}
