//! The 'exec' module lets clients run a command in another container.  We spawn the requested
//! command through containerd and use a WebSocket for communication with the client.  Process
//! input and output is sent back and forth directly through a binary channel, and control messages
//! are sent through a multiplexed text channel.

// Implementation note: this module manages the WebSocket, which is created for us by Actix, and
// Actix works with 'actors' - individual entities that can send each other different message types
// and take action as desired.  Their message handlers aren't async, so you won't see async/await
// here.  The 'child' module manages the child process, and for simplicity of communication between
// the WebSocket actors and the child, it's not async either - it uses standard threads and
// channels.  See its docs for more detail.

use actix::prelude::{Actor, ActorContext, AsyncContext, Handler, StreamHandler};
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws::{self, Message};
use log::{debug, error, info};
use model::exec::{Capacity, ClientMessage, ServerMessage};
use std::convert::TryFrom;
use std::fmt::Debug;
use std::path::PathBuf;
use std::sync::mpsc::TrySendError;
use std::time::{Duration, Instant};

mod child;
mod stop;
use child::ChildHandles;
use stop::{ok_or_stop, some_or_stop, stop};

/// To guard against stale connections, we send ping and pong messages through the channel
/// regularly as a 'heartbeat'; this is how often we send them.
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(2);
/// If we haven't heard from the client in this much time, we consider it gone and we stop.
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

/// Actix manages the WebSocket for us, but we don't have a way to block (stop receiving) requests
/// at the socket level.  That's called "backpressure" and is important so buffers don't fill up
/// endlessly, and so the client doesn't think we've processed a bunch of data that we don't have
/// room for.
///
/// To alleviate this, we keep track of how many messages of user input we've written to the child
/// process, and regularly update the client about our available capacity.
///
/// This represents the maximum number of messages we want to allow sitting in buffers, as-yet
/// unwritten to the child process, before we fail and stop.
// The number is sort of arbitrary; apiclient uses 4k buffers, so this allows a few megabytes
// outstanding; if someone changes apiclient or writes an aggressive client, Actix has a max 64k
// message size, so we're still somewhat bounded.  (We don't use the Continuation message type
// intended for long-running streams.)  If total number of connections becomes an issue, we'd
// probably have to implement something like a counting semaphore at the controller level.
const MAX_MESSAGES_OUTSTANDING: u64 = 1024;
/// This represents how often we send capacity updates to the client; every X writes.  There's no
/// need to send them on a time interval because there may have been few writes.
// The number is sort of arbitrary.  Lower means more overhead of control messages, higher means
// the client can't read and send messages for longer.  Testing didn't show huge differences in
// performance between 64 and 512.
const CAPACITY_UPDATE_INTERVAL: u64 = 128;

/// Starts the WebSocket, handing control of the message stream to our WsExec actor.
pub(crate) async fn ws_exec(
    r: HttpRequest,
    stream: web::Payload,
    data: web::Data<crate::server::SharedData>,
) -> Result<HttpResponse, Error> {
    info!(
        "Received exec request to {}:{}",
        r.connection_info().host(),
        r.path()
    );

    ws::start(WsExec::new(data.exec_socket_path.clone()), &r, stream)
}

/// WsExec is an actor that represents the WebSocket connection to the client.  All messages to and
/// from the client must pass through WsExec.  For example, the 'child' module holds the Addr
/// (address) of WsExec so it can send us actor messages that we can turn into WebSocket
/// communication.
// If the exec feature sees much use, it may be worthwhile including a generated session ID in this
// struct, and including that in log output to distinguish exec requests.
#[derive(Debug)]
pub(crate) struct WsExec {
    /// This tracks the last time we heard from the client; if it's been too long, we consider the
    /// connection stale and terminate it.
    heartbeat: Instant,

    /// This represents the child process we spawn based on the client's request.  It's an Option
    /// because we don't spawn the process until we get an Initialize message with request details.
    child_handles: Option<ChildHandles>,

    /// This represents the path to the containerd socket that we use to spawn the requested
    /// process in a container namespace.
    exec_socket_path: PathBuf,
}

impl WsExec {
    fn new(exec_socket_path: PathBuf) -> Self {
        Self {
            heartbeat: Instant::now(),
            child_handles: None,
            exec_socket_path,
        }
    }

    /// This starts a task that's responsible for confirming that our connection to the client
    /// isn't stale.  We ping the client regularly so it knows we're alive, and we confirm that the
    /// client has pinged us recently so we know it's alive.
    fn heartbeat(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |actor, ctx| {
            // If we don't hear from the client in a while, consider it stale and terminate.
            if Instant::now().duration_since(actor.heartbeat) > CLIENT_TIMEOUT {
                info!("exec client heartbeat failed, disconnecting");
                ctx.stop();
                return;
            }

            debug!("exec client heartbeat ok, sending ping");
            ctx.ping(b"");
        });
    }
}

