//! A Proxy Connector crate for Hyper based applications

// Original Copyright 2017 Johann Tuffe. Licensed under the MIT License.
// Modifications Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.

mod stream;
mod tunnel;
use futures_util::future::TryFutureExt;
use headers::{authorization::Credentials, Authorization, HeaderMapExt, ProxyAuthorization};
use http::header::HeaderMap;
use hyper::{service::Service, Uri};
use std::{fmt, io, sync::Arc};
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

pub use stream::ProxyStream;
use tokio::io::{AsyncRead, AsyncWrite};

use hyper_rustls::ConfigBuilderExt;
use tokio_rustls::rustls::{ClientConfig, ServerName};
use tokio_rustls::TlsConnector;

pub(crate) type BoxError = Box<dyn std::error::Error + Send + Sync>;

/// The Intercept enum to filter connections
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Intercept {
    /// Only https connections will go through proxy
    Https,
    /// No connection will go through this proxy
    None,
    /// A custom intercept
    Custom(Custom),
}

/// A trait for matching between Destination and Uri
pub trait Dst {
    /// Returns the connection scheme, e.g. "http" or "https"
    fn scheme(&self) -> Option<&str>;
    /// Returns the host of the connection
    fn host(&self) -> Option<&str>;
    /// Returns the port for the connection
    fn port(&self) -> Option<u16>;
}

impl Dst for Uri {
    fn scheme(&self) -> Option<&str> {
        self.scheme_str()
    }

    fn host(&self) -> Option<&str> {
        self.host()
    }

    fn port(&self) -> Option<u16> {
        self.port_u16()
    }
}

#[inline]
pub(crate) fn io_err<E: Into<Box<dyn std::error::Error + Send + Sync>>>(e: E) -> io::Error {
    io::Error::new(io::ErrorKind::Other, e)
}

/// A Custom struct to proxy custom uris
#[derive(Clone)]
#[allow(clippy::type_complexity)]
pub struct Custom(Arc<dyn Fn(Option<&str>, Option<&str>, Option<u16>) -> bool + Send + Sync>);

impl fmt::Debug for Custom {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "_")
    }
}

impl<F: Fn(Option<&str>, Option<&str>, Option<u16>) -> bool + Send + Sync + 'static> From<F>
    for Custom
{
    fn from(f: F) -> Custom {
        Custom(Arc::new(f))
    }
}

impl Intercept {
    /// A function to check if given `Uri` is proxied
    pub fn matches<D: Dst>(&self, uri: &D) -> bool {
        match (self, uri.scheme()) {
            (&Intercept::Https, Some("https")) => true,
            (&Intercept::Custom(Custom(ref f)), _) => f(uri.scheme(), uri.host(), uri.port()),
            _ => false,
        }
    }
}

impl<F: Fn(Option<&str>, Option<&str>, Option<u16>) -> bool + Send + Sync + 'static> From<F>
    for Intercept
{
    fn from(f: F) -> Intercept {
        Intercept::Custom(f.into())
    }
}

/// A Proxy struct
#[derive(Clone, Debug)]
pub struct Proxy {
    intercept: Intercept,
    force_connect: bool,
    headers: HeaderMap,
    uri: Uri,
}

impl Proxy {
    /// Create a new `Proxy`
    pub fn new<I: Into<Intercept>>(intercept: I, uri: Uri) -> Proxy {
        Proxy {
            intercept: intercept.into(),
            uri,
            headers: HeaderMap::new(),
            force_connect: false,
        }
    }

    /// Set `Proxy` authorization
    pub fn set_authorization<C: Credentials + Clone>(&mut self, credentials: Authorization<C>) {
        // In pluto, we use custom intercept for HTTPS traffic we might proxy based on the no proxy specification.
        match self.intercept {
            Intercept::Custom(_) | Intercept::Https => {
                self.headers.typed_insert(ProxyAuthorization(credentials.0));
            }
            _ => {}
        }
    }
}

/// A wrapper around `Proxy`s with a connector.
#[derive(Clone)]
pub struct ProxyConnector<C> {
    proxies: Vec<Proxy>,
    connector: C,
    tls: Option<TlsConnector>,
}

impl<C: fmt::Debug> fmt::Debug for ProxyConnector<C> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(
            f,
            "ProxyConnector {}{{ proxies: {:?}, connector: {:?} }}",
            if self.tls.is_some() {
                ""
            } else {
                "(unsecured)"
            },
            self.proxies,
            self.connector
        )
    }
}

