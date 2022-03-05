use crate::error::{self, Result};
use log::trace;
use snafu::ResultExt;
use std::process::Command;

const SYSTEMCTL: &str = "/usr/bin/systemctl";

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct SystemHealth {
    /// Whether or not the system is healthy.
    pub(crate) is_healthy: bool,
    /// The system status description.
    pub(crate) status: String,
    /// The system status code.
    pub(crate) exit_code: i32,
}

pub(crate) struct SystemCheck {}

impl SystemCheck {
    pub fn system_running(&self) -> Result<SystemHealth> {
        let mut result = is_system_running()?;
        // remove trailing newline
        result.stdout.pop();
        Ok(SystemHealth {
            is_healthy: result.is_success(),
            status: result.stdout,
            exit_code: result.exit,
        })
    }
}

struct Outcome {
    exit: i32,
    stdout: String,
}

impl Outcome {
    fn is_success(&self) -> bool {
        self.exit == 0
    }
}

fn systemctl(args: &[&str]) -> Result<Outcome> {
    trace!("calling systemctl with '{:?}'", args);
    let output = Command::new(SYSTEMCTL)
        .args(args)
        .output()
        .with_context(|_| error::CommandSnafu {
            command: SYSTEMCTL,
            args: args.iter().map(|&s| s.to_owned()).collect::<Vec<String>>(),
        })?;
    Ok(Outcome {
        exit: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(output.stdout.as_slice()).into(),
    })
}

fn is_system_running() -> Result<Outcome> {
    let outcome = systemctl(&["--wait", "is-system-running"])?;
    Ok(outcome)
}
