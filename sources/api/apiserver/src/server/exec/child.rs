//! The 'child' module owns the child process for an exec request, and provides handles for the
//! caller to communicate with the process.
//!
//! The process is spawned into the namespaces of an existing container (task) using containerd;
//! the desired container and command are given by the caller.  We can also optionally create a PTY
//! for the task to run in, useful for interactive programs.  Process output is sent back as actor
//! messages, and process input is received on a channel.

// Implementation note: the main job of this module is communicating with the child process.  We
// use simple blocking calls for communication, so we organize the module with threads and
// channels.  A thread is started to manage each type of communication, like reading, writing,
// waiting for exit, etc.  When we need to send messages to the WebSocket actor, like for sending
// process output or return code, we're given its actor address.  When we need to receive ongoing
// messages from the WebSocket actor, like for process input, we create and hand back a channel.
//
// This behavior is encapsulated in structs.  For example, there's a WriteToChild struct; you
// create it and give it the actor address it can use to send (capacity updates) to the client, it
// starts a thread, and you get back the struct, which contains the channel where you dump user
// input.

use super::{message, WsExec, CAPACITY_UPDATE_INTERVAL, MAX_MESSAGES_OUTSTANDING};
use actix::prelude::{Addr, SendError};
use bytes::Bytes;
use libc::{ioctl, login_tty, winsize as WinSize, TIOCSWINSZ as SetWinSize};
use log::{debug, error};
use model::exec::{Capacity, Initialize, Size, TtyInit};
use nix::{
    errno::Errno,
    fcntl::{fcntl, FcntlArg, FdFlag, OFlag},
    pty::openpty,
    sys::signal::{kill, Signal},
    sys::wait::{waitpid, WaitStatus},
    unistd::{close, pipe2, read, Pid},
};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use snafu::{OptionExt, ResultExt};
use std::convert::TryFrom;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{self, Write};
use std::os::unix::{
    io::{FromRawFd, IntoRawFd, RawFd},
    process::CommandExt,
};
use std::process::{Command, Stdio};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::thread::{self, sleep};
use std::time::Duration;

/// ChildHandles represents a spawned child process and contains the handles necessary to interact
/// with it.
#[derive(Debug)]
pub(crate) struct ChildHandles {
    /// The process ID, for signaling or other manual actions on a process.
    pub(crate) pid: Pid,

    /// A file descriptor for the parent side of the PTY, which we use to change its window size.
    // Not pub; don't want to allow arbitrary reads/writes to child.
    pty_fd: RawFd,

    /// Whether we created a PTY for the user.
    // Not pub; used for internal tracking of whether to resize.
    tty: bool,

    /// We'll write anything sent to this channel to the stdin of the child process.
    pub(crate) write_tx: Option<SyncSender<Bytes>>,
}

