//! The 'exec' module holds types used to communicate between client and server for
//! 'apiclient exec'.
use libc::winsize as WinSize;
use serde::{Deserialize, Serialize};
use std::ffi::OsString;

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// Server messages to client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    Capacity(Capacity),
}

/// A capacity update; this tells the client how many writes the server has completed so the client
/// can figure out how many more input messages it can read and send.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capacity {
    /// The maximum number of messages the server is willing to have outstanding before it
    /// terminates the client.
    pub max_messages_outstanding: u64,
    /// The number of input messages the server has successfully written to the child process.
    pub messages_written: u64,
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// Client messages to server.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ClientMessage {
    // It'd be nice to include initialization parameters in the initial HTTP request body, but not
    // all WebSocket clients support data there.
    Initialize(Initialize),
    ContentComplete,
    Winch(Size),
}

/// Tells the server how to initialize the command the user is requesting.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Initialize {
    /// What command (and arguments) to run.
    pub command: Vec<OsString>,
    /// What container (task) to run the command in.
    pub target: String,
    /// Whether the user wants a TTY.
    pub tty: Option<TtyInit>,
}

/// If the user wants a TTY, these are the initial parameters the TTY should be set up with.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TtyInit {
    /// Initial size of the TTY window.
    pub size: Option<Size>,
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=
// Helper types

// Note: nix::pty::Winsize == libc::winsize.
// WinSize doesn't support serde, so we make a slim wrapper.
/// Size of the terminal window.
#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
pub struct Size {
    pub rows: u16,
    pub cols: u16,
}

impl From<Size> for WinSize {
    fn from(size: Size) -> Self {
        Self {
            ws_row: size.rows,
            ws_col: size.cols,
            ws_xpixel: 0,
            ws_ypixel: 0,
        }
    }
}

impl From<WinSize> for Size {
    fn from(winsize: WinSize) -> Self {
        Self {
            rows: winsize.ws_row,
            cols: winsize.ws_col,
        }
    }
}
