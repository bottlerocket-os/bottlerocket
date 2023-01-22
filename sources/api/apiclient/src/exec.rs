//! The 'exec' module lets you run a command in another container through the apiserver.  A
//! WebSocket is used for communication with the server.  Process input and output is sent back and
//! forth directly through a binary channel, and control messages are sent through a multiplexed
//! text channel.

// Implementation note: the main job of this module is managing communication to and from the
// server through a WebSocket.  This is accomplished mainly with threads and channels - a thread is
// started to manage each particular resource, like input, output, signals, heartbeat, etc.  If it
// needs to send messages to the server, it's given a channel to the server.  If the caller needs to
// hear back from the thread, it's given back a channel.
//
// This behavior is encapsulated in structs.  For example, there's a Heartbeat struct; you create
// it and give it a channel it can use to send to the server, it starts a thread, and you get back
// the struct, which contains a channel that tells you if the heartbeat dies.

use futures::{Future, FutureExt, Stream, StreamExt, TryStream, TryStreamExt};
use futures_channel::{mpsc, oneshot};
use libc::{ioctl, winsize as WinSize, STDOUT_FILENO, TIOCGWINSZ as GetWinSize};
use log::{debug, error, trace, warn};
use model::exec::{ClientMessage, Initialize, ServerMessage, Size};
use retry_read::RetryRead;
use signal_hook::{consts::signal, iterator::Signals};
use snafu::{OptionExt, ResultExt};
use std::ffi::OsString;
use std::io::Read;
use std::os::unix::io::RawFd;
use std::path::Path;
use std::pin::Pin;
use std::process;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, Mutex,
};
use std::thread::{self, sleep};
use std::time::{Duration, Instant};
use tokio::io::AsyncWriteExt;
use tokio_tungstenite::tungstenite::{
    protocol::{frame::coding::CloseCode, CloseFrame, Message},
    Error as WsError,
};

mod connect;
mod terminal;
use connect::websocket_connect;
use terminal::Terminal;

/// To guard against stale connections, we send ping and pong messages through the channel
/// regularly as a 'heartbeat'; this is how often we send them.
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(2);
/// If we haven't heard from the server in this much time, we consider it gone and we stop.
const SERVER_TIMEOUT: Duration = Duration::from_secs(10);

