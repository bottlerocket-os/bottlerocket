//! The 'terminal' module provides a Terminal type that acts as a guard around changes to terminal
//! settings, resetting them to their original state when the Terminal is dropped.

use super::get_winsize;
use libc::{STDIN_FILENO, STDOUT_FILENO};
use log::{debug, warn};
use model::exec::TtyInit;
use nix::{
    sys::termios::{cfmakeraw, tcgetattr, tcsetattr, SetArg, Termios},
    unistd::isatty,
};
use snafu::ResultExt;

/// The Terminal type acts as a guard around changes to terminal settings, resetting them to their
/// original state when the Terminal is dropped.
#[derive(Debug)]
pub(crate) struct Terminal {
    /// If the user requested a TTY or we detected one, this will contain a TtyInit that represents
    /// the desired initial state of the TTY.
    tty: Option<TtyInit>,
    /// Represents the original terminal settings we found when we were created, so they can be
    /// restored when we're dropped.
    orig_termios: Option<Termios>,
}

impl Terminal {
    /// Parameters:
    /// * tty: Represents the user's desire for a TTY, where `Some(true)` means to use a TTY,
    /// `Some(false)` means not to use a TTY, and `None` means to detect whether we think we should
    /// use a TTY.
    ///
    /// For the purposes of terminal settings, "use a TTY" means to set the terminal to
    /// raw mode so that input is read directly, not interpreted; for example, things like ctrl-c
    /// will no longer generate a signal, so they can be passed on exactly as received.
    ///
    /// We detect a TTY by checking whether stdin *and* stdout are linked to a terminal device,
    /// which seems to surprise the fewest number of users.
    pub(crate) fn new(tty: Option<bool>) -> Result<Self> {
        let is_tty = match tty {
            Some(true) => true,
            Some(false) => false,
            None => {
                let stdin_tty = isatty(STDIN_FILENO) == Ok(true);
                let stdout_tty = isatty(STDOUT_FILENO) == Ok(true);
                let is_tty = stdin_tty && stdout_tty;
                debug!("Detected tty: {}", is_tty);
                is_tty
            }
        };

        let mut tty = None;
        let mut orig_termios = None;

        if is_tty {
            // We want any new TTY to match the size of our current terminal.
            tty = Some(TtyInit {
                size: get_winsize(STDOUT_FILENO),
            });

            // Get the current settings of the user's terminal so we can restore them later.
            let current_termios =
                tcgetattr(STDOUT_FILENO).context(error::TermAttrSnafu { op: "get" })?;

            debug!("Setting terminal to raw mode, sorry about the carriage returns");
            let mut new_termios = current_termios.clone();
            // Set to raw mode, sushi-grade.  ctrl-c, ctrl-z, etc. will no longer generate local
            // signals so they can be passed on unchanged and you can interact with remote programs
            // as expected.
            cfmakeraw(&mut new_termios);
            // We make the change 'NOW' because we don't expect any input/output yet, and so should
            // have nothing to FLUSH.
            tcsetattr(STDOUT_FILENO, SetArg::TCSANOW, &new_termios)
                .context(error::TermAttrSnafu { op: "set" })?;

            orig_termios = Some(current_termios);
        }

        Ok(Self { tty, orig_termios })
    }

    pub(crate) fn tty(&self) -> &Option<TtyInit> {
        &self.tty
    }
}

impl Drop for Terminal {
    /// Restore the user's original terminal settings on drop.
    fn drop(&mut self) {
        if let Some(orig_termios) = &self.orig_termios {
            // We shouldn't fail to reset unless stdout was closed somehow, and there's not much we
            // can do about cleaning it up then.
            if tcsetattr(STDOUT_FILENO, SetArg::TCSANOW, orig_termios).is_err() {
                warn!("Failed to clean up terminal :(");
            }
        }
    }
}

pub(crate) mod error {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub enum Error {
        #[snafu(display("Have TTY, but failed to {} terminal attributes: {}", op, source))]
        TermAttr { op: String, source: nix::Error },
    }
}
pub(crate) use error::Error;
type Result<T> = std::result::Result<T, error::Error>;