impl ChildHandles {
    /// Parameters:
    /// * init: The initialization parameters for the process, meaning the target container, the
    /// command, and any TTY settings.
    ///
    /// * exec_socket_path: The containerd socket we'll use to start the process in the desired
    /// container's namespace.
    ///
    /// * ws_addr: The address of the WebSocket actor, for sending messages back.
    pub(crate) fn new(
        init: Initialize,
        exec_socket_path: impl AsRef<OsStr>,
        ws_addr: Addr<WsExec>,
    ) -> Result<Self> {
        // containerd requires an "exec ID" for each task exec.
        let exec_id = format!(
            "apiexec-{}",
            thread_rng()
                .sample_iter(&Alphanumeric)
                .take(16)
                .map(char::from)
                .collect::<String>()
        );

        // We use ctr as a simple interface to containerd exec requests; it does bookkeeping for us
        // that's required by containerd, and is simpler to interact with than the containerd API,
        // at least in Rust in 2021.
        let mut command = Command::new("/usr/bin/ctr");

        // Point it at the requested containerd socket; changes are useful for local testing.
        command.arg("-a");
        // (The path is a different type, OsStr, so it's passed separately.)
        command.arg(exec_socket_path.as_ref());

        // Ask ctr to exec into an existing task, with a TTY if requested by the user.
        command.args(["task", "exec", "--exec-id", &exec_id]);
        if init.tty.is_some() {
            command.arg("--tty");
        }
        // Pass the target container (task) and the requested command.
        command.arg(init.target);
        command.args(init.command);

        // ctr sets up a basic environment for spawned processes; there's no reason to inherit
        // anything from apiserver.
        command.env_clear();

        // Get read and write file descriptors, configured appropriately for the requested TTY
        // setup.  (Sometimes we'll also have a fd to close because PTYs are finicky.)
        let child_fds = ChildFds::new(&mut command, &init.tty)?;

        // We don't want to pass through a "real" TERM value because TUI programs will query for
        // terminal capabilities, like cursor position and color support, and we don't feed that
        // back to a terminal, just stdin of the requested program.  Use TERM=screen because it's
        // widely understood and doesn't emulate a real terminal in ways that are awkward for us,
        // meaning no ANSI escapes are passed back and forth.
        command.env("TERM", "screen");

        debug!("Spawning command for exec request: {:?}", command);
        let mut child = command.spawn().context(error::SpawnSnafu)?;

        // `Command` returns pid as u32, but we want i32 to deal with nix.
        let pid_raw = i32::try_from(child.id())
            .ok()
            .context(error::InvalidPidSnafu { given: child.id() })?;
        let pid = Pid::from_raw(pid_raw);
        debug!("Spawned child has pid {}", pid);

        // Work around partial move of 'init' into closure; not needed in Rust 2021?
        let tty = init.tty;

        // At this point we've spawned a child process but still have some configuration to do.  If
        // any of it fails, we want to return failure, but we want to make sure we kill the child
        // process too, or it'd stick around forever.  Perform the rest of initialization in a
        // closure so that we can kill the child easily on any error, returning the original value.
        (move || {
            // Starting with Rust 2021, need to explicitly capture the while child variable
            let _ = &child;

            // Now that the process is spawned, if we created a PTY, close its slave fd in the
            // parent process, or reads of the master side will block.
            if let Some(close_fd) = child_fds.close_fd {
                close(close_fd).context(error::CloseFdSnafu)?;
            }

            // `ctr` doesn't (yet) send initial PTY size to containerd, requiring us to send a
            // WINCH to have it fetch the given size and update.  Testing shows that 20ms is
            // usually enough time for it to set up its SIGWINCH handler; we wait 100ms, which
            // should be before any human reaction, but it still isn't perfect.  (The worst case
            // scenario is that terminal size is too small, e.g. 80 columns, until the user resizes
            // their terminal; we can live with that.)
            if tty.is_some() {
                thread::spawn(move || {
                    sleep(Duration::from_millis(100));
                    let _ = kill(pid, Signal::SIGWINCH);
                });
            }

            // Set up the thread that reads output from the child, sending it to the WebSocket.
            let read_from_child = ReadFromChild::new(child_fds.read_fd, ws_addr.clone());

            // If we didn't create a PTY, we have to fetch the child's stdin handle; this isn't
            // available until after the child is spawned, so ChildFds can't do it.
            let write_fd = match child_fds.write_fd {
                Some(write_fd) => write_fd,
                None => child
                    .stdin
                    .take()
                    .context(error::NoStdinSnafu)?
                    .into_raw_fd(),
            };

            // Set up the thread that writes input from the WebSocket to the child.
            let write_to_child = WriteToChild::new(write_fd, ws_addr.clone());

            // Set up the thread that waits for the child to exit, at which point it can clean up
            // and send the return code through the WebSocket.
            let _ = WaitForChild::new(pid, ws_addr, read_from_child.complete_rx);

            Ok(Self {
                pid,
                pty_fd: child_fds.read_fd,
                tty: tty.is_some(),
                write_tx: Some(write_to_child.write_tx),
            })
        })()
        // If anything went wrong when configuring the child process, kill it and return the
        // original error.
        .map_err(|e| {
            Self::stop_impl(pid);
            e
        })
    }

    /// Terminates the child process.
    pub(crate) fn stop(&self) {
        Self::stop_impl(self.pid)
    }

