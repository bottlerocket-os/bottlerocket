use crate::error::{self, Result};
use log::trace;
use snafu::ResultExt;
use std::process::Command;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct ServiceHealth {
    /// Whether or not the service is healthy.
    pub(crate) is_healthy: bool,
    /// In the event of an unhealthy service, the service's exit code (if found).
    pub(crate) exit_code: Option<i32>,
}

pub(crate) trait ServiceCheck {
    /// Checks the given service to see if it is healthy.
    fn check(&self, service_name: &str) -> Result<ServiceHealth>;
}

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

fn systemctl(args: &[&str]) -> Result<Outcome> {
    trace!("calling systemctl with '{:?}'", args);
    let output = Command::new("systemctl")
        .args(args)
        .output()
        .with_context(|| error::Command {
            command: "systemctl",
            args: args.iter().map(|&s| s.to_owned()).collect::<Vec<String>>(),
        })?;
    Ok(Outcome {
        exit: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(output.stdout.as_slice()).into(),
    })
}

fn is_active(service: &str) -> Result<bool> {
    let outcome = systemctl(&["is-active", service])?;
    Ok(outcome.is_exit_true())
}

fn is_failed(service: &str) -> Result<bool> {
    let outcome = systemctl(&["is-failed", service])?;
    Ok(outcome.is_exit_true())
}

fn is_ok(service: &str) -> Result<bool> {
    Ok(!is_failed(service)? && is_active(service)?)
}

const STATUS_PROPERTY: &str = "ExecMainStatus";

fn parse_service_exit_code(service: &str) -> Result<Option<i32>> {
    // we don't check the command's exit code because systemctl returns non-zero codes for various
    // non-exceptional execution outcomes.
    let outcome = systemctl(&["show", "--property", STATUS_PROPERTY, service])?;
    Ok(parse_stdout(&outcome.stdout))
}

fn parse_stdout(stdout: &str) -> Option<i32> {
    trace!(
        "parsing stdout from 'systemctl show --property {}':\n{}",
        STATUS_PROPERTY,
        stdout
    );

    // we expect the response to be formatted like this: ExecMainStatus=1\n
    // we will split this at the equals sign, verify the left side and parse the right side.
    let mut split = stdout.splitn(2, '=');

    // verify that the returned property matches the expected/desired property
    if split.next().unwrap_or("") != STATUS_PROPERTY {
        return None;
    }

    // the iterator should now give us the exit code. if it does, we remove the trailing newline
    // and parse it into an int. if we cannot parse it into an int, then we return None.
    split
        .next()
        .and_then(|exit_code| exit_code.trim_end().parse::<i32>().ok())
}

#[test]
fn parse_stdout_exit_0() {
    let got = parse_stdout(format!("{}=0", STATUS_PROPERTY).as_str()).unwrap();
    let want = 0;
    assert_eq!(got, want);
}

#[test]
fn parse_stdout_exit_255() {
    let got = parse_stdout(format!("{}=255", STATUS_PROPERTY).as_str()).unwrap();
    let want = 255;
    assert_eq!(got, want);
}

#[test]
fn parse_stdout_exit_0_with_newline() {
    let got = parse_stdout(format!("{}=0\n", STATUS_PROPERTY).as_str()).unwrap();
    let want = 0;
    assert_eq!(got, want);
}

#[test]
fn parse_stdout_exit_255_with_newline() {
    let got = parse_stdout(format!("{}=255\n", STATUS_PROPERTY).as_str()).unwrap();
    let want = 255;
    assert_eq!(got, want);
}

#[test]
fn parse_stdout_exit_extra_chars() {
    let got = parse_stdout(format!("{}=255foo", STATUS_PROPERTY).as_str());
    assert!(got.is_none());
}

#[test]
fn parse_stdout_malformed() {
    let got = parse_stdout(format!("{} = 123", STATUS_PROPERTY).as_str());
    assert!(got.is_none());
}

#[test]
fn parse_stdout_empty_string() {
    let got = parse_stdout("");
    assert!(got.is_none());
}

#[test]
fn parse_stdout_property_only() {
    let got = parse_stdout(STATUS_PROPERTY);
    assert!(got.is_none());
}

#[test]
fn parse_stdout_property_and_equals_only() {
    let got = parse_stdout(format!("{}=", STATUS_PROPERTY).as_str());
    assert!(got.is_none());
}