/// This is the main entry point.  We start a connection with the server, request a command be run,
/// set up helper threads to manage communication, and wait for a result.
pub async fn exec<P>(
    socket_path: P,
    command: Vec<OsString>,
    target: String,
    tty: Option<bool>,
) -> Result<()>
where
    P: AsRef<Path>,
{
    // We want to send the user's input to the server untouched; for interactive use cases, this
    // means we need to set the terminal to 'raw mode' so that certain keystrokes aren't
    // interpreted and turned into signals, etc.  The Terminal type manages that for us, and resets
    // the terminal when it's dropped later.  We set this up first so that we don't unnecessarily
    // talk to the server if it fails.
    let terminal = Terminal::new(tty).context(error::TerminalSnafu)?;

    // Connect to the server over the Unix-domain socket and upgrade to a WebSocket.
    let ws_stream = websocket_connect(socket_path, "/exec")
        .await
        .context(error::ConnectSnafu)?;

    // We're going to split the stream into write and read halves so we can manage them with
    // separate threads, which simplifies the use of blocking calls, not requiring a totally new
    // async infrastructure.
    let (write, read) = ws_stream.split();

    // We make a multi-producer channel that forwards anything it receives to the WebSocket; we can
    // share the transmission end of the channel with any number of threads that need to send
    // messages to the server.
    let (ws_tx, ws_rx) = mpsc::unbounded();
    let forward_to_ws = ws_rx.map(Ok).forward(write);
    debug!("Spawning task to write to WebSocket");
    tokio::spawn(forward_to_ws);

    // The first thing we want to send is an initialize message that tells the server what program
    // we want to run, what container to run it in, and whether we want a TTY.  (It's important
    // not to send other types of messages first or the server won't have a process to act on and
    // will reject us.  It'd be nice to send initialization parameters in the HTTP request body,
    // but not all WebSocket clients support it.)
    debug!(
        "Sending initialize request for target '{}' with tty: {} and command: {:?}",
        target,
        terminal.tty().is_some(),
        command
    );
    let init = Initialize {
        command,
        target,
        tty: terminal.tty().clone(),
    };
    // Control messages go to the server in a text channel, so we serialize to JSON before sending.
    let msg =
        serde_json::to_string(&ClientMessage::Initialize(init)).context(error::SerializeSnafu)?;
    ws_tx
        .unbounded_send(Message::Text(msg))
        .context(error::SendMessageSnafu {
            kind: "initialization",
        })?;

    // Now that the server knows what we want, we set up helper threads to manage communication.
    // First, a heartbeat type that regularly pings the server and keeps track of responses.
    let mut heartbeat = Heartbeat::new(ws_tx.clone());
    // Next, a type that watches for signals to the local process, and either forwards them to the
    // server (e.g. if you change your window size) or ends communication (e.g. for SIGTERM).
    let mut signal_handler = HandleSignals::new(ws_tx.clone())?;

    // We don't want to overload the server with our process input.  It sends us capacity updates
    // to let us know how many more messages we can send before we should wait.  We keep track of
    // that capacity in an AtomicCapacity that we can share across threads - the one where we
    // receive messages from the server, and the one that's reading input to send to the server.
    let capacity = Arc::new(AtomicCapacity::default());
    let capacity_reader = Arc::clone(&capacity);

    // Start a thread that reads input from the user and sends it across the WebSocket, waiting for
    // capacity between reads if necessary.
    let mut read_from_user =
        ReadFromUser::new(ws_tx.clone(), capacity_reader, terminal.tty().is_some());
    // Start a future that reads the stream of messages from the server.
    let mut read_from_server = ReadFromServer::new(read, heartbeat.setter, capacity);

    // We're all set up!  Wait for something that indicates we're done.
    debug!("Waiting for completion: server, signal, heartbeat, or read error");
    // Store the signal number, if that's why we stop.
    let mut signal_ret = None;
    // We drop Terminal early in each branch so we can print results cleanly.
    tokio::select! {
        // This is the normal case; the server finishes running the program.
        res = &mut read_from_server.future => {
            drop(terminal);
            debug!("Server read completed");
            // If our ReadFromServer future hit an error, log it, except for the special case of a
            // Close error, which is just an empty marker that we're done.
            if let Err(e) = res {
                let msg = e.to_string();
                if !msg.is_empty() {
                    error!("{}", e);
                }
            }
        }

        // Stop if we fail to read input.
        // Match against Ok(err) because the Err case means the other end of the channel was
        // dropped; that would imply the ReadFromUser thread was dropped, but that doesn't mean the
        // process is done, just that our input is done.
        Ok(err) = &mut read_from_user.error_rx => {
            drop(terminal);
            Err(err)?;
        }

        // Stop if we receive a terminal signal.
        signal = &mut signal_handler.signal_rx => {
            drop(terminal);
            debug!("Received signal: {:?}", signal);
            signal_ret = Some(signal);
        }

        // Stop if the server heartbeat dies.
        _ = &mut heartbeat.finished_rx => {
            drop(terminal);
            warn!("Server heartbeat died");
        }
    }

    // Determine how to exit based on the information we got back from the server, or from a
    // local signal.
    if let Some(Some(ret)) = read_from_server.ret_rx.next().now_or_never() {
        match ret.code {
            // The connection is closing normally, we expect the process exit code in the reason message.
            CloseCode::Normal => {
                if !ret.reason.is_empty() {
                    // This is the normal case where the server gives us the exit code of the process.
                    if let Ok(exit_code) = ret.reason.parse::<u16>() {
                        process::exit(i32::from(exit_code))
                    }
                }
                // If there is no exit code in the reason message, we assume the worst and exit 1.
                warn!("Connection close reason: {}", ret.reason);
                process::exit(1)
            }
            // We don't expect any other CloseCode in normal operation.  The server will send
            // specific CloseCodes if the client disobeyed protocol, but we obey.  The server can
            // also send a generic Error if it's unhealthy.
            _ => {
                if !ret.reason.is_empty() {
                    warn!("Connection close reason: {}", ret.reason);
                }
                process::exit(1)
            }
        }
    } else if let Some(Ok(signal)) = signal_ret {
        // Use shell-style return codes for signals.
        process::exit(128 + signal);
    } else {
        warn!("Didn't receive a return code or signal; unsure what happened");
        process::exit(1)
    }
}