impl<C> ProxyConnector<C> {
    /// Create a new secured Proxies
    pub fn new(connector: C) -> Result<Self, io::Error> {
        let config = ClientConfig::builder()
            .with_safe_defaults()
            .with_native_roots()
            .with_no_client_auth();

        let cfg = Arc::new(config);
        let tls = TlsConnector::from(cfg);

        Ok(ProxyConnector {
            proxies: Vec::new(),
            connector,
            tls: Some(tls),
        })
    }

    /// Create a proxy connector and attach a particular proxy
    pub fn from_proxy(connector: C, proxy: Proxy) -> Result<Self, io::Error> {
        let mut c = ProxyConnector::new(connector)?;
        c.proxies.push(proxy);
        Ok(c)
    }

    fn match_proxy<D: Dst>(&self, uri: &D) -> Option<&Proxy> {
        self.proxies.iter().find(|p| p.intercept.matches(uri))
    }
}

macro_rules! mtry {
    ($e:expr) => {
        match $e {
            Ok(v) => v,
            Err(e) => break Err(e.into()),
        }
    };
}

impl<C> Service<Uri> for ProxyConnector<C>
where
    C: Service<Uri>,
    C::Response: AsyncRead + AsyncWrite + Send + Unpin + 'static,
    C::Future: Send + 'static,
    C::Error: Into<BoxError>,
{
    type Response = ProxyStream<C::Response>;
    type Error = io::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        match self.connector.poll_ready(cx) {
            Poll::Ready(Ok(())) => Poll::Ready(Ok(())),
            Poll::Ready(Err(e)) => Poll::Ready(Err(io_err(e.into()))),
            Poll::Pending => Poll::Pending,
        }
    }

    fn call(&mut self, uri: Uri) -> Self::Future {
        if let (Some(p), Some(host)) = (self.match_proxy(&uri), uri.host()) {
            if uri.scheme() == Some(&http::uri::Scheme::HTTPS) || p.force_connect {
                let host = host.to_owned();
                let port =
                    uri.port_u16()
                        .unwrap_or(if uri.scheme() == Some(&http::uri::Scheme::HTTP) {
                            80
                        } else {
                            443
                        });
                let tunnel = tunnel::new(&host, port, &p.headers);
                let connection =
                    proxy_dst(&uri, &p.uri).map(|proxy_url| self.connector.call(proxy_url));
                let tls = if uri.scheme() == Some(&http::uri::Scheme::HTTPS) {
                    self.tls.clone()
                } else {
                    None
                };

                Box::pin(async move {
                    #[allow(clippy::never_loop)]
                    loop {
                        // this hack will gone once `try_blocks` will eventually stabilized
                        let proxy_stream = mtry!(mtry!(connection).await.map_err(io_err));
                        let tunnel_stream = mtry!(tunnel.with_stream(proxy_stream).await);

                        break match tls {
                            Some(tls) => {
                                let server_name: ServerName =
                                    mtry!(host.as_str().try_into().map_err(io_err));
                                let secure_stream = mtry!(tls
                                    .connect(server_name, tunnel_stream)
                                    .await
                                    .map_err(io_err));

                                Ok(ProxyStream::Secured(Box::new(secure_stream)))
                            }

                            None => Ok(ProxyStream::Regular(tunnel_stream)),
                        };
                    }
                })
            } else {
                match proxy_dst(&uri, &p.uri) {
                    Ok(proxy_uri) => Box::pin(
                        self.connector
                            .call(proxy_uri)
                            .map_ok(ProxyStream::Regular)
                            .map_err(|err| io_err(err.into())),
                    ),
                    Err(err) => Box::pin(futures_util::future::err(io_err(err))),
                }
            }
        } else {
            Box::pin(
                self.connector
                    .call(uri)
                    .map_ok(ProxyStream::NoProxy)
                    .map_err(|err| io_err(err.into())),
            )
        }
    }
}

fn proxy_dst(dst: &Uri, proxy: &Uri) -> io::Result<Uri> {
    Uri::builder()
        .scheme(
            proxy
                .scheme_str()
                .ok_or_else(|| io_err(format!("proxy uri missing scheme: {}", proxy)))?,
        )
        .authority(
            proxy
                .authority()
                .ok_or_else(|| io_err(format!("proxy uri missing host: {}", proxy)))?
                .clone(),
        )
        .path_and_query(dst.path_and_query().unwrap().clone())
        .build()
        .map_err(|err| io_err(format!("other error: {}", err)))
}
