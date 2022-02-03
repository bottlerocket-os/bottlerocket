//! The 'connect' module provides a function for connecting to a WebSocket over a Unix-domain
//! socket, which is a bit more finicky than normal.

use hyper::service::Service;
use hyper_unix_connector::{UnixClient, Uri, UDS};
use log::debug;
use snafu::{ensure, ResultExt};
use std::path::Path;
use tokio_tungstenite::{client_async, tungstenite::http::StatusCode, WebSocketStream};

/// Connects to a WebSocket over the given Unix-domain socket.  'path' is an HTTP request path on
/// the server that allows for WebSocket upgrades, like "/exec".
pub(crate) async fn websocket_connect<P>(socket_path: P, path: &str) -> Result<WebSocketStream<UDS>>
where
    P: AsRef<Path>,
{
    // To talk over a Unix socket, we use hyper-unix-connector, which needs a different type of URI
    // than our WebSocket client, tokio-tungstenite.  This URI can't contain the schema/host.  This
    // initially has to be constructed as a hyper-unix-connector URI so we can use the socket, then
    // transformed into a hyper URI that the client can accept.
    let raw_uri = Uri::new(socket_path.as_ref(), path);
    let uri: hyper::Uri = raw_uri.into();

    debug!(
        "Connecting to {} over {}",
        uri,
        socket_path.as_ref().display()
    );
    // We start with a plain HTTP request over the Unix-domain socket so we can upgrade it to a
    // WebSocket afterward.
    let response = UnixClient.call(uri).await.map_err(|e| {
        // hyper-unix-connector doesn't have its own error type; not worth bringing in 'anyhow'
        error::ConnectSnafu {
            socket: socket_path.as_ref(),
            message: e.to_string(),
        }
        .build()
    })?;

    // Create a request object that tokio-tungstenite understands, pointed at a local WebSocket
    // URI.  This is used to create the WebSocket client.
    let ws_uri = format!("ws://localhost{}", path);
    let ws_request = httparse::Request {
        method: Some("GET"),
        path: Some(&ws_uri),
        version: Some(1), // HTTP/1.1
        headers: &mut [],
    };

    // Now we can use tokio-tungstenite to upgrade the connection to a WebSocket.  We get back a
    // WebSocket stream that we can use to talk to the server, and the HTTP response.
    let (ws_stream, resp) = client_async(ws_request, response)
        .await
        .context(error::UpgradeSnafu)?;

    // We only use the HTTP response to confirm that we switched protocols correctly.
    ensure!(
        resp.status() == StatusCode::SWITCHING_PROTOCOLS,
        error::ProtocolSnafu {
            code: resp.status()
        }
    );

    Ok(ws_stream)
}

pub(crate) mod error {
    use super::StatusCode;
    use snafu::Snafu;
    use std::path::PathBuf;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub enum Error {
        #[snafu(display("Failed to connect to server at {}: {}", socket.display(), message))]
        Connect { socket: PathBuf, message: String },

        #[snafu(display(
            "Server did not upgrade to WebSocket; expected 101 Switching Protocols, got {}",
            code
        ))]
        Protocol { code: StatusCode },

        #[snafu(display("Failed to request upgrade to WebSocket: {}", source))]
        Upgrade {
            source: tokio_tungstenite::tungstenite::Error,
        },
    }
}
pub(crate) use error::Error;
type Result<T> = std::result::Result<T, error::Error>;