/// ReadFromServer is responsible for handling WebSocket messages received from the server.
struct ReadFromServer {
    /// Represents the task that handles the stream of server messages; when it completes, either
    /// the server has closed the connection or we've hit an error.
    future: Pin<Box<dyn Future<Output = Result<()>>>>,
    /// If the server sends us a reason for closing the connection (which normally includes the
    /// return code of the command) we'll forward it on this channel.
    ret_rx: mpsc::UnboundedReceiver<CloseFrame<'static>>,
}

impl ReadFromServer {
    /// Parameters:
    /// * read: The stream of messages from the server.
    ///
    /// * heartbeat_setter: An atomic handle to a timestamp; this will be updated whenever we
    /// receive a ping or pong from the server so we can make sure the connection isn't stale.
    ///
    /// * capacity: When the server sends a capacity update, we update this AtomicCapacity, so we
    /// can make sure we're not sending (or even reading) data the server can't handle.
    fn new(
        read: impl Stream<Item = std::result::Result<Message, WsError>> + 'static,
        heartbeat_setter: Arc<Mutex<Instant>>,
        capacity: Arc<AtomicCapacity>,
    ) -> Self {
        // Create a channel we use to tell the caller if we get a return value from the server.
        let (ret_tx, ret_rx) = mpsc::unbounded();

        let future = Self::read_from_server(read, heartbeat_setter, ret_tx, capacity);

        Self { future, ret_rx }
    }

    fn read_from_server(
        read: impl TryStream<Ok = Message, Error = WsError> + 'static,
        heartbeat_setter: Arc<Mutex<Instant>>,
        ret_tx: mpsc::UnboundedSender<CloseFrame<'static>>,
        capacity: Arc<AtomicCapacity>,
    ) -> Pin<Box<dyn Future<Output = Result<()>>>> {
        // Turn tungstenite errors into our own error type.
        read.err_into::<error::Error>()
            // Process each message from the server, stopping on Close or error.
            .try_for_each(move |ws_msg| {
                // For ownership reasons, make copies of the atomic handles that can be passed into
                // the async closure.
                let heartbeat_setter = heartbeat_setter.clone();
                let capacity = capacity.clone();
                let ret_tx = ret_tx.clone();

                async move {
                    match ws_msg {
                        // Binary messages represent process output, not encoded in any way.  Write
                        // it to stdout.
                        Message::Binary(data) => {
                            trace!("Received {} bytes of output from server", data.len());
                            let mut stdout = tokio::io::stdout();
                            stdout.write_all(&data).await.context(error::WriteOutputSnafu)?;
                            // May not be a full line of output, so flush any bytes we got.  Failure here
                            // isn't worthy of stopping the whole process.
                            let _ = stdout.flush().await;
                        }
                        // tokio-tungstenite replies to ping with pong; we just update our heartbeat.
                        Message::Ping(_) | Message::Pong(_) => {
                            // If we fail to get the mutex, the heartbeat thread has panicked, which means
                            // we'll no longer send pings to the server, and it'll disconnect us at some
                            // point.  Might as well try to finish our processing in the meantime.
                            if let Ok(mut hb) = heartbeat_setter.lock() {
                                trace!("Got ping/pong from server, updating heartbeat");
                                *hb = Instant::now();
                            }
                        }
                        // The server requested we close the connection, so we stop processing.
                        // Usually it includes the return code of the requested process.
                        Message::Close(c) => {
                            if let Some(ret) = c {
                                // If we fail to send the return code, there's nothing we can do to rectify
                                // the situation, and this is a Close so we definitely want to return below
                                // anyway.
                                let _ = ret_tx.unbounded_send(ret);
                            }
                            return error::CloseSnafu.fail();
                        }
                        // Text messages represent encoded control messages from the server.
                        Message::Text(raw_msg) => {
                            let server_message =
                                serde_json::from_str(&raw_msg).context(error::DeserializeSnafu)?;
                            match server_message {
                                // Capacity messages tell us how many messages the server is
                                // willing to receive before it rejects us.
                                ServerMessage::Capacity(new) => {
                                    debug!(
                                        "Received capacity update from server: {} max outstanding, {} written",
                                        new.max_messages_outstanding,
                                        new.messages_written
                                    );
                                    capacity
                                        .max_messages_outstanding
                                        .store(new.max_messages_outstanding, Ordering::SeqCst);
                                    capacity
                                        .messages_written
                                        .store(new.messages_written, Ordering::SeqCst);
                                }
                            }
                        }
                    }
                    Ok(())
                }
            })
            // This puts the future in a Pin<Box>; we use Box so we don't have to name the exact
            // future type, and Pin is required for tokio to select! it.
            .boxed_local()
    }
}

