/*!
This module simplifies the usage of the update APIs, and makes them synchronous by waiting for
updated status after commands are started.

The update API forks off a thar-be-updates process to do the long-running work that affects the
system.  So that users can know when that's complete, the /updates/status call includes a
`most_recent_command` structure giving details about the time, type, and result of the last issued
command.  So, this is our basic process:
* Check initial status, saving the timestamp.
* Issue our request.
* Poll the status API (with timeout and failure checks) until the timestamp changes.
* Confirm that the status shows that our requested command was successful.

Note: there's some potential for conflict with other callers, even with the API locking.  If
someone else issues a command after ours completes, but before we can check, the status could be
lost.  The update API's state machine ensures we only move in expected directions, and we don't
want to reinvent its logic here, or be too strict about timing.  We can just time out, or perhaps
sync up if their request was the same.  If it becomes a problem, we could perhaps use something
like transactions, or timed lock files, to avoid it.
*/

use super::{raw_request, raw_request_unchecked};
use http::StatusCode;
use log::{debug, info, trace, warn};
use snafu::{ensure, ResultExt};
use std::path::Path;
use std::time::Duration;
use tokio::time;

/// Refresh the list of available updates and return the current status.
pub async fn check<P>(socket_path: P) -> Result<String>
where
    P: AsRef<Path>,
{
    info!("Refreshing updates...");
    let (_body, status) = wait_request(
        socket_path,
        "/actions/refresh-updates",
        "POST",
        None,
        "refresh",
        &WaitPolicy::new(Duration::from_millis(100), 10 * 10),
    )
    .await?;

    Ok(status)
}

/// Checks whether an update is required given the current update status, and informs the user
/// about the specific state.
// Note: we shouldn't reimplement the update API's state tracking or knowledge here, but giving
// a useful message rather than "failed to prepare update" in common scenarios is critical.
pub fn required(check_output: &str) -> bool {
    let state =
        response_field(&["update_state"], check_output).unwrap_or_else(|| "unknown".to_string());
    match state.as_ref() {
        "Idle" => {
            info!("No updates available.");
            false
        }
        "Ready" => {
            info!("Update already applied; reboot for it to take effect, or request a cancel.");
            false
        }
        _ => {
            debug!("Saw state '{}', assuming update is required", state);
            true
        }
    }
}

/// Applies the update shown as selected in the output of check(), and makes it active.
pub async fn apply<P>(socket_path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    info!("Downloading and applying update to disk...");
    let (_body, _status) = wait_request(
        &socket_path,
        "/actions/prepare-update",
        "POST",
        None,
        "prepare",
        &WaitPolicy::new(Duration::from_millis(500), 2 * 60 * 10),
    )
    .await
    .context(error::PrepareUpdateSnafu)?;

    info!("Setting the update active so it will apply on the next reboot...");
    let (_body, _status) = wait_request(
        &socket_path,
        "/actions/activate-update",
        "POST",
        None,
        "activate",
        &WaitPolicy::new(Duration::from_millis(100), 10 * 5),
    )
    .await?;

    Ok(())
}

