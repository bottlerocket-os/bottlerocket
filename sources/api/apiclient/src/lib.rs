#![deny(rust_2018_idioms)]

//! The apiclient library provides simple methods to query an HTTP API over a Unix-domain socket.
//!
//! The `raw_request` method takes care of the basics of making an HTTP request on a Unix-domain
//! socket, and requires you to specify the socket path, the URI (including query string), the
//! HTTP method, and any request body data.
//!
//! In the future, we intend to add methods that understand the Bottlerocket API and help more with common
//! types of requests.

// Think "reqwest" but for Unix-domain sockets.  Would be nice to use the simpler reqwest instead
// of hyper, but it lacks Unix-domain socket support:
// https://github.com/seanmonstar/reqwest/issues/39

use hyper::{body, header, Body, Client, Request};
use hyper_unix_connector::{UnixClient, Uri};
use snafu::{ensure, ResultExt};
use std::path::Path;

mod error {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility = "pub(super)")]
    pub enum Error {
        #[snafu(display("Failed to build request: {}", source))]
        RequestSetup { source: http::Error },

        #[snafu(display("Failed to send request: {}", source))]
        RequestSend { source: hyper::Error },

        #[snafu(display("Status {} when {}ing {}: {}", code.as_str(), method, uri, body))]
        ResponseStatus {
            method: String,
            code: http::StatusCode,
            uri: http::uri::Uri,
            body: String,
        },

        #[snafu(display("Failed to read body of response: {}", source))]
        ResponseBodyRead { source: hyper::Error },

        #[snafu(display("Response was not UTF-8: {}", source))]
        NonUtf8Response { source: std::string::FromUtf8Error },
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
pub async fn raw_request<P, S1, S2>(
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
    let method = method.as_ref();

    // We talk over a local Unix-domain socket to the server.
    let client = Client::builder().build::<_, ::hyper::Body>(UnixClient);
    let uri: hyper::Uri = Uri::new(socket_path, uri.as_ref()).into();

    // Build request.
    let request_data = if let Some(data) = data {
        Body::from(data)
    } else {
        Body::empty()
    };
    let request = Request::builder()
        .method(method)
        .uri(&uri)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(request_data))
        .context(error::RequestSetup)?;

    // Send request.
    let res = client.request(request).await.context(error::RequestSend)?;
    let status = res.status();

    // Read streaming response body into a string.
    let body_bytes = body::to_bytes(res.into_body())
        .await
        .context(error::ResponseBodyRead)?;
    let body = String::from_utf8(body_bytes.to_vec()).context(error::NonUtf8Response)?;

    // Error if the response status is in not in the 2xx range.
    ensure!(
        status.is_success(),
        error::ResponseStatus {
            method,
            code: status,
            uri,
            body,
        }
    );

    Ok((status, body))
}
