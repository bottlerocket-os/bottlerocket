#![deny(unused_imports)]

/*!
# Introduction

Metricdog sends anonymous information about the health of a Bottlerocket host.
It does so by sending key-value pairs as query params in an HTTP GET request.

Metricdog also has the ability to check that a list of critical services is running.
It does so using `systemctl` and reports services that are not healthy.

### Proxy Support

Metricdog respects the environment variables `HTTPS_PROXY` and `NO_PROXY` to determine whether or
not its traffic should be proxied. These are set with the `network.http-proxy` and `network.noproxy`
settings when Metricdog is invoked by systemd. If you run Metricdog manually, you would need to
seed the environment with these variables manually.

# What it Sends

### The standard set of metrics:

* `sender`: the application sending the report.
* `event`: the event that invoked the report.
* `version`: the Bottlerocket version.
* `variant`: the Bottlerocket variant.
* `arch`: the machine architecture, e.g.'x86_64' or 'aarch64'.
* `region`: the region the machine is running in.
* `seed`: the seed value used to roll-out updates.
* `version_lock`: the optional setting that controls Bottlerocket update selection.
* `ignore_waves`: an update setting that allows hosts to update before their seed is reached.

### Additionally, when `metricdog` sends a 'health ping', it adds:

* `is_healthy`: true or false based on whether critical services are running.
* `failed_services`: a list of critical services that have failed, if any.

# Configuration

Configuration is read from a TOML file, which is generated from Bottlerocket settings:

```toml
# the url to which metricdog will send metrics information
metrics_url = "https://example.com/metrics"
# whether or not metricdog will send metrics. opt-out by setting this to false
send_metrics = true
# a list of systemd service names that will be checked
service_checks = ["apiserver", "containerd", "kubelet"]
# the region
region = "us-west-2"
# the update wave seed
seed = 1234
# what version bottlerocket should stay on
version_lock = "latest"
# whether bottlerocket should ignore update roll-out timing
ignore_waves = false
```
*/

mod args;
mod config;
mod error;
mod host_check;
#[cfg(test)]
mod main_test;
mod metricdog;
#[cfg(test)]
mod metricdog_test;
mod service_check;
mod systemd;

use crate::args::{Arguments, Command};
use crate::config::Config;
use crate::error::Result;
use crate::host_check::HostCheck;
use crate::metricdog::Metricdog;
use crate::service_check::ServiceCheck;
use crate::systemd::SystemdCheck;
use bottlerocket_release::BottlerocketRelease;
use log::error;
use simplelog::{Config as LogConfig, SimpleLogger};
use snafu::ResultExt;
use std::process;

fn main() -> ! {
    let args: Arguments = argh::from_env();
    SimpleLogger::init(args.log_level, LogConfig::default()).expect("unable to configure logger");
    let systemd_check = SystemdCheck {};
    process::exit(
        match main_inner(args, Box::new(systemd_check), Box::new(systemd_check)) {
            Ok(()) => 0,
            Err(err) => {
                eprintln!("{}", err);
                1
            }
        },
    )
}

/// pub(crate) for testing.
pub(crate) fn main_inner(
    arguments: Arguments,
    service_check: Box<dyn ServiceCheck>,
    host_check: Box<dyn HostCheck>,
) -> Result<()> {
    // load the metricdog config file
    let config = match &arguments.config {
        None => Config::new()?,
        Some(filepath) => Config::from_file(filepath)?,
    };

    // exit early with no error if the opt-out flag is set
    if !config.send_metrics {
        return Ok(());
    }

    // load bottlerocket release info
    let os_release = if let Some(os_release_path) = &arguments.os_release {
        BottlerocketRelease::from_file(os_release_path)
    } else {
        BottlerocketRelease::new()
    }
    .context(error::BottlerocketReleaseSnafu)?;

    // instantiate the metricdog object
    let metricdog = Metricdog::from_parts(config, os_release, service_check, host_check)?;

    // execute the specified command
    match arguments.command {
        Command::SendBootSuccess(_) => {
            if let Err(err) = metricdog.send_boot_success() {
                // we don't want to fail the boot if there is a failure to send this message, so
                // we log the error and return Ok(())
                error!("Error while reporting boot success: {}", err);
            }
        }
        Command::SendHealthPing(_) => {
            metricdog.send_health_ping()?;
        }
    }
    Ok(())
}