/// ReadFromUser is responsible for reading user input from stdin and sending it to the given
/// channel so it can be forwarded to the server.
struct ReadFromUser {
    /// If we fail to read input, we'll return the error on this channel so the client can be
    /// stopped.
    error_rx: oneshot::Receiver<Error>,
}

impl ReadFromUser {
    /// Parameters:
    /// * stdin_tx: The channel to which we should send messages containing user input.
    ///
    /// * capacity_reader: We'll only read input when the server has capacity, according to this
    /// parameter, so that we don't unnecessarily fill buffers or overwhelm the server.
    ///
    /// * is_tty: whether input is coming from a TTY; think of it as whether the command is
    /// interactive.  If so, we read a byte at a time and send it immediately to the server so that
    /// things like tab completion work.
    fn new(
        stdin_tx: mpsc::UnboundedSender<Message>,
        capacity_reader: Arc<AtomicCapacity>,
        is_tty: bool,
    ) -> Self {
        // Create a channel we use to tell the caller if reading fails.
        let (error_tx, error_rx) = oneshot::channel();

        debug!("Spawning thread to read from user");
        let stdin_fn = if is_tty {
            Self::read_stdin_tty
        } else {
            Self::read_stdin
        };
        thread::spawn(move || {
            if let Err(e) = stdin_fn(stdin_tx, capacity_reader) {
                let _ = error_tx.send(e);
            }
        });

        Self { error_rx }
    }

