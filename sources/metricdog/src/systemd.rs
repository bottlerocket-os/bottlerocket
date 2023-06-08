use crate::service_check::{ServiceCheck, ServiceHealth};

use crate::error::{self, Result};
use crate::host_check::HostCheck;
use log::trace;
use snafu::{OptionExt, ResultExt};
use std::process::Command;

const EXIT_STATUS_PROPERTY: &str = "ExecMainStatus";
/// This systemd unit property records the time between boot and when the unit enters the 'active'
/// state in the order of microseconds
const ACTIVE_TIMESTAMP_PROPERTY: &str = "ActiveEnterTimestampMonotonic";

const SYSTEMCTL: &str = "/usr/bin/systemctl";
const JOURNALCTL: &str = "/usr/bin/journalctl";

#[derive(Clone, Copy)]
pub(crate) struct SystemdCheck {}

impl ServiceCheck for SystemdCheck {
    fn check(&self, service_name: &str) -> Result<ServiceHealth> {
        if is_ok(service_name)? {
            return Ok(ServiceHealth {
                is_healthy: true,
                exit_code: None,
            });
        }
        Ok(ServiceHealth {
            is_healthy: false,
            exit_code: parse_service_exit_code(service_name)?,
        })
    }
}

struct Outcome {
    exit: i32,
    stdout: String,
}

impl Outcome {
    fn is_exit_true(&self) -> bool {
        self.exit == 0
    }
}

fn command(cmd: &str, args: &[&str]) -> Result<Outcome> {
    trace!("calling '{}' with '{:?}'", cmd, args);
    let output = Command::new(cmd)
        .args(args)
        .output()
        .with_context(|_| error::CommandSnafu {
            command: cmd,
            args: args.iter().map(|&s| s.to_owned()).collect::<Vec<String>>(),
        })?;
    Ok(Outcome {
        exit: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(output.stdout.as_slice()).into(),
    })
}

fn is_active(service: &str) -> Result<bool> {
    let outcome = command(SYSTEMCTL, &["is-active", service])?;
    Ok(outcome.is_exit_true())
}

fn is_failed(service: &str) -> Result<bool> {
    let outcome = command(SYSTEMCTL, &["is-failed", service])?;
    Ok(outcome.is_exit_true())
}

fn is_ok(service: &str) -> Result<bool> {
    Ok(!is_failed(service)? && is_active(service)?)
}

/// Parses out the systemd unit's exit status from `systemctl show` output
fn parse_service_exit_code(service: &str) -> Result<Option<i32>> {
    // Don't check the command's exit code because systemctl returns non-zero codes for various
    // non-exceptional execution outcomes.
    let outcome = command(
        SYSTEMCTL,
        &["show", "--property", EXIT_STATUS_PROPERTY, service],
    )?;
    Ok(parse_property(&outcome.stdout, EXIT_STATUS_PROPERTY)
        .and_then(|exit_code| exit_code.parse::<i32>().ok()))
}

impl HostCheck for SystemdCheck {
    fn is_first_boot(&self) -> Result<bool> {
        // Upon first boot, the expected output contains a single line entry leading with '0 <hash>'
        let outcome = command(JOURNALCTL, &["--list-boots", "--quiet"])?;
        let lines: Vec<&str> = outcome.stdout.lines().collect();
        if lines.len() == 1 {
            if let Some(line) = lines.first() {
                return Ok(line.trim_start().starts_with("0 "));
            }
        }
        Ok(false)
    }

    fn preconfigured_time_ms(&self) -> Result<String> {
        activate_time("preconfigured.target")
    }

    fn configured_time_ms(&self) -> Result<String> {
        activate_time("configured.target")
    }

    fn network_ready_time_ms(&self) -> Result<String> {
        activate_time("network-online.target")
    }

    fn filesystem_ready_time_ms(&self) -> Result<String> {
        activate_time("local-fs.target")
    }
}

// Returns the time (in milliseconds) it took for the service to become active
fn activate_time(unit: &str) -> Result<String> {
    let outcome = command(
        SYSTEMCTL,
        &["show", "--property", ACTIVE_TIMESTAMP_PROPERTY, unit],
    )?;

    let time_in_microseconds = parse_property(&outcome.stdout, ACTIVE_TIMESTAMP_PROPERTY)
        .context(error::ActiveEnterTimestampSnafu { unit })?;

    // Return the time in milliseconds
    Ok((time_in_microseconds
        .parse::<u64>()
        .context(error::ParseToU64Snafu {
            input: time_in_microseconds,
        })?
        / 1_000)
        .to_string())
}

