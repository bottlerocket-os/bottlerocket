//! The 'stop' module provides helpers for stopping a WebSocket actor and sending a Close message
//! to the client.

use super::WsExec;
use actix::ActorContext;
use actix_web_actors::ws;
use log::info;
use std::fmt::Debug;

/// If the given Option is Some, this evaluates to the inner value.  If it's None, this stops the
/// WebSocket and returns from the current function.
///
/// Pass in the actor context so we're able to call stop, and the desired message and CloseCode to
/// be sent to the client in the case of None.
macro_rules! some_or_stop {
    ($option:expr, $context:expr, $message:expr, $closecode:expr $(,)?) => {
        match $option {
            Some(inner) => inner,
            None => {
                stop($context, $message, $closecode);
                return;
            }
        }
    };
}
pub(crate) use some_or_stop;

/// If the given Result is Ok, this evaluates to the inner value.  If it's Err, this stops the
/// WebSocket and returns from the current function.
///
/// Pass in the actor context so we're able to call stop, and the desired message and CloseCode to
/// be sent to the client in the case of Err.  The Error inside the Err is appended to your
/// message.
macro_rules! ok_or_stop {
    ($result:expr, $context:expr, $message:expr, $closecode:expr $(,)?) => {
        match $result {
            Ok(inner) => inner,
            Err(e) => {
                stop($context, Some(format!("{}: {}", $message, e)), $closecode);
                return;
            }
        }
    };
}
pub(crate) use ok_or_stop;

/// Sends the given Close message (if any) and CloseCode to the client and stops the WebSocket.
pub(crate) fn stop<S>(
    ctx: &mut ws::WebsocketContext<WsExec>,
    message: Option<S>,
    closecode: ws::CloseCode,
) where
    S: Into<String> + Debug,
{
    info!(
        "Closing exec connection{}",
        if let Some(message) = &message {
            format!("; message: {:?}", message)
        } else {
            "".to_string()
        }
    );
    ctx.close(Some(ws::CloseReason {
        code: closecode,
        description: message.map(|s| s.into()),
    }));
    ctx.stop();
}