    /// Read from stdin with the lowest possible latency, sending each byte to the server.
    fn read_stdin_tty(
        tx: mpsc::UnboundedSender<Message>,
        capacity: Arc<AtomicCapacity>,
    ) -> Result<()> {
        let mut stdin = std::io::stdin();
        // Keep track of the number of messages we've read.  We compare this to the number of
        // messages the server has written, as received in its regular capacity update messages, so
        // that we don't overwhelm the server.
        let mut messages_read = 0u64;

        loop {
            // Wait for server to have capacity for writes before reading; don't give "false hope" to
            // whatever's writing to our stdin that it'll be read until there's room for it.
            // (Note: we're unlikely to hit this interactively, which is the primary use for TTY.)
            Self::wait_for_capacity(messages_read, &capacity)?;

            // Read a byte at a time.
            let mut buf = [0; 1];
            match stdin.read_exact(&mut buf) {
                Ok(()) => {
                    messages_read += 1;
                    // Send the data to the server in a Binary message without encoding.
                    tx.unbounded_send(Message::Binary(Vec::from(buf)))
                        .context(error::SendMessageSnafu { kind: "user input" })?;
                }
                // We don't normally get Err, since the server will close connection first, but for
                // completeness...
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::UnexpectedEof {
                        debug!("Finished reading input");
                        // If we can, send a ContentComplete message to the server so we can exit more
                        // cleanly, but either way we're done.  (Again, we shouldn't get here.)
                        match serde_json::to_string(&ClientMessage::ContentComplete) {
                            Ok(msg) => {
                                if let Err(e) = tx.unbounded_send(Message::Text(msg)) {
                                    warn!("Unable to send ContentComplete to server, may hang if process doesn't exit: {}", e);
                                }
                            }
                            Err(e) => warn!("Unable to serialize ContentComplete, may hang if process doesn't exit: {}", e),
                        }
                        return Ok(());
                    } else {
                        // Any error other than EOF is a real read error.
                        Err(e).context(error::ReadFromUserSnafu)?
                    }
                }
            }
        }
    }

    /// Read from stdin in bulk, sending larger batches of data at a time.
    fn read_stdin(tx: mpsc::UnboundedSender<Message>, capacity: Arc<AtomicCapacity>) -> Result<()> {
        let mut stdin = std::io::stdin();
        // Keep track of the number of messages we've read.  We compare this to the number of
        // messages the server has written, as received in its regular capacity update messages, so
        // that we don't overwhelm the server.
        let mut messages_read = 0u64;

        loop {
            // Wait for server to have capacity for writes before reading; don't give "false hope" to
            // whatever's writing to our stdin that it'll be read until there's room for it.
            Self::wait_for_capacity(messages_read, &capacity)?;

            // Read a batch of data at a time; 4k is a balanced number for small and large jobs.
            let mut buf = [0; 4096];
            let count = stdin
                .retry_read(&mut buf)
                .context(error::ReadFromUserSnafu)?;
            // A read of 0 indicates EOF, so we're done.
            if count == 0 {
                break;
            }
            messages_read += 1;

            // Send the data to the server in a Binary message without encoding.
            let msg = Vec::from(&buf[..count]);
            tx.unbounded_send(Message::Binary(msg))
                .context(error::SendMessageSnafu { kind: "user input" })?;
        }
        debug!("Finished reading input");

        // Send a ContentComplete message to the server so it can exit the process more cleanly.
        // This is more important than the TTY case; interactive use typically has users typing
        // exit, or quit, or ctrl-d... noninteractive programs typically wait for EOF.
        let msg = serde_json::to_string(&ClientMessage::ContentComplete)
            .context(error::SerializeSnafu)?;
        tx.unbounded_send(Message::Text(msg))
            .context(error::SendMessageSnafu {
                kind: "content complete",
            })?;

        Ok(())
    }

    /// Sleeps until the server has capacity to receive more process input.
    ///
    /// We know how many messages we've read from user input, and the AtomicCapacity is updated any
    /// time the server sends us a capacity update.  We compare read count to written count to know
    /// how many messages the server has yet to write, and if that's over the maximum number of
    /// messages the server wants outstanding, we wait.  (The server will terminate us otherwise.)
    fn wait_for_capacity(messages_read: u64, capacity: &Arc<AtomicCapacity>) -> Result<()> {
        let mut waited = 0u64;
        loop {
            let max_outstanding = capacity.max_messages_outstanding.load(Ordering::SeqCst);
            let messages_written = capacity.messages_written.load(Ordering::SeqCst);

            // Check how many messages are currently waiting to be written; read - written.
            // If the server has written more than we've read, something is quite wrong!
            let messages_outstanding =
                messages_read
                    .checked_sub(messages_written)
                    .context(error::ServerCountSnafu {
                        messages_read,
                        messages_written,
                    })?;

            // If there's capacity, we're done waiting.
            if messages_outstanding <= max_outstanding {
                break;
            }

            // Occasionally log that we're still waiting, if someone is watching at trace level.
            waited += 1;
            if waited % 100 == 0 {
                trace!("Waiting for server capacity...");
            }
            sleep(Duration::from_millis(10));
        }
        trace!("Server capacity OK, reading input");
        Ok(())
    }
}

/// AtomicCapacity is used to track the numbers we receive in capacity updates from the server in a
/// way that can be shared across our threads.
struct AtomicCapacity {
    /// The server will reject us if we have more than this number of input messages outstanding.
    max_messages_outstanding: AtomicU64,
    /// The number of messages that the server has confirmed it's written.  Messages are always
    /// handled in order, so we can directly compare this to the number of inputs we've read.
    messages_written: AtomicU64,
}