/// Cancels an applied update so another can be applied.
pub async fn cancel<P>(socket_path: P) -> Result<String>
where
    P: AsRef<Path>,
{
    info!("Canceling update...");
    let (_body, status) = wait_request(
        socket_path,
        "/actions/deactivate-update",
        "POST",
        None,
        "deactivate",
        &WaitPolicy::new(Duration::from_millis(100), 10 * 5),
    )
    .await?;

    Ok(status)
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// Pulls a nested field out of a JSON string.  The input is a list of strings representing the
/// nested structures, e.g. ["a", "b"] to select the 42 from {"a": {"b": 42}}.
///
/// Returns None if requested key doesn't exist or we can't fetch it for any reason.
// This deserializes the JSON each time, but we know our input is small, and it's worth the
// convenience.  Same reason for not using a Result.
fn response_field(
    field: impl IntoIterator<Item = &'static &'static str>,
    response_str: &str,
) -> Option<String> {
    let response: serde_json::Value = match serde_json::from_str(response_str) {
        Ok(json) => json,
        Err(_) => return None,
    };

    let mut result = &response;
    for component in field {
        match result.get(component) {
            Some(x) => result = x,
            None => break,
        }
    }
    result.as_str().map(|s| s.to_string())
}

/// Represents how a caller wants wait_request to handle waiting for status updates after
/// requesting an action from an API call.  After `max_attempts` checks with `between_attempts`
/// duration between them, the call will be timed out and fail.
#[derive(Debug)]
struct WaitPolicy {
    between_attempts: Duration,
    max_attempts: u32,
}

impl WaitPolicy {
    fn new(between_attempts: Duration, max_attempts: u32) -> Self {
        Self {
            between_attempts,
            max_attempts,
        }
    }
}

/// This synchronously wraps a call to the update API, waiting for asynchronous status updates to
/// be complete before returning.  The parameters are the same as for raw_request, plus a few
/// required to understand how we should wait:
/// *  The name of the command, as given in "cmd_type" of the most_recent_command structure of
///    update API responses, e.g. "refresh" for calls to /action/refresh-updates.
/// * A WaitPolicy that describes how long we should wait for the action to complete.
async fn wait_request<P, S1, S2>(
    socket_path: P,
    url: S1,
    method: S2,
    data: Option<String>,
    command_name: &'static str,
    wait: &WaitPolicy,
) -> Result<(String, String)>
where
    P: AsRef<Path>,
    S1: AsRef<str>,
    S2: AsRef<str>,
{
    // Fetch the initial status of the API so we know when it's changed.
    let (code, initial_body) = raw_request_unchecked(&socket_path, "/updates/status", "GET", None)
        .await
        .context(error::GetStatusSnafu)?;

    // The timestamp is the primary field we use to notice a change.
    // The first call to the update API on a new system will return a 404, which is fine.
    let before_timestamp = if code == StatusCode::NOT_FOUND {
        "first call".to_string()
    } else if code.is_success() {
        response_field(&["most_recent_command", "timestamp"], &initial_body)
            .unwrap_or_else(|| "first call".to_string())
    } else {
        return error::MissingStatusSnafu {
            code,
            body: initial_body,
        }
        .fail();
    };
    debug!("Found initial timestamp '{}'", before_timestamp);

    // Make the real request the user wanted.
    let (_code, response_body) = raw_request(&socket_path, &url, &method, data)
        .await
        .context(error::RequestSnafu { command_name })?;

    // Note: we've now made the real request the user asked for, and the rest is our bookkeeping to
    // wait for it to finish.  We're more careful with retries and don't want to early-exit with ?.

    let mut attempt: u32 = 0;
    let mut failures: u32 = 0;
    // How many times we'll retry our bookkeeping checks, e.g. if the status API fails.
    let max_failures: u32 = 5;
    // How often to let the user know we're still waiting.
    let notify_every = Duration::from_secs(5);
    // Counter so we can tell whether we should notify.
    let mut waited = Duration::from_millis(0);

    loop {
        // Check if we've timed out or failed too many requests.
        attempt += 1;
        ensure!(
            attempt < wait.max_attempts,
            error::TimedOutSnafu {
                waited: format!("{:?}", wait.between_attempts * wait.max_attempts),
                method: method.as_ref(),
                url: url.as_ref(),
            }
        );
        ensure!(
            failures < max_failures,
            error::StatusCheckSnafu {
                failures,
                method: method.as_ref(),
                url: url.as_ref(),
            }
        );

        // Let the user know what's going on every once in a while, as we wait.
        if attempt > 1 && waited >= notify_every {
            waited = Duration::from_millis(0);
            info!(
                "Still waiting for updated status, will wait up to {:?} longer...",
                (wait.max_attempts * wait.between_attempts) - (attempt * wait.between_attempts)
            );
        }
        time::sleep(wait.between_attempts).await;
        waited += wait.between_attempts;

        // Get updated status to see if anything's changed.
        let response = raw_request_unchecked(&socket_path, "/updates/status", "GET", None).await;
        let (code, status_body) = match response {
            Ok((code, status_body)) => (code, status_body),
            Err(e) => {
                failures += 1;
                warn!(
                    "Unable to check for update status, failure #{}: {}",
                    failures, e
                );
                continue;
            }
        };

        // Mutating actions will return a LOCKED status if they're not yet complete.
        if code == StatusCode::LOCKED {
            trace!("Lock still held, presumably by our request...");
            continue;
        } else if !code.is_success() {
            failures += 1;
            warn!(
                "Got code {} when checking update status, failure #{}: {}",
                code, failures, status_body
            );
            continue;
        }

        // Get the specific status fields we check.
        let after_timestamp = response_field(&["most_recent_command", "timestamp"], &status_body)
            .unwrap_or_else(|| "missing".to_string());
        let after_command = response_field(&["most_recent_command", "cmd_type"], &status_body)
            .unwrap_or_else(|| "missing".to_string());
        debug!("Found timestamp '{}' and command '{}' (looking for command '{}' and a change from initial timestamp '{}')",
                after_timestamp, after_command, command_name, before_timestamp);

        // If we have a new timestamp and see our expected command, we're done.
        // Note: we keep waiting if the timestamp changed but the command isn't what we expected;
        // see module docstring.
        if after_command == command_name && before_timestamp != after_timestamp {
            let after_status = response_field(&["most_recent_command", "cmd_status"], &status_body)
                .unwrap_or_else(|| "missing".to_string());
            // If the command wasn't successful, give as much info as we can from the status.
            ensure!(
                after_status == "Success",
                error::CommandSnafu {
                    command_name,
                    stderr: response_field(&["most_recent_command", "stderr"], &status_body)
                        .unwrap_or_else(|| "<missing>".to_string()),
                    status_name: after_status,
                    exit_status: response_field(
                        &["most_recent_command", "exit_status"],
                        &status_body
                    )
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(-1),
                }
            );
            return Ok((response_body, status_body));
        }
    }
}

mod error {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub enum Error {
        #[snafu(display(
            "{} attempt failed with status '{}' ({}): {}",
            command_name,
            status_name,
            exit_status,
            stderr
        ))]
        Command {
            command_name: String,
            status_name: String,
            exit_status: i32,
            stderr: String,
        },

        #[snafu(display("Failed getting update status: {}", source))]
        GetStatus {
            #[snafu(source(from(crate::Error, Box::new)))]
            source: Box<crate::Error>,
        },

        #[snafu(display("Unable to check initial update status, got code '{}': {}", code, body))]
        MissingStatus {
            code: http::StatusCode,
            body: String,
        },

        // This is likely to be a common source of issues, so we have an extra-clear error message
        // wrapper.
        #[snafu(display(
            "Failed to prepare update.  This could mean that we don't have a list of updates yet \
             or that an update is already applied.  Running 'apiclient update check' will help \
             you find out.  You can cancel an applied ('Ready') update with 'apiclient update \
             cancel' if desired.  Detail: {}",
            source
        ))]
        PrepareUpdate {
            #[snafu(source(from(Error, Box::new)))]
            source: Box<Error>,
        },

        #[snafu(display("Failed to make {} request: {}", command_name, source))]
        Request {
            command_name: String,
            #[snafu(source(from(crate::Error, Box::new)))]
            source: Box<crate::Error>,
        },

        #[snafu(display(
            "Failed to check status {} times after {} to {}, unsure of result",
            failures,
            method,
            url
        ))]
        StatusCheck {
            failures: u32,
            method: String,
            url: String,
        },

        #[snafu(display(
            "Timed out after waiting {} seconds after {} to {}, unsure of result",
            waited,
            method,
            url
        ))]
        TimedOut {
            waited: String,
            method: String,
            url: String,
        },
    }
}
pub use error::Error;
pub type Result<T> = std::result::Result<T, error::Error>;
