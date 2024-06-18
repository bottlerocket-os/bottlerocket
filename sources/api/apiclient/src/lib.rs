//! The apiclient library provides high-level methods to interact with the Bottlerocket API.  See
//! the documentation for submodules [`apply`], [`exec`], [`get`], [`reboot`], [`report`], [`set`],
//! and [`update`] for high-level helpers.
//!
//! For more control, and to handle APIs without high-level wrappers, there are also 'raw' methods
//! to query an HTTP API over a Unix-domain socket.
//!
//! The `raw_request` method takes care of the basics of making an HTTP request on a Unix-domain
//! socket, and requires you to specify the socket path, the URI (including query string), the
//! HTTP method, and any request body data.

// Think "reqwest" but for Unix-domain sockets.  Would be nice to use the simpler reqwest instead
// of hyper, but it lacks Unix-domain socket support:
// https://github.com/seanmonstar/reqwest/issues/39

use hyper::{body, header, Body, Client, Request};
use hyper_unix_connector::{UnixClient, Uri};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use snafu::{ensure, ResultExt};
use std::{fmt, fmt::Display, path::Path};

pub mod apply;
pub mod exec;
pub mod get;
pub mod reboot;
pub mod report;
pub mod set;
pub mod update;

mod error {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub enum Error {
        #[snafu(display("Failed to build request: {}", source))]
        RequestSetup { source: http::Error },

        #[snafu(display("Failed to send request: {}", source))]
        RequestSend { source: hyper::Error },

        #[snafu(display("Status {} when {}ing {}: {}", code.as_str(), method, uri, body))]
        ResponseStatus {
            method: String,
            code: http::StatusCode,
            uri: String,
            body: String,
        },

        // This type of error just returns the source.
        #[snafu(display("{}", body))]
        Raw { body: String },

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
    let (status, body) = raw_request_unchecked(&socket_path, &uri, &method, data).await?;
    check_invalid_client_input(body.as_ref(), status, method.as_ref(), uri.as_ref())?;
    Ok((status, body))
}

/// Works exactly like raw_request in making an HTTP request over a Unix-domain socket, but doesn't
/// check that the returned status code represents success.  This can be useful if you have to
/// handle specific error codes, rather than inspecting the Error type of raw_request.
pub async fn raw_request_unchecked<P, S1, S2>(
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
        .body(request_data)
        .context(error::RequestSetupSnafu)?;

    // Send request.
    let res = client
        .request(request)
        .await
        .context(error::RequestSendSnafu)?;
    let status = res.status();

    // Read streaming response body into a string.
    let body_bytes = body::to_bytes(res.into_body())
        .await
        .context(error::ResponseBodyReadSnafu)?;
    let body = String::from_utf8(body_bytes.to_vec()).context(error::NonUtf8ResponseSnafu)?;
    Ok((status, body))
}

/// Generates a random ID, affectionately known as a 'rando'.
pub(crate) fn rando() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect()
}

/// Different Client type errors we expect.
const CLIENT_DESERIALIZATION_MAP_ERROR: &str = "Unable to match your input to the data model.  We may not have enough type information.  Please try the --json input form";
const CLIENT_DESERIALIZATION_JSON_ERROR: &str = "Unable to deserialize input JSON into model";
const SERVER_DESERIALIZATION_JSON_ERROR: &str = "Json deserialize error";
const CLIENT_SERIALIZATION_ERROR: &str = "Unable to serialize data";

#[derive(Debug)]
enum ClientTypeErrors {}

impl ClientTypeErrors {
    fn from(input: &str) -> Option<&str> {
        if input.contains(CLIENT_DESERIALIZATION_JSON_ERROR)
            || input.contains(CLIENT_DESERIALIZATION_MAP_ERROR)
            || input.contains(SERVER_DESERIALIZATION_JSON_ERROR)
            || input.contains(CLIENT_SERIALIZATION_ERROR)
        {
            Some("client_error")
        } else {
            None
        }
    }
}

fn check_invalid_client_input(
    body: &str,
    status: http::StatusCode,
    method: &str,
    uri: &str,
) -> Result<()> {
    match ClientTypeErrors::from(body) {
        Some(_) => ensure!(status.is_success(), error::RawSnafu { body }),
        None => ensure!(
            status.is_success(),
            error::ResponseStatusSnafu {
                method: method.to_string(),
                code: status,
                uri: uri.to_string(),
                body,
            }
        ),
    };
    Ok(())
}

/// Different input types supported by the Settings API.
#[derive(Debug)]
pub enum SettingsInput {
    KeyPair(String),
    Json(String),
}

impl Display for SettingsInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SettingsInput::KeyPair(value) => write!(f, "{}", value),
            SettingsInput::Json(value) => write!(f, "{}", value),
        }
    }
}
