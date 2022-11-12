# metricdog

Current version: 0.1.0

## Introduction

Metricdog sends anonymous information about the health of a Bottlerocket host.
It does so by sending key-value pairs as query params in an HTTP GET request.

Metricdog also has the ability to check that a list of critical services is running.
It does so using `systemctl` and reports services that are not healthy.

#### Proxy Support

Metricdog respects the environment variables `HTTPS_PROXY` and `NO_PROXY` to determine whether or
not its traffic should be proxied. These are set with the `network.http-proxy` and `network.noproxy`
settings when Metricdog is invoked by systemd. If you run Metricdog manually, you would need to
seed the environment with these variables manually.

## What it Sends

#### The standard set of metrics:

* `sender`: the application sending the report.
* `event`: the event that invoked the report.
* `version`: the Bottlerocket version.
* `variant`: the Bottlerocket variant.
* `arch`: the machine architecture, e.g.'x86_64' or 'aarch64'.
* `region`: the region the machine is running in.
* `seed`: the seed value used to roll-out updates.
* `version_lock`: the optional setting that controls Bottlerocket update selection.
* `ignore_waves`: an update setting that allows hosts to update before their seed is reached.

#### Additionally, when `metricdog` sends a 'health ping', it adds:

* `is_healthy`: true or false based on whether critical services are running.
* `failed_services`: a list of critical services that have failed, if any.

## Configuration

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

## Colophon

This text was generated from `README.tpl` using [cargo-readme](https://crates.io/crates/cargo-readme), and includes the rustdoc from `src/main.rs`.
