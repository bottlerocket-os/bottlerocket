use crate::config::Config;
use crate::error::{self, Result};
use crate::service_check::ServiceCheck;
use bottlerocket_release::BottlerocketRelease;
use log::debug;
use reqwest::blocking::Client;
use snafu::ResultExt;
use std::collections::HashMap;
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
    /// A trait object that checks if a service (listed in `config`) is healthy. This can be passed-
    /// in, but defaults to an object that uses `systemctl` to check services.
    healthcheck: Box<dyn ServiceCheck>,
    /// The metrics_url, having been parsed during construction of the `Metricdog` object.
    metrics_url: Url,
}

impl Metricdog {
    /// Create a new instance by passing in the `Config`, `BottlerocketRelease`, and `ServiceCheck`
    /// objects.
    pub(crate) fn from_parts(
        config: Config,
        os_release: BottlerocketRelease,
        healthcheck: Box<dyn ServiceCheck>,
    ) -> Result<Self> {
        let metrics_url = Url::from_str(&config.metrics_url).context(error::UrlParse {
            url: &config.metrics_url,
        })?;
        Ok(Self {
            config,
            os_release,
            healthcheck,
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
    /// * `event`:           The name of the type of metrics event that is being sent. For example
    ///                      `boot_success` or `health_ping`.
    /// * `values`:          The key-value pairs that you want to send. These will be sorted by key
    ///                      before sending to ensure consistency of key-value ordering.
    /// * `timeout_seconds`: The timeout setting for the HTTP client. Defaults to
    ///                      `DEFAULT_TIMEOUT_SECONDS` when `None` is passed.
    pub(crate) fn send<S1, S2>(
        &self,
        sender: S1,
        event: S2,
        values: Option<&HashMap<String, String>>,
        timeout_seconds: Option<u64>,
    ) -> Result<()>
    where
        S1: AsRef<str>,
        S2: AsRef<str>,
    {
        let mut url = self.metrics_url.clone();
        {
            let mut q = url.query_pairs_mut();
            q.append_pair("sender", sender.as_ref());
            q.append_pair("event", event.as_ref());
            q.append_pair("version", &self.os_release.version_id.to_string());
            q.append_pair("variant", &self.os_release.variant_id);
            q.append_pair("arch", &self.os_release.arch);
            q.append_pair("region", &self.config.region);
            q.append_pair("seed", &self.config.seed.to_string());
            q.append_pair("version_lock", &self.config.version_lock);
            q.append_pair("ignore_waves", &self.config.ignore_waves.to_string());
            if let Some(map) = values {
                let mut keys: Vec<&String> = map.keys().collect();
                // sorted for consistency
                keys.sort();
                for key in keys {
                    if let Some(val) = map.get(key) {
                        q.append_pair(key, val);
                    }
                }
            }
        }
        Self::send_get_request(url, timeout_seconds)?;
        Ok(())
    }

    /// Sends a notification to the metrics url that boot succeeded.
    pub(crate) fn send_boot_success(&self) -> Result<()> {
        // timeout of 3 seconds to prevent blocking the completion of mark-boot-success
        self.send("metricdog", "boot_success", None, Some(3))?;
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
            let service_status = self.healthcheck.check(service)?;
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
        let mut values = HashMap::new();
        values.insert(String::from("is_healthy"), format!("{}", is_healthy));
        // consistent ordering of failed services could be helpful when viewing raw records.
        failed_services.sort();
        values.insert(String::from("failed_services"), failed_services.join(","));
        self.send("metricdog", "health_ping", Some(&values), None)?;
        Ok(())
    }

    fn send_get_request(url: Url, timeout_sec: Option<u64>) -> Result<()> {
        debug!("sending: {}", url.as_str());
        let client = Client::builder()
            .timeout(Duration::from_secs(
                timeout_sec.unwrap_or(DEFAULT_TIMEOUT_SECONDS),
            ))
            .build()
            .context(error::HttpClient { url: url.clone() })?;
        let response = client
            .get(url.clone())
            .send()
            .context(error::HttpSend { url: url.clone() })?;
        response
            .error_for_status()
            .context(error::HttpResponse { url })?;
        Ok(())
    }
}