impl Default for AtomicCapacity {
    fn default() -> Self {
        Self {
            // We assume 0 capacity until the server tells us otherwise so that we don't send data
            // the server isn't ready to process.
            max_messages_outstanding: AtomicU64::new(0),
            messages_written: AtomicU64::new(0),
        }
    }
}

/// Heartbeat is responsible for confirming our connection to the server isn't stale.  We ping the
/// server regularly so it knows we're alive, and we confirm that the server has pinged us recently
/// so we know it's alive.
struct Heartbeat {
    /// An atomic handle to a timestamp; this should be updated whenever we receive a ping or pong
    /// from the server so we can make sure the connection isn't stale.
    setter: Arc<Mutex<Instant>>,
    /// If the heartbeat dies, we send a message on this channel so the client can stop.
    finished_rx: oneshot::Receiver<()>,
}

impl Heartbeat {
    /// Parameters:
    /// * ping_tx: The channel to which we should send ping messages.
    fn new(ping_tx: mpsc::UnboundedSender<Message>) -> Self {
        // Create the Instant we use to track when we last heard from the server.
        let getter = Arc::new(Mutex::new(Instant::now()));
        // Create another handle to the Instant that the caller uses to update the Instant.
        let setter = getter.clone();
        // Create a channel we use to tell the caller when the heartbeat dies.
        let (finished_tx, finished_rx) = oneshot::channel();

        debug!("Spawning heartbeat thread");
        thread::spawn(move || Self::heartbeat(ping_tx, getter, finished_tx));

        Self {
            setter,
            finished_rx,
        }
    }

    fn heartbeat(
        ping_tx: mpsc::UnboundedSender<Message>,
        heartbeat_getter: Arc<Mutex<Instant>>,
        finished_tx: oneshot::Sender<()>,
    ) {
        // Runs forever, unless we don't hear from the server for longer than SERVER_TIMEOUT, or if
        // the thread that updates the heartbeat dies.
        loop {
            sleep(HEARTBEAT_INTERVAL);

            match heartbeat_getter.lock() {
                Ok(hb) => {
                    if Instant::now().duration_since(*hb) > SERVER_TIMEOUT {
                        break;
                    }
                }
                Err(_) => {
                    // If we fail to get the mutex, the thread reading from the WebSocket has
                    // panicked, so there's no more need for a heartbeat; we're dead.
                    break;
                }
            }

            // There's not much we can do if we fail to send a ping; we're progressing toward a
            // timeout in any case, so we'll naturally do the right thing.
            let _ = ping_tx.unbounded_send(Message::Ping(vec![]));
        }

        // Tell the caller the heartbeat died.
        let _ = finished_tx.send(());
    }
}

/// HandleSignals is responsible for managing non-terminal signals (like when your window changes
/// size) and alerting the caller for terminal signals.
struct HandleSignals {
    /// If a terminal signal is received, its value is sent over this channel.
    signal_rx: oneshot::Receiver<i32>,
}

impl HandleSignals {
    /// Parameters:
    /// * winch_tx: The channel to which we should send window size change messages.
    fn new(winch_tx: mpsc::UnboundedSender<Message>) -> Result<Self> {
        // Create a channel we use to tell the caller when we receive a terminal signal.
        let (signal_tx, signal_rx) = oneshot::channel();

        // Set up the signal handler; do this before starting a thread so we can die quickly on
        // failure.
        use signal::*;
        let signals = Signals::new([SIGWINCH, SIGTERM, SIGINT, SIGQUIT])
            .context(error::HandleSignalsSnafu)?;

        debug!("Spawning thread to manage signals");
        thread::spawn(move || {
            if let Err(e) = Self::handle_signals(signals, winch_tx, signal_tx) {
                error!("Signal manager failed: {}", e);
            }
        });

        Ok(Self { signal_rx })
    }