    // Internal helper for stopping the child by PID, for when we don't have a &self yet.
    fn stop_impl(pid: Pid) {
        // Note: if we started ctr with --tty, it doesn't forward signals or otherwise terminate
        // the requested child when it stops.  It's not in the same process group and we don't have
        // a good way to get the pids of the requested process and anything else it started, so
        // they live on.  This isn't too bad because a PTY is usually used for interactive
        // processes like shells which exit when you request or when their stdin is closed.
        // Processes started without a PTY are more likely to continue, but ctr does forward
        // signals to them so they're stopped when we issue this TERM.
        //
        // If we fail to send a signal to the child, there's not much we can do; it likely means
        // the process no longer exists.  (It's theoretically possible that this is a different
        // process if the client exited their requested process and the system spins through all
        // available pids in the split second since.  We could use a pidfd to be sure, but it would
        // lock us to Linux for testing.)
        let _ = kill(pid, Signal::SIGTERM);
    }

    /// Updates the window size that's recorded in the PTY device, which allows the child process
    /// to receive the new size and adjust its output, if desired.  Note that this is treated as a
    /// "nice to have" without a return value because the program will still function and it's easy
    /// to retry.
    pub(crate) fn set_winsize(&mut self, size: Size) {
        // If the user didn't request a TTY, we didn't create a PTY for them, and we shouldn't be
        // trying to resize it - the process may create its own PTY or something, we don't know.
        if !self.tty {
            return;
        }

        debug!(
            "Updating window size to {} cols {} rows",
            size.cols, size.rows
        );

        let mut winsize = WinSize::from(size);
        unsafe { ioctl(self.pty_fd, SetWinSize, &mut winsize) };
    }
}

/// ChildFds sets up read and write file descriptors for a Command (before it's spawned) based on
/// whether the user requested a TTY.
struct ChildFds {
    /// The file descriptor to read from to receive child process output.
    read_fd: RawFd,
    /// The file descriptor to write to when you have input for the child process.  If None, you
    /// should use Child.stdin() after spawning the child.
    write_fd: Option<RawFd>,
    /// A file descriptor that should be closed after spawning the process; only applicable for the
    /// TTY use case, to prevent blocking IO.
    close_fd: Option<RawFd>,
}

impl ChildFds {
    /// Parameters:
    /// * child: The Command for which we want read and write file descriptors.
    ///
    /// * tty: Represents the user's desire for a TTY.  If None, don't create a PTY.  If Some,
    /// create a PTY, and start it with the specs given in TtyInit.
    fn new(child: &mut Command, tty: &Option<TtyInit>) -> Result<Self> {
        if let Some(tty_init) = tty {
            Self::tty_fds(child, tty_init)
        } else {
            Self::pipe_fds(child)
        }
    }

    /// Set up FDs for the TTY use case.
    fn tty_fds(child: &mut Command, tty_init: &TtyInit) -> Result<Self> {
        debug!("Creating PTY for exec request");
        // Create a PTY with openpty, starting with a size if the user gave one.
        let pty = if let Some(size) = tty_init.size {
            let size = WinSize::from(size);
            openpty(Some(&size), None).context(error::OpenPtySnafu)?
        } else {
            openpty(None, None).context(error::OpenPtySnafu)?
        };
        // The "master" end of a PTY represents a user typing at a physical terminal; we connect
        // that to the user over the WebSocket.  The "slave" end is connected to the process
        // requested by the user.
        let read_fd = pty.master;

        // Set CLOEXEC on read_fd so it's closed automatically in the child; the child doesn't need
        // access to our end of the PTY.
        cloexec(read_fd)?;

        // We need to read and write to the master end; we dup the FD so that closing one doesn't
        // break the other.  dup() sets CLOEXEC so that this is closed in the child automatically.
        let write_fd = dup(read_fd)?;

        // We need to set up the slave end of the TTY in the child process after we fork but before
        // we exec, so that the requested process just sees normal file descriptors, not needing to
        // know that they're actually dealing with a PTY.  pre_exec lets us run a closure at that
        // time.  It's marked unsafe because you can't do much safely in that environment; you're
        // still in the parent process space, with its threads and open file descriptors, and it's
        // easy to misuse the duplicated resources.  We only need to do one safe operation designed
        // for this environment.
        // Implementation note: this seemed simpler than forkpty&exec because Command is a familiar
        // abstraction and this keeps the low-level bits to the TTY case that needs them.
        unsafe {
            child.pre_exec(move || {
                // login_tty does a bunch of useful things for us that make the child process act
                // like a "real" process the user would start in their own terminal.  It makes a
                // new session, sets the given FD as the controlling terminal and as stdin, stdout,
                // and stderr, and then closes the FD.
                if login_tty(pty.slave) != 0 {
                    return Err(io::Error::last_os_error());
                }
                Ok(())
            })
        };

        Ok(Self {
            read_fd,
            write_fd: Some(write_fd),
            close_fd: Some(pty.slave),
        })
    }