impl Actor for WsExec {
    // This tells Actix to give us access to a WebsocketContext in every handler, and the
    // WebsocketContext lets us send messages or stop as needed.
    type Context = ws::WebsocketContext<Self>;

    /// When the actor is first started, we set up long-running processes like the heartbeat, and
    /// send an initial capacity update so the client can start reading and sending input.
    fn started(&mut self, ctx: &mut Self::Context) {
        debug!("Starting heartbeat and sending initial capacity update");
        self.heartbeat(ctx);

        let capacity = Capacity {
            max_messages_outstanding: MAX_MESSAGES_OUTSTANDING,
            messages_written: 0,
        };
        ctx.notify(message::CapacityUpdate(capacity));
    }
}

impl StreamHandler<Result<Message, ws::ProtocolError>> for WsExec {
    /// This handler is run every time we receive a message from the client.  We determine what
    /// type of WebSocket message it was, and if it was a control message (serialized over the Text
    /// channel), we further determine what type of ClientMessage it was.  Process input is sent
    /// directly over the Binary channel.
    fn handle(&mut self, msg: Result<Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            // Respond to Ping with Pong so the client knows we're alive, and record that we've
            // heard from them.
            Ok(Message::Ping(msg)) => {
                debug!("Received ping, updating heartbeat and responding");
                self.heartbeat = Instant::now();
                ctx.pong(&msg);
            }

            // When the client responds to our Ping with a Pong, record that we've heard from them.
            Ok(Message::Pong(_)) => {
                debug!("Received pong, updating heartbeat");
                self.heartbeat = Instant::now();
            }

            // Binary means process input, which we write directly to the child process.
            Ok(Message::Binary(data)) => {
                trace!("Received {} bytes of input from client", data.len());

                // Confirm we have a child, i.e. the client didn't send messages out of order.
                let child_handles = some_or_stop!(
                    &self.child_handles,
                    ctx,
                    Some("process data sent before initialization"),
                    ws::CloseCode::Policy,
                );

                // Confirm that we still have a channel open to write to the child process.  We
                // drop this when the client sends a ContentComplete; they shouldn't send anything
                // after that, but if they do, we can just ignore it.
                if let Some(write_tx) = &child_handles.write_tx {
                    // This is where we check that the client is actually obeying the capacity
                    // updates we're sending them.  The write_tx channel is bounded, and if we fail
                    // to write to it because it's full, we can righteously yell at the client.
                    match write_tx.try_send(data) {
                        // Sent the write request OK.
                        Ok(_unit) => {}

                        // Disconnect the client if they ignore our capacity.  We can't just wait
                        // for capacity because either (1) we'd use unlimited memory, or (2) we'd
                        // block the whole actor, meaning nothing gets done; heartbeats would fail,
                        // output wouldn't get sent, etc.
                        Err(TrySendError::Full(_data)) => {
                            info!("Client not obeying capacity updates, closing connection");
                            let msg = "write buffer full; obey capacity updates".to_string();
                            ctx.close(Some(ws::CloseReason {
                                code: ws::CloseCode::Size,
                                description: Some(msg),
                            }));
                            // Note: we don't ctx.stop() here because the close message wouldn't
                            // get sent; the actor message load from the incoming data delays
                            // sending the close, but stop() acts immediately.  This means we'll
                            // continue to receive client messages until it receives our stop.  Any
                            // more process data will likely hit this spot again and be dropped.
                        }

                        // Disconnected means the write channel is closed, meaning a write to the
                        // child process failed and we can no longer write to it safely; tell the
                        // client to stop.
                        Err(TrySendError::Disconnected(_data)) => {
                            stop(ctx, Some("writing to process failed"), ws::CloseCode::Error);
                        }
                    }
                }
            }

            // A Text message is a multiplexed control message giving us some control information
            // from the client.  We deserialize it to figure out what they want.
            Ok(Message::Text(msg)) => {
                let msg = ok_or_stop!(
                    serde_json::from_str(&msg),
                    ctx,
                    "invalid JSON in client message",
                    ws::CloseCode::Invalid
                );
                match msg {
                    // Initialize should be the first message the client sends, and it tells us
                    // what process they want to run and how.  (It'd be nice to include in the HTTP
                    // request body so we don't worry as much about ordering, but not all clients
                    // support that.)
                    ClientMessage::Initialize(init) => {
                        debug!("Client initialized for target container '{}' and command {:?} with tty: {}",
                               init.target,
                               init.command,
                               init.tty.is_some());
                        // Spawn the process, getting back handles that let us interact with it.
                        let child_handles = ok_or_stop!(
                            ChildHandles::new(init, &self.exec_socket_path, ctx.address()),
                            ctx,
                            "failed to spawn process",
                            ws::CloseCode::Error
                        );
                        self.child_handles = Some(child_handles);
                    }

                    // This means the client is done reading input from the user and we can close
                    // the write channel to the process, closing its stdin.
                    ClientMessage::ContentComplete => {
                        debug!("Received client content complete, dropping write handle");
                        // Confirm we have a child, i.e. the client didn't send messages out of
                        // order.
                        let child_handles = some_or_stop!(
                            &mut self.child_handles,
                            ctx,
                            Some("ContentComplete sent before initialization"),
                            ws::CloseCode::Policy
                        );
                        drop(child_handles.write_tx.take());
                    }

                    // This means the client changed window size, so we should relay that to the
                    // child process so it can update its output as needed.
                    ClientMessage::Winch(size) => {
                        // Unlikely to get here without a child being spawned yet, but if so,
                        // initialization should include window size so this isn't needed.
                        if let Some(child_handles) = self.child_handles.as_mut() {
                            child_handles.set_winsize(size);
                        } else {
                            debug!("Received client winch before child was spawned");
                        }
                    }
                }
            }

            // This means the client is done with us; stop the actor.
            Ok(Message::Close(reason)) => {
                info!("Client closed exec connection with reason: {:?}", reason);
                ctx.close(reason);
                ctx.stop();
            }

            // We don't use Continuation frames; it's easier to deal with individual Text/Binary
            // messages.
            Ok(Message::Continuation(_)) => {
                let msg = "Continuation messages not supported";
                stop(ctx, Some(msg), ws::CloseCode::Unsupported);
            }

            // no-op
            Ok(Message::Nop) => {}

            Err(e) => {
                error!("Stopping after receiving error message: {}", e);
                ctx.stop();
            }
        }
    }

    /// We hit finished() as soon as the client closes the channel or exits, so it's our first
    /// indication that we're done with the child process.  We want to stop it so it doesn't run
    /// forever with no one watching.
    fn finished(&mut self, ctx: &mut Self::Context) {
        info!("exec client disconnected");
        if let Some(child_handles) = &self.child_handles {
            child_handles.stop();
        }

        // Note: stopping the actor prevents the ProcessReturn message being received and the
        // return code being logged, but the client is gone and doesn't care about the return code,
        // and it's probably better not to leave an actor around in case the child can't be killed.
        ctx.stop();
    }
}