    fn handle_signals(
        mut signals: Signals,
        winch_tx: mpsc::UnboundedSender<Message>,
        signal_tx: oneshot::Sender<i32>,
    ) -> Result<()> {
        use signal::*;
        loop {
            // Block until our process receives a signal.
            for signal in signals.wait() {
                if signal == SIGWINCH {
                    // Window size changes can happen any number of times; send an update to the
                    // server and wait for more signals.
                    Self::send_winch(&winch_tx);
                } else {
                    // Anything else is terminal; notify the caller and exit.
                    signal_tx
                        .send(signal)
                        .ok()
                        .context(error::SendSignalSnafu { signal })?;
                    // The signal and our handler have done their job, it's not an error.
                    return Ok(());
                }
            }
        }
    }

    /// Try to send a window size update to the server.  We don't consider window size updates to
    /// be critical, since the program is still functioning, so we don't return errors.
    fn send_winch(tx: &mpsc::UnboundedSender<Message>) {
        if let Some(winsize) = get_winsize(STDOUT_FILENO) {
            debug!(
                "Sending new window size to server: {} cols {} rows",
                winsize.cols, winsize.rows
            );
            if let Ok(msg) = serde_json::to_string(&ClientMessage::Winch(winsize)) {
                let _ = tx.unbounded_send(Message::Text(msg));
            }
        }
    }
}

/// Get the current window size of the user's terminal, if possible.  We don't consider window size
/// to be critical, since the program is still functioning, so we return Option rather than Result.
fn get_winsize(fd: RawFd) -> Option<Size> {
    let mut winsize = WinSize {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0, // unused
        ws_ypixel: 0, // unused
    };
    // unsafe because ioctls can do any number of crazy things and this is a libc call, but it's
    // about as safe an ioctl as there is.
    let ret = unsafe { ioctl(fd, GetWinSize, &mut winsize) };
    if ret != 0 {
        debug!("Failed to get window size");
        return None;
    }

    // Convert to our type that we can serialize for the server.
    Some(Size::from(winsize))
}

mod error {
    use super::{connect, mpsc, terminal, Message};
    use snafu::{IntoError, Snafu};

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub enum Error {
        // This is used as a sort of marker; the user doesn't need wording about the connection
        // being closed because the process will end, and if because of an error, they'll see that.
        #[snafu(display(""))]
        Close,

        // This is from our own module which includes enough context.
        #[snafu(display("{}", source))]
        Connect { source: connect::Error },

        #[snafu(display("Failed to deserialize message from server: {}", source))]
        Deserialize { source: serde_json::Error },

        #[snafu(display("Failed to set up signal handler: {}", source))]
        HandleSignals { source: std::io::Error },

        #[snafu(display("Failed to read input: {}", source))]
        ReadFromUser { source: std::io::Error },

        #[snafu(display("Failed to read from WebSocket: {}", source))]
        ReadWebSocket {
            source: tokio_tungstenite::tungstenite::Error,
        },

        #[snafu(display("Failed to send {} message to server: {}", kind, source))]
        SendMessage {
            kind: String,
            source: mpsc::TrySendError<Message>,
        },

        #[snafu(display("Received signal {}", signal))]
        SendSignal { signal: i32 },

        #[snafu(display(
            "Server said {} messages written, but we've only read {}?  Logic error!",
            messages_written,
            messages_read
        ))]
        ServerCount {
            messages_read: u64,
            messages_written: u64,
        },

        #[snafu(display("Failed to serialize message to server: {}", source))]
        Serialize { source: serde_json::Error },

        // This is from our own module which includes enough context.
        #[snafu(display("{}", source))]
        Terminal { source: terminal::Error },

        #[snafu(display("Failed to write output: {}", source))]
        WriteOutput { source: std::io::Error },
    }

    // This allows for the nice usage of err_into() on our WebSocket stream.
    impl From<tokio_tungstenite::tungstenite::Error> for Error {
        fn from(e: tokio_tungstenite::tungstenite::Error) -> Self {
            ReadWebSocketSnafu.into_error(e)
        }
    }
}
pub use error::Error;
type Result<T> = std::result::Result<T, error::Error>;