/// Utility function to parse out a systemd unit's property value
fn parse_property(stdout: &str, property: &str) -> Option<String> {
    trace!(
        "parsing stdout from 'systemctl show --property {}':\n{}",
        property,
        stdout
    );

    // The format of the response is expected to be: `ExecMainStatus=1\n`.
    // Split this at the equals sign, verify the left side and parse the right side.
    let mut split = stdout.splitn(2, '=');

    // verify that the returned property matches the expected/desired property
    if split.next().unwrap_or("") != property {
        return None;
    }

    // The iterator should now point to the exit code. If it does, remove the trailing newline
    // and parse it into an int. If the exit code cannot parse into an int, then return None.
    split
        .next()
        .map(|exit_code| exit_code.trim_end().to_string())
}

#[cfg(test)]
mod parse_property_tests {
    use crate::systemd::{parse_property, EXIT_STATUS_PROPERTY};

    #[test]
    fn parse_stdout_exit_0() {
        let got = parse_property(
            format!("{}=0", EXIT_STATUS_PROPERTY).as_str(),
            EXIT_STATUS_PROPERTY,
        )
        .map(|exit_code| exit_code.parse::<i32>().ok())
        .flatten()
        .unwrap();

        let want = 0;
        assert_eq!(got, want);
    }

    #[test]
    fn parse_stdout_exit_255() {
        let got = parse_property(
            format!("{}=255", EXIT_STATUS_PROPERTY).as_str(),
            EXIT_STATUS_PROPERTY,
        )
        .map(|exit_code| exit_code.parse::<i32>().ok())
        .flatten()
        .unwrap();

        let want = 255;
        assert_eq!(got, want);
    }

    #[test]
    fn parse_stdout_exit_0_with_newline() {
        let got = parse_property(
            format!("{}=0\n", EXIT_STATUS_PROPERTY).as_str(),
            EXIT_STATUS_PROPERTY,
        )
        .map(|exit_code| exit_code.parse::<i32>().ok())
        .flatten()
        .unwrap();

        let want = 0;
        assert_eq!(got, want);
    }

    #[test]
    fn parse_stdout_exit_255_with_newline() {
        let got = parse_property(
            format!("{}=255\n", EXIT_STATUS_PROPERTY).as_str(),
            EXIT_STATUS_PROPERTY,
        )
        .map(|exit_code| exit_code.parse::<i32>().ok())
        .flatten()
        .unwrap();

        let want = 255;
        assert_eq!(got, want);
    }

    #[test]
    fn parse_stdout_exit_extra_chars() {
        let got = parse_property(
            format!("{}=255foo", EXIT_STATUS_PROPERTY).as_str(),
            EXIT_STATUS_PROPERTY,
        )
        .map(|exit_code| exit_code.parse::<i32>().ok())
        .flatten();

        assert!(got.is_none());
    }

    #[test]
    fn parse_stdout_malformed() {
        let got = parse_property(
            format!("{} = 123", EXIT_STATUS_PROPERTY).as_str(),
            EXIT_STATUS_PROPERTY,
        )
        .map(|exit_code| exit_code.parse::<i32>().ok())
        .flatten();

        assert!(got.is_none());
    }

    #[test]
    fn parse_stdout_empty_string() {
        let got = parse_property("", EXIT_STATUS_PROPERTY)
            .map(|exit_code| exit_code.parse::<i32>().ok())
            .flatten();

        assert!(got.is_none());
    }

    #[test]
    fn parse_stdout_property_only() {
        let got = parse_property(EXIT_STATUS_PROPERTY, EXIT_STATUS_PROPERTY)
            .map(|exit_code| exit_code.parse::<i32>().ok())
            .flatten();

        assert!(got.is_none());
    }

    #[test]
    fn parse_stdout_property_and_equals_only() {
        let got = parse_property(
            format!("{}=", EXIT_STATUS_PROPERTY).as_str(),
            EXIT_STATUS_PROPERTY,
        )
        .map(|exit_code| exit_code.parse::<i32>().ok())
        .flatten();

        assert!(got.is_none());
    }
}
