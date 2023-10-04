// Original Copyright 2017 Johann Tuffe. Licensed under the MIT License.
// Modifications Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.

use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

use tokio_rustls::client::TlsStream as RustlsStream;

use hyper::client::connect::{Connected, Connection};

pub type TlsStream<R> = RustlsStream<R>;

/// A Proxy Stream wrapper
pub enum ProxyStream<R> {
    NoProxy(R),
    Regular(R),
    Secured(Box<TlsStream<R>>),
}

macro_rules! match_fn_pinned {
    ($self:expr, $fn:ident, $ctx:expr, $buf:expr) => {
        match $self.get_mut() {
            ProxyStream::NoProxy(s) => Pin::new(s).$fn($ctx, $buf),
            ProxyStream::Regular(s) => Pin::new(s).$fn($ctx, $buf),
            ProxyStream::Secured(s) => Pin::new(s).$fn($ctx, $buf),
        }
    };

    ($self:expr, $fn:ident, $ctx:expr) => {
        match $self.get_mut() {
            ProxyStream::NoProxy(s) => Pin::new(s).$fn($ctx),
            ProxyStream::Regular(s) => Pin::new(s).$fn($ctx),
            ProxyStream::Secured(s) => Pin::new(s).$fn($ctx),
        }
    };
}

impl<R: AsyncRead + AsyncWrite + Unpin> AsyncRead for ProxyStream<R> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        match_fn_pinned!(self, poll_read, cx, buf)
    }
}

impl<R: AsyncRead + AsyncWrite + Unpin> AsyncWrite for ProxyStream<R> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        match_fn_pinned!(self, poll_write, cx, buf)
    }

    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[io::IoSlice<'_>],
    ) -> Poll<Result<usize, io::Error>> {
        match_fn_pinned!(self, poll_write_vectored, cx, bufs)
    }

    fn is_write_vectored(&self) -> bool {
        match self {
            ProxyStream::NoProxy(s) => s.is_write_vectored(),
            ProxyStream::Regular(s) => s.is_write_vectored(),
            ProxyStream::Secured(s) => s.is_write_vectored(),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match_fn_pinned!(self, poll_flush, cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match_fn_pinned!(self, poll_shutdown, cx)
    }
}

impl<R: AsyncRead + AsyncWrite + Connection + Unpin> Connection for ProxyStream<R> {
    fn connected(&self) -> Connected {
        match self {
            ProxyStream::NoProxy(s) => s.connected(),

            ProxyStream::Regular(s) => s.connected().proxy(true),

            ProxyStream::Secured(s) => s.get_ref().0.connected().proxy(true),
        }
    }
}