    /// Sets up FDs for the non-TTY use case.
    fn pipe_fds(child: &mut Command) -> Result<Self> {
        debug!("Creating Stdio pipe (no PTY) for exec request");
        // We'd like the child process's stdout and stderr to be read from one fd like the TTY
        // case.  Using the standard `Stdio::piped()` would leave us with two separate devices.
        // Instead, create an OS-level pipe; the child will write both stdout and stderr to one end
        // of the pipe and the parent will read from the other.  The child doesn't need access to
        // our end of the pipe so we use CLOEXEC to have it closed in the child automatically.
        let (read_fd, write_fd) = pipe2(OFlag::O_CLOEXEC).context(error::CreatePipeSnafu)?;
        // Make a duplicate for stderr.  dup() sets CLOEXEC for us.
        let write_fd_dup = dup(write_fd)?;

        // Create Stdio objects based on the pipe that the Command can accept for stdout and
        // stderr.  (It's marked unsafe to represent that these take sole ownership of the fd,
        // which is what we want, and why we dup the fd - closing one won't break the other.)
        let stdout = unsafe { Stdio::from_raw_fd(write_fd) };
        let stderr = unsafe { Stdio::from_raw_fd(write_fd_dup) };

        child.stdout(stdout);
        child.stderr(stderr);

        // We can use standard piped stdin, which we take() and turn into an fd after spawn.
        child.stdin(Stdio::piped());

        Ok(Self {
            read_fd,
            write_fd: None,
            close_fd: None,
        })
    }
}

/// Set CLOEXEC on the given file descriptor so it's automatically closed in child processes.
fn cloexec(fd: RawFd) -> Result<()> {
    // First, get the current settings.
    let flags = fcntl(fd, FcntlArg::F_GETFD).context(error::FcntlSnafu)?;
    // Turn the result into the nix type; can't fail, the bits just came from fcntl.
    let mut flags = FdFlag::from_bits(flags).expect("F_GETFD result not valid FdFlag?");
    // Set CLOEXEC.
    flags.set(FdFlag::FD_CLOEXEC, true);
    // Update the settings on the fd.
    fcntl(fd, FcntlArg::F_SETFD(flags)).context(error::FcntlSnafu)?;
    Ok(())
}

/// Duplicates a file descriptor with CLOEXEC set, returning the new fd.  (If you care which fd
/// number you get back, use unistd::dup3 instead.)
fn dup(fd: RawFd) -> Result<RawFd> {
    // We don't care what FD number the following duplicate gets.
    let minimum_fd = 0;
    // Create the requested duplicate.  Use fcntl rather than unistd::dup so we can immediately set
    // CLOEXEC, automatically closing this FD in any child process.
    fcntl(fd, FcntlArg::F_DUPFD_CLOEXEC(minimum_fd)).context(error::DupFdSnafu)
}

/// WriteToChild is responsible for accepting user input from a channel connected to the WebSocket
/// and writing that input to the child's stdin.  Based on its write progress, it sends capacity
/// updates back to the client (through the WebSocket actor) so the client knows how much progress
/// we've made and how many more input messages we can accept.
struct WriteToChild {
    /// Send user input to this channel and it'll be written to the child process's stdin.
    write_tx: SyncSender<Bytes>,
}

