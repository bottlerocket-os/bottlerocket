use crate::config::Config;
use crate::error::{self, Result};
use crate::host_check::HostCheck;
use crate::service_check::ServiceCheck;
use bottlerocket_release::BottlerocketRelease;
use log::{debug, error};
use reqwest::blocking::Client;
use serde::Serialize;
use snafu::ResultExt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::time::Duration;
use url::Url;

/// The send function optionally takes a timeout parameter so that we can have a short timeout for
/// `boot_success`. When `None` is passed, the default timeout is used. 20 seconds was arbitrarily
/// chosen and can be changed if the need arises.
const DEFAULT_TIMEOUT_SECONDS: u64 = 20;

/// Sends key-value pairs as query params to a URL configured in `config`. Also provides the ability
/// to check the health of a list of services and send information about whether or not the services
/// are running.
pub(crate) struct Metricdog {
    /// The `Metricdog` configuration, e.g. from `/etc/metricdog.toml`
    config: Config,
    /// Information about the Bottlerocket release, e.g. from `os-release`
    os_release: BottlerocketRelease,
    /// A trait object that checks attributes of a service (listed in `config`). This can be passed-
    /// in, but defaults to an object that uses `systemctl` to check services.
    service_check: Box<dyn ServiceCheck>,
    /// The trait checks aspects of the host such as whether it's the first boot or how long it took
    /// for the host to become ready.
    host_check: Box<dyn HostCheck>,
    /// The metrics_url, having been parsed during construction of the `Metricdog` object.
    metrics_url: Url,
}

#[derive(Serialize, Debug, Default)]
pub(crate) struct BootMetrics {
    is_first_boot: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    preconfigured_time_ms: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    configured_time_ms: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    network_ready_time_ms: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    filesystem_ready_time_ms: Option<String>,
}

#[derive(Serialize, Debug, Default)]
pub(crate) struct HealthCheck {
    is_healthy: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    failed_services: Option<String>,
}

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub(crate) enum Event {
    BootSuccess(BootMetrics),
    HealthPing(HealthCheck),
}

impl Display for Event {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Event::BootSuccess(_) => write!(f, "boot_success"),
            Event::HealthPing(_) => write!(f, "health_ping"),
        }
    }
}

impl Metricdog {
    /// Create a new instance by passing in the `Config`, `BottlerocketRelease`, and `ServiceCheck`
    /// objects.
    pub(crate) fn from_parts(
        config: Config,
        os_release: BottlerocketRelease,
        service_check: Box<dyn ServiceCheck>,
        host_check: Box<dyn HostCheck>,
    ) -> Result<Self> {
        let metrics_url = Url::from_str(&config.metrics_url).context(error::UrlParseSnafu {
            url: &config.metrics_url,
        })?;
        Ok(Self {
            config,
            os_release,
            service_check,
            host_check,
            metrics_url,
        })
    }

    /// # Description
    ///
    /// Sends key-value pairs as query parameters in a GET request to the URL in `config`. A
    /// standard set of key-value pairs are added first, and appended by any additional parameters
    /// passed in to this function.
    ///
    /// # Parameters
    ///
    /// * `sender`:          This is the name of the application sending the metrics e.g.
    ///                      `metricdog` or `updog`.
    /// * `event`:           The metrics event that is being sent. For example `BootSuccess` or
    ///                      `HealthPing`.
    /// * `timeout_seconds`: The timeout setting for the HTTP client. Defaults to
    ///                      `DEFAULT_TIMEOUT_SECONDS` when `None` is passed.
    pub(crate) fn send<S1>(
        &self,
        sender: S1,
        event: Event,
        timeout_seconds: Option<u64>,
    ) -> Result<()>
    where
        S1: AsRef<str>,
    {
        let mut url = self.metrics_url.clone();
        {
            let mut q = url.query_pairs_mut();
            q.append_pair("sender", sender.as_ref());
            q.append_pair("event", &event.to_string());
            q.append_pair("version", &self.os_release.version_id.to_string());
            q.append_pair("variant", &self.os_release.variant_id);
            q.append_pair("arch", &self.os_release.arch);
            q.append_pair("region", &self.config.region);
            q.append_pair("seed", &self.config.seed.to_string());
            q.append_pair("version_lock", &self.config.version_lock);
            q.append_pair("ignore_waves", &self.config.ignore_waves.to_string());

            let values = serde_json::value::to_value(event).unwrap_or_default();
            if let serde_json::Value::Object(obj) = values {
                for (key, val) in obj {
                    if let Some(val) = val.as_str() {
                        q.append_pair(&key, val);
                        continue;
                    }
                    if let Some(val) = val.as_bool() {
                        q.append_pair(&key, &val.to_string());
                    }
                }
            }
        }
        Self::send_get_request(url, timeout_seconds)?;
        Ok(())
    }

    /// Sends a notification to the metrics url that boot succeeded.
    pub(crate) fn send_boot_success(&self) -> Result<()> {
        let event = Event::BootSuccess(BootMetrics {
            is_first_boot: self.host_check.is_first_boot()?,
            preconfigured_time_ms: self
                .host_check
                .preconfigured_time_ms()
                .map_err(|e| error!("Unable to get preconfigured time: '{}'", e))
                .ok(),
            configured_time_ms: self
                .host_check
                .configured_time_ms()
                .map_err(|e| error!("Unable to get configured time: '{}'", e))
                .ok(),
            network_ready_time_ms: self
                .host_check
                .network_ready_time_ms()
                .map_err(|e| error!("Unable to get network ready time: '{}'", e))
                .ok(),
            filesystem_ready_time_ms: self
                .host_check
                .filesystem_ready_time_ms()
                .map_err(|e| error!("Unable to get filesystem ready time: '{}'", e))
                .ok(),
        });

        // Timeout of 3 seconds to prevent blocking the completion of mark-boot-success
        self.send("metricdog", event, Some(3))?;
        Ok(())
    }

    /// Checks the services listed in `config.service_checks` using `healthcheck`. Sends a
    /// notification to the metrics url reporting `is_healthy=true&failed_services=` if all services
    /// are healthy, or `is_healthy=false&failed_services=a:1,b:2` where `a` and `b` are the failed
    /// services, and `1` and `2` are exit codes of the failed services.
    pub(crate) fn send_health_ping(&self) -> Result<()> {
        let mut is_healthy = true;
        let mut failed_services = Vec::new();
        for service in &self.config.service_checks {
            let service_status = self.service_check.check(service)?;
            if !service_status.is_healthy {
                is_healthy = false;
                match service_status.exit_code {
                    None => failed_services.push(service.clone()),
                    Some(exit_code) => {
                        failed_services.push(format!("{}:{}", service.as_str(), exit_code))
                    }
                }
            }
        }
        // Consistent ordering of failed services could be helpful when viewing raw records.
        failed_services.sort();

        let event = Event::HealthPing(HealthCheck {
            is_healthy,
            failed_services: Some(failed_services.join(",")),
        });
        self.send("metricdog", event, None)?;
        Ok(())
    }

    fn send_get_request(url: Url, timeout_sec: Option<u64>) -> Result<()> {
        debug!("sending: {}", url.as_str());
        let client = Client::builder()
            .timeout(Duration::from_secs(
                timeout_sec.unwrap_or(DEFAULT_TIMEOUT_SECONDS),
            ))
            .build()
            .context(error::HttpClientSnafu { url: url.clone() })?;
        let response = client
            .get(url.clone())
            .send()
            .context(error::HttpSendSnafu { url: url.clone() })?;
        response
            .error_for_status()
            .context(error::HttpResponseSnafu { url })?;
        Ok(())
    }
}
