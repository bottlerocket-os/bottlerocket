//! The apiclient library provides simple, synchronous methods to query an HTTP API over a
//! Unix-domain socket.
//!
//! The `raw_request` method takes care of the basics of making an HTTP request on a Unix-domain
//! socket, and requires you to specify the socket path, the URI (including query string), the
//! HTTP method, and any request body data.
//!
//! In the future, we intend to add methods that understand the Thar API and help more with common
//! types of requests.

// Think "reqwest" but for Unix-domain sockets.  Would be nice to use the simpler reqwest instead
// of hyper, but it lacks Unix-domain socket support:
// https://github.com/seanmonstar/reqwest/issues/39

use futures::Future;
use hyper::rt::Stream;
use hyper::{header, Body, Client, Request};
use hyperlocal::{UnixConnector, Uri};
use snafu::ResultExt;
use std::path::Path;
use tokio::runtime::Runtime;

mod error {
    use snafu::Snafu;
    use std::io;

    #[derive(Debug, Snafu)]
    #[snafu(visibility = "pub(super)")]
    pub enum Error {
        #[snafu(display("Failed to initialize client: {}", source))]
        ClientSetup { source: io::Error },

        #[snafu(display("Failed to build request: {}", source))]
        RequestSetup { source: http::Error },

        #[snafu(display("Failed to send request: {}", source))]
        RequestSend { source: hyper::Error },

        #[snafu(display("Failed to read body of response: {}", source))]
        ResponseBodyRead { source: hyper::Error },

        #[snafu(display("Response was not UTF-8: {}", source))]
        NonUtf8Response { source: std::str::Utf8Error },
    }
}
pub use error::Error;
pub type Result<T> = std::result::Result<T, error::Error>;

/// Makes an HTTP request to a Unix-domain socket.
///
/// The socket is specified as a path, for example "/tmp/api.sock".
/// The URI on that server is specified as a string, for example "/settings".
/// The HTTP method is also specified as a string, for example "GET".
///
/// For read-only methods like GET, `data` should be None, otherwise you can use Some(string) to
/// specify the body of the request.
///
/// If we were able to talk to the server, returns an Ok value with the status code of the response
/// as an http::StatusCode, and the response body as a String.  (Binary responses are not supported
/// and will return an error.)  You should check the status code if you want to consider 4xx/5xx
/// responses as an error; `StatusCode` has various methods to help check.
///
/// If we failed to talk to the server, returns Err.
pub fn raw_request<P, S1, S2>(
    socket_path: P,
    uri: S1,
    method: S2,
    data: Option<String>,
) -> Result<(http::StatusCode, String)>
where
    P: AsRef<Path>,
    S1: AsRef<str>,
    S2: AsRef<str>,
{
    let request_data = if let Some(data) = data {
        Body::from(data)
    } else {
        Body::empty()
    };
    let uri: hyper::Uri = Uri::new(socket_path, uri.as_ref()).into();

    let mut runtime = Runtime::new().context(error::ClientSetup)?;

    let client = Client::builder().build::<_, ::hyper::Body>(UnixConnector::new());

    let request = Request::builder()
        .method(method.as_ref())
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(request_data))
        .context(error::RequestSetup)?;

    // `block_on` is what waits on the asynchronous response future and returns a real response;
    // it's the simplest way to switch to a synchronous mode.
    //
    // hyper's "into_parts" method splits the response into two parts: (1) the head, which we have
    // immediately, and (2) the body, which may be streamed back in chunks.  Therefore, the second
    // return value here is another future.
    let (head, body_future) = runtime
        .block_on(client.request(request).map(|res| res.into_parts()))
        .context(error::RequestSend)?;

    // Wait on the second future (the streaming body) and concatenate all the pieces together so we
    // have a single response body.  We make sure each piece is a string, as we go; we assume that
    // we're not handling binary data.
    let body_stream = body_future
        .concat2()
        .map(|chunk| std::str::from_utf8(&chunk).map(|s| s.to_string()));
    let body = runtime
        .block_on(body_stream)
        .context(error::ResponseBodyRead)?
        .context(error::NonUtf8Response)?;

    Ok((head.status, body))
}