impl WriteToChild {
    /// Parameters:
    /// * write_fd: The stdin FD of the child to which we'll write process input.
    ///
    /// * ws_addr: The address of the WebSocket actor, to which we'll send capacity updates.
    fn new(write_fd: RawFd, ws_addr: Addr<WsExec>) -> Self {
        // Create a File from the FD so we can use convenience methods like write_all.
        // This method is marked unsafe to represent that it takes sole ownership of the fd; we
        // dup() write_fd so closes of read_fd or write_fd don't break the other.
        let write_file = unsafe { File::from_raw_fd(write_fd) };
        // When we receive data from the client to write to the child process, it's sent to the
        // writer thread through this bounded channel.  The bound lets us control how many process
        // input messages can be outstanding at any time.  We send regular capacity updates to the
        // client so they can throttle their reads and not get cut off.
        let (write_tx, write_rx) = sync_channel(MAX_MESSAGES_OUTSTANDING as usize);

        debug!("Spawning thread to write to child");
        thread::spawn(move || Self::write_to_child(write_file, ws_addr, write_rx));

        Self { write_tx }
    }

    fn write_to_child(mut file: File, ws_addr: Addr<WsExec>, write_rx: Receiver<Bytes>) {
        // Keep track of the number of messages we've written to the child.  We put this in the
        // capacity update to the client, and use it to determine whether we're ready to send an
        // update.
        let mut messages_written = 0u64;

        while let Ok(data) = write_rx.recv() {
            // If we can't write to the child process, end the loop and drop the channel so the
            // WebSocket receiver knows we can't accept any more and it should close.
            if let Err(e) = file.write_all(&data) {
                error!("Failed to write to child process: {}", e);
                break;
            }

            messages_written += 1;
            // Every so often, send a capacity update to the client so it knows what we've written
            // and how many messages we're willing to accept.
            if messages_written % CAPACITY_UPDATE_INTERVAL == 0 {
                let capacity = Capacity {
                    max_messages_outstanding: MAX_MESSAGES_OUTSTANDING,
                    messages_written,
                };
                // Capacity updates are mandatory messages back to the client because the client
                // would wait forever if they believe there's no write capacity; use do_send to
                // ignore mailbox limits.
                ws_addr.do_send(message::CapacityUpdate(capacity));
            }
        }
    }
}

/// WaitForChild is responsible for waiting for the child process to exit so it can check the
/// return code and do any necessary process cleanup.
struct WaitForChild {}

impl WaitForChild {
    /// Parameters:
    /// * pid: The child process ID we're waiting for.
    ///
    /// * ws_addr: The address of the WebSocket actor, to which we'll send the return code.
    ///
    /// * read_complete_rx: We should receive a signal on this channel when the reader thread is
    /// finished.  PTY I/O is buffered in the kernel, so when a process exits, it doesn't mean
    /// we're done reading from the PTY; this lets us be sure.
    fn new(pid: Pid, ws_addr: Addr<WsExec>, read_complete_rx: Receiver<()>) -> Self {
        debug!("Spawning thread to wait for child exit");
        thread::spawn(move || Self::wait_for_child(pid, ws_addr, read_complete_rx));

        Self {}
    }

    fn wait_for_child(pid: Pid, ws_addr: Addr<WsExec>, read_complete_rx: Receiver<()>) {
        // Wait for the child to exit.  (Command::wait closes stdin; we need more control.)
        let res = waitpid(Some(pid), None);
        debug!("Child process exited");

        let code = match res {
            // If it exited with a code, use that.
            Ok(WaitStatus::Exited(_pid, code)) => code,

            // Use shell-style return codes for signals.
            Ok(WaitStatus::Signaled(_pid, signal, _core)) => {
                // (nix signals are repr(i32) to match c_int from libc.)
                let signal_int = signal as i32;
                128 + signal_int
            }

            // waitpid() shouldn't complete unless the process terminated, since we didn't request
            // notification of stopped/signaled processes.  If we get here, we don't know what
            // happened and don't have a useful code to send.
            _ => 0,
        };

        // Wait for reads to complete from the PTY, if possible; PTYs are buffered so we may not
        // have all the output when the process finishes.
        //
        // It's important to do this before sending the ProcessReturn message that stops the
        // WebSocket, rather than waiting in the ProcessReturn handler, because this way we're
        // putting all ProcessOutput messages into the mailbox before ProcessReturn, guaranteeing
        // they'll be handled (sent to client) before we stop.
        //
        // The timeout is somewhat arbitrary; PTYs have no timing guarantees.  It usually takes a
        // few milliseconds, but losing output is bad.
        let _ = read_complete_rx.recv_timeout(Duration::from_millis(500));

        // Return code is a mandatory message back to client, so use do_send to ignore mailbox
        // limits.
        ws_addr.do_send(message::ProcessReturn { code });
    }
}