/// The 'message' module contains the non-WebSocket messages that our WebSocket actor can handle;
/// they're how our child process code talks to the actor so data can be sent to the client.
mod message {
    /// Represents any output from the child process that should be sent directly to the client.
    #[derive(actix::Message)]
    #[rtype(result = "()")]
    pub(super) struct ProcessOutput {
        pub(super) output: Vec<u8>,
    }

    /// Represents the return code of the child process that should be communicated to the client
    /// while closing the WebSocket.
    #[derive(actix::Message)]
    #[rtype(result = "()")]
    pub(super) struct ProcessReturn {
        pub(super) code: i32,
    }

    /// Represents a capacity update that tells the client how much data we're prepared to receive.
    #[derive(Debug, actix::Message)]
    #[rtype(result = "()")]
    pub(super) struct CapacityUpdate(pub(super) super::Capacity);
}

impl Handler<message::ProcessOutput> for WsExec {
    type Result = ();

    /// Send some process output directly (unencoded) to the client using a Binary message.
    /// Messages are always sent in order.
    fn handle(&mut self, msg: message::ProcessOutput, ctx: &mut Self::Context) -> Self::Result {
        trace!(
            "Sending {} bytes of process output to client",
            msg.output.len()
        );
        ctx.binary(msg.output)
    }
}

impl Handler<message::ProcessReturn> for WsExec {
    type Result = ();

    /// Sends the process return code to the client inside a Close message.
    fn handle(&mut self, msg: message::ProcessReturn, ctx: &mut Self::Context) -> Self::Result {
        info!("exec process returned {}", msg.code);
        // nix deals with i32 (c_int) return codes, but we know they're never negative; really,
        // they're just a u8.  If that assumption breaks for some reason, we don't have a
        // reasonable code to send to the user, so just give a 0.
        let code = u16::try_from(msg.code).unwrap_or(0);
        // We send the process return code in the closing frame's reason message.
        stop(ctx, Some(code.to_string()), ws::CloseCode::Normal);
    }
}

impl Handler<message::CapacityUpdate> for WsExec {
    type Result = ();

    /// Sends a capacity update to the client, multiplexed into a ServerMessage.
    fn handle(&mut self, msg: message::CapacityUpdate, ctx: &mut Self::Context) -> Self::Result {
        debug!(
            "Sending capacity update; {} max outstanding, {} written",
            msg.0.max_messages_outstanding, msg.0.messages_written
        );
        let msg = ok_or_stop!(
            serde_json::to_string(&ServerMessage::Capacity(msg.0)),
            ctx,
            "failed to send capacity update",
            ws::CloseCode::Error,
        );
        ctx.text(msg);
    }
}