/// ReadFromChild is responsible for reading output from the child process and sending it to the
/// WebSocket actor for transmission back to the client.
struct ReadFromChild {
    /// The caller can read from this channel to be notified when reading is complete.
    complete_rx: Receiver<()>,
}

impl ReadFromChild {
    /// Parameters:
    /// * read_fd: The file descriptor of the child from which we'll read process output.
    ///
    /// * ws_addr: The address of the WebSocket actor, to which we'll send process output.
    fn new(read_fd: RawFd, ws_addr: Addr<WsExec>) -> Self {
        let (complete_tx, complete_rx) = sync_channel(1);

        debug!("Spawning thread to read from child");
        thread::spawn(move || Self::read_from_child(read_fd, ws_addr, complete_tx));

        Self { complete_rx }
    }

    fn read_from_child(fd: RawFd, ws_addr: Addr<WsExec>, complete_tx: SyncSender<()>) {
        // Read until the process is done or we fail.
        'outer: loop {
            // Read a batch of data at a time; 4k is a balanced number for small and large jobs.
            let mut output = vec![0; 4096];
            match read(fd, &mut output) {
                Ok(0) => {
                    debug!("Finished reading from child");
                    break;
                }
                Ok(n) => {
                    // Don't store extra zeroes if the child didn't have a full buffer's worth.
                    output.truncate(n);

                    // Send the output to the WebSocket actor for transmission to the client.  If
                    // the actor's mailbox is full, just keep trying; we don't have to worry about
                    // backpressure here because there are no buffers filling up, the child can't
                    // output unless we're reading.
                    let mut msg = message::ProcessOutput { output };
                    loop {
                        match ws_addr.try_send(msg) {
                            // Sent to actor OK.
                            Ok(_unit) => {
                                break;
                            }

                            // Mailbox full; wait a bit and try again.
                            Err(SendError::Full(returned_msg)) => {
                                msg = returned_msg;
                                sleep(Duration::from_millis(10));
                            }

                            // The actor stopped, so we're done; there is no more client.
                            Err(SendError::Closed(_msg)) => {
                                break 'outer;
                            }
                        }
                    }
                }
                Err(e) => {
                    // Retry if read is interrupted.
                    if e == Errno::EINTR {
                        continue;
                    }

                    // EIO happens naturally when the child exits or closes output file
                    // descriptors, so log that quietly.  Any other error is probably trouble.
                    // Either way, we're done reading.  --Old MacDonald, 2021
                    if e == Errno::EIO {
                        debug!("Child closed output");
                    } else {
                        error!("Failed reading from child: {}", e);
                    }
                    break;
                }
            }
        }
        // Notify that we're done reading.
        let _ = complete_tx.try_send(());
    }
}

mod error {
    use snafu::Snafu;
    use std::io;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub(crate) enum Error {
        #[snafu(display(
            "Failed to close file descriptor, which could lead to a hang: {}",
            source
        ))]
        CloseFd { source: nix::Error },

        #[snafu(display("Unable to create pipe to coalesce child output: {}", source))]
        CreatePipe { source: nix::Error },

        #[snafu(display("Unable to dup file descriptor for writing: {}", source))]
        DupFd { source: nix::Error },

        #[snafu(display("Unable to set CLOEXEC on file descriptor: {}", source))]
        Fcntl { source: nix::Error },

        #[snafu(display("Child had invalid PID '{}', should never happen", given))]
        InvalidPid { given: u32 },

        #[snafu(display("Child has no stdin, should never happen"))]
        NoStdin,

        #[snafu(display("Unable to open PTY for child: {}", source))]
        OpenPty { source: nix::Error },

        #[snafu(display("Failed to spawn process: {}", source))]
        Spawn { source: io::Error },
    }
}
type Result<T> = std::result::Result<T, error::Error>;
