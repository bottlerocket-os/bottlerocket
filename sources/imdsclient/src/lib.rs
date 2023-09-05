/*!
`imdsclient` provides high-level methods to interact with the AWS Instance Metadata Service (IMDS).

The library uses IMDSv2 (session-oriented) requests over a pinned schema to guarantee compatibility.
Session tokens are fetched automatically and refreshed if the request receives a `401` response.
If an IMDS token fetch or query fails, the library will continue to retry with a fibonacci backoff
strategy until it is successful or times out. The default timeout is 300s to match the ifup timeout
set in wicked.service, but can configured using `.with_timeout` during client creation.

Each public method is explicitly targeted and return either bytes or a `String`.

For example, if we need a piece of metadata, like `instance_type`, a method `fetch_instance_type`,
will create an IMDSv2 session _(if one does not already exist)_ and send a request to:

`http://169.254.169.254/2021-01-03/meta-data/instance-type`

The result is returned as a `String` _(ex. m5.large)_.
*/

use std::sync::RwLock;

use http::StatusCode;
use log::{debug, info, trace, warn};
use reqwest::Client;
use snafu::{ensure, OptionExt, ResultExt};
use tokio::time::{timeout, Duration};
use tokio_retry::{strategy::FibonacciBackoff, Retry};

const BASE_URI: &str = "http://169.254.169.254";
const PINNED_SCHEMA: &str = "2021-07-15";

// Currently only able to get fetch session tokens from `latest`.
const SESSION_TARGET: &str = "latest/api/token";

// Retry timeout tied to wicked.service ifup timeout.
const RETRY_TIMEOUT_SECS: u64 = 300;

fn retry_strategy() -> impl Iterator<Item = Duration> {
    // Retry attempts at 0.25s, 0.5s, 1s, 1.75s, 3s, 5s, 8.25s, 13.5s, 22s and then every 10s after.
    FibonacciBackoff::from_millis(250).max_delay(Duration::from_secs(10))
}

/// A client for making IMDSv2 queries.
pub struct ImdsClient {
    client: Client,
    imds_base_uri: String,
    retry_timeout: Duration,
    // The token is reader-writer locked to prevent reads while it's being refreshed in retry logic.
    session_token: RwLock<Option<String>>,
}

impl Default for ImdsClient {
    fn default() -> Self {
        Self::new()
    }
}

impl ImdsClient {
    pub fn new() -> Self {
        Self::new_impl(BASE_URI.to_string())
    }

    fn new_impl(imds_base_uri: String) -> Self {
        Self {
            client: Client::new(),
            retry_timeout: Duration::from_secs(RETRY_TIMEOUT_SECS),
            session_token: RwLock::new(None),
            imds_base_uri,
        }
    }

    /// Overrides the default timeout when building your own ImdsClient.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.retry_timeout = timeout;
        self
    }

    /// Gets `user-data` from IMDS. The user-data may be either a UTF-8 string or compressed bytes.
    pub async fn fetch_userdata(&mut self) -> Result<Option<Vec<u8>>> {
        self.fetch_imds(PINNED_SCHEMA, "user-data").await
    }

    /// Gets instance identity document from IMDS.
    async fn fetch_identity_document(&mut self) -> Result<Option<serde_json::Value>> {
        let target = "dynamic/instance-identity/document";
        let response = match self.fetch_bytes(target).await? {
            Some(response) => response,
            None => return Ok(None),
        };
        serde_json::from_slice(&response)
            .context(error::SerdeSnafu)
            .map(Some)
    }

    /// Returns the region described in the identity document.
    pub async fn fetch_region(&mut self) -> Result<Option<String>> {
        Ok(self.fetch_identity_document().await?.and_then(|doc| {
            doc.get("region")
                .and_then(|value| value.as_str())
                .map(|region| region.to_string())
        }))
    }

    /// Returns the availability zone described in the identity document.
    pub async fn fetch_zone(&mut self) -> Result<Option<String>> {
        Ok(self.fetch_identity_document().await?.and_then(|doc| {
            doc.get("availabilityZone")
                .and_then(|value| value.as_str())
                .map(|az| az.to_string())
        }))
    }

    /// Returns the partition that the instance is in.
    pub async fn fetch_partition(&mut self) -> Result<Option<String>> {
        let partition_target = "meta-data/services/partition";
        self.fetch_string(&partition_target).await
    }

    /// Returns the list of network interface mac addresses.
    pub async fn fetch_mac_addresses(&mut self) -> Result<Option<Vec<String>>> {
        let macs_target = "meta-data/network/interfaces/macs";
        let macs = self
            .fetch_string(&macs_target)
            .await?
            .map(|macs| macs.lines().map(|s| s.to_string()).collect());
        Ok(macs)
    }

    /// Gets the list of CIDR blocks for a given network interface `mac` address.
    pub async fn fetch_cidr_blocks_for_mac(&mut self, mac: &str) -> Result<Option<Vec<String>>> {
        // Infer the cluster DNS based on our CIDR blocks.
        let mac_cidr_blocks_target = format!(
            "meta-data/network/interfaces/macs/{}/vpc-ipv4-cidr-blocks",
            mac
        );
        let cidr_blocks = self
            .fetch_string(&mac_cidr_blocks_target)
            .await?
            .map(|cidr_blocks| cidr_blocks.lines().map(|s| s.to_string()).collect());
        Ok(cidr_blocks)
    }

    /// Gets the local IPV4 address from instance metadata.
    pub async fn fetch_local_ipv4_address(&mut self) -> Result<Option<String>> {
        let node_ip_target = "meta-data/local-ipv4";
        self.fetch_string(&node_ip_target).await
    }

    /// Gets the IPV6 address associated with the primary network interface from instance metadata.
    pub async fn fetch_primary_ipv6_address(&mut self) -> Result<Option<String>> {
        // Get the mac address for the primary network interface.
        let mac = self
            .fetch_mac_addresses()
            .await?
            .context(error::MacAddressesSnafu)?
            .first()
            .context(error::MacAddressesSnafu)?
            .clone();

        // Get the IPv6 addresses associated with the primary network interface.
        let ipv6_address_target = format!("meta-data/network/interfaces/macs/{}/ipv6s", mac);

        let ipv6_address = self
            .fetch_string(&ipv6_address_target)
            .await?
            .and_then(|ipv6_addresses| ipv6_addresses.lines().next().map(|s| s.to_string()));
        Ok(ipv6_address)
    }

    /// Gets the instance-type from instance metadata.
    pub async fn fetch_instance_type(&mut self) -> Result<Option<String>> {
        let instance_type_target = "meta-data/instance-type";
        self.fetch_string(&instance_type_target).await
    }

    /// Gets the instance-id from instance metadata.
    pub async fn fetch_instance_id(&mut self) -> Result<Option<String>> {
        let instance_type_target = "meta-data/instance-id";
        self.fetch_string(&instance_type_target).await
    }

    /// Get lifecycle state from instance metadata.
    pub async fn fetch_autoscaling_lifecycle_state(&mut self) -> Result<Option<String>> {
        let instance_type_target = "meta-data/autoscaling/target-lifecycle-state";
        self.fetch_string(&instance_type_target).await
    }

    /// Returns a list of public ssh keys skipping any keys that do not start with 'ssh'.
    pub async fn fetch_public_ssh_keys(&mut self) -> Result<Option<Vec<String>>> {
        info!("Fetching list of available public keys from IMDS");
        // Returns a list of available public keys as '0=my-public-key'.
        let public_key_list = match self.fetch_string("meta-data/public-keys").await? {
            Some(public_key_list) => {
                debug!("available public keys '{}'", &public_key_list);
                public_key_list
            }
            None => {
                debug!("no available public keys");
                return Ok(None);
            }
        };

        debug!("available public keys '{}'", &public_key_list);
        info!("Generating targets to fetch text of available public keys");
        let public_key_targets = build_public_key_targets(&public_key_list);

        let mut public_keys = Vec::new();
        let target_count: u32 = 0;
        for target in &public_key_targets {
            let target_count = target_count + 1;
            info!(
                "Fetching public key ({}/{})",
                target_count,
                &public_key_targets.len()
            );

            let public_key_text = self
                .fetch_string(&target)
                .await?
                .context(error::KeyNotFoundSnafu { target })?;
            let public_key = public_key_text.trim_end();
            // Simple check to see if the text is probably an ssh key.
            if public_key.starts_with("ssh") {
                debug!("{}", &public_key);
                public_keys.push(public_key.to_string())
            } else {
                warn!(
                    "'{}' does not appear to be a valid key. Skipping...",
                    &public_key
                );
                continue;
            }
        }
        if public_keys.is_empty() {
            warn!("No valid keys found");
        }
        Ok(Some(public_keys))
    }

    /// Gets the hostname from instance metadata. The`metadata/local-hostname` IMDS target may
    /// potentially return multiple space-delimited hostnames; choose the first one.
    pub async fn fetch_hostname(&mut self) -> Result<Option<String>> {
        let hostname_target = "meta-data/local-hostname";
        Ok(self.fetch_string(&hostname_target).await?.and_then(|h| {
            h.split_whitespace()
                .next()
                .map(|h| h.trim_end_matches('.'))
                .map(String::from)
        }))
    }

    /// Helper to fetch bytes from IMDS using the pinned schema version.
    async fn fetch_bytes<S>(&mut self, end_target: S) -> Result<Option<Vec<u8>>>
    where
        S: AsRef<str>,
    {
        self.fetch_imds(PINNED_SCHEMA, end_target.as_ref()).await
    }

    /// Helper to fetch a string from IMDS using the pinned schema version.
    async fn fetch_string<S>(&mut self, end_target: S) -> Result<Option<String>>
    where
        S: AsRef<str>,
    {
        match self.fetch_imds(PINNED_SCHEMA, end_target).await? {
            Some(response_body) => Ok(Some(
                String::from_utf8(response_body).context(error::NonUtf8ResponseSnafu)?,
            )),
            None => Ok(None),
        }
    }

    /// Fetch data from IMDS.
    async fn fetch_imds<S1, S2>(
        &mut self,
        schema_version: S1,
        target: S2,
    ) -> Result<Option<Vec<u8>>>
    where
        S1: AsRef<str>,
        S2: AsRef<str>,
    {
        let uri = format!(
            "{}/{}/{}",
            self.imds_base_uri,
            schema_version.as_ref(),
            target.as_ref()
        );
        debug!("Requesting {}", &uri);
        timeout(
            self.retry_timeout,
            Retry::spawn(retry_strategy(), || async {
                let session_token = match self.read_token().await? {
                    Some(session_token) => session_token,
                    None => self.write_token().await?,
                };
                let response = self
                    .client
                    .get(&uri)
                    .header("X-aws-ec2-metadata-token", session_token)
                    .send()
                    .await
                    .context(error::RequestSnafu {
                        method: "GET",
                        uri: &uri,
                    })?;
                trace!("IMDS response: {:?}", &response);

                match response.status() {
                    code @ StatusCode::OK => {
                        info!("Received {}", target.as_ref());
                        let response_body = response
                            .bytes()
                            .await
                            .context(error::ResponseBodySnafu {
                                method: "GET",
                                uri: &uri,
                                code,
                            })?
                            .to_vec();

                        let response_str = printable_string(&response_body);
                        trace!("Response: {:?}", response_str);

                        Ok(Some(response_body))
                    }

                    // IMDS returns 404 if no user data is given, or if IMDS is disabled.
                    StatusCode::NOT_FOUND => Ok(None),

                    // IMDS returns 401 if the session token is expired or invalid.
                    StatusCode::UNAUTHORIZED => {
                        warn!("IMDS request unauthorized");
                        self.clear_token()?;
                        error::TokenInvalidSnafu.fail()
                    }

                    code => {
                        let response_body = response
                            .bytes()
                            .await
                            .context(error::ResponseBodySnafu {
                                method: "GET",
                                uri: &uri,
                                code,
                            })?
                            .to_vec();

                        let response_str = printable_string(&response_body);

                        trace!("Response: {:?}", response_str);

                        error::ResponseSnafu {
                            method: "GET",
                            uri: &uri,
                            code,
                            response_body: response_str,
                        }
                        .fail()
                    }
                }
            }),
        )
        .await
        .context(error::TimeoutFetchIMDSSnafu)?
    }

    /// Fetches a new session token and writes it to the current ImdsClient.
    async fn write_token(&self) -> Result<String> {
        match fetch_token(&self.client, &self.imds_base_uri, &self.retry_timeout).await? {
            Some(written_token) => {
                *self
                    .session_token
                    .write()
                    .map_err(|_| error::Error::FailedWriteToken {})? = Some(written_token.clone());
                Ok(written_token)
            }
            None => error::FailedWriteTokenSnafu.fail(),
        }
    }

    /// Clears the session token in the current ImdsClient.
    fn clear_token(&self) -> Result<()> {
        *self
            .session_token
            .write()
            .map_err(|_| error::Error::FailedClearToken {})? = None;
        Ok(())
    }

    /// Helper to read session token within the ImdsClient.
    async fn read_token(&self) -> Result<Option<String>> {
        match self
            .session_token
            .read()
            .map_err(|_| error::Error::FailedReadToken {})?
            // Cloned to release RwLock as soon as possible.
            .clone()
        {
            Some(read_token) => Ok(Some(read_token)),
            None => Ok(None),
        }
    }
}

/// Converts `bytes` to a `String` if it is a UTF-8 encoded string.
/// Truncates the string if it is too long for printing.
fn printable_string(bytes: &[u8]) -> String {
    if let Ok(s) = String::from_utf8(bytes.into()) {
        if s.len() < 2048 {
            s
        } else {
            format!("{}<truncated...>", &s[0..2034])
        }
    } else {
        "<binary>".to_string()
    }
}

/// Returns a list of public keys available in IMDS. Since IMDS returns the list of keys as
/// '0=my-public-key', we need to strip the index and insert it into the public key target.
fn build_public_key_targets(public_key_list: &str) -> Vec<String> {
    let mut public_key_targets = Vec::new();
    for available_key in public_key_list.lines() {
        let f: Vec<&str> = available_key.split('=').collect();
        // If f[0] isn't a number, then it isn't a valid index.
        if f[0].parse::<u32>().is_ok() {
            let public_key_target = format!("meta-data/public-keys/{}/openssh-key", f[0]);
            public_key_targets.push(public_key_target);
        } else {
            warn!(
                "'{}' does not appear to be a valid index. Skipping...",
                &f[0]
            );
            continue;
        }
    }
    if public_key_targets.is_empty() {
        warn!("No valid key targets found");
    }
    public_key_targets
}

/// Helper to fetch an IMDSv2 session token that is valid for 60 seconds.
async fn fetch_token(
    client: &Client,
    imds_base_uri: &str,
    retry_timeout: &Duration,
) -> Result<Option<String>> {
    let uri = format!("{}/{}", imds_base_uri, SESSION_TARGET);
    timeout(
        *retry_timeout,
        Retry::spawn(retry_strategy(), || async {
            let response = client
                .put(&uri)
                .header("X-aws-ec2-metadata-token-ttl-seconds", "60")
                .send()
                .await
                .context(error::RequestSnafu {
                    method: "PUT",
                    uri: &uri,
                })?;

            let code = response.status();
            ensure!(code == StatusCode::OK, error::FailedFetchTokenSnafu);

            let response_body = response.text().await.context(error::ResponseBodySnafu {
                method: "PUT",
                uri: &uri,
                code,
            })?;
            Ok(Some(response_body))
        }),
    )
    .await
    .context(error::TimeoutFetchTokenSnafu)?
}

mod error {
    use http::StatusCode;
    use snafu::Snafu;

    // Extracts the status code from a reqwest::Error and converts it to a string to be displayed.
    fn get_status_code(source: &reqwest::Error) -> String {
        source
            .status()
            .as_ref()
            .map(|i| i.as_str())
            .unwrap_or("Unknown")
            .to_string()
    }

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]

    // snafu doesn't yet support the lifetimes used by std::sync::PoisonError.
    pub enum Error {
        #[snafu(display("Response '{}' from '{}': {}", get_status_code(source), uri, source))]
        BadResponse { uri: String, source: reqwest::Error },

        #[snafu(display("Failed to clear token within ImdsClient"))]
        FailedClearToken,

        #[snafu(display("IMDS fetch failed after {} attempts", attempt))]
        FailedFetchIMDS { attempt: u8 },

        #[snafu(display("Failed to fetch IMDSv2 session token"))]
        FailedFetchToken,

        #[snafu(display("Failed to read token within ImdsClient"))]
        FailedReadToken,

        #[snafu(display("IMDS session failed: {}", source))]
        FailedSession { source: reqwest::Error },

        #[snafu(display("Failed to write token to ImdsClient"))]
        FailedWriteToken,

        #[snafu(display("Error retrieving key from {}", target))]
        KeyNotFound { target: String },

        #[snafu(display("No mac addresses found"))]
        MacAddresses,

        #[snafu(display("Response was not UTF-8: {}", source))]
        NonUtf8Response { source: std::string::FromUtf8Error },

        #[snafu(display("Error {}ing '{}': {}", method, uri, source))]
        Request {
            method: String,
            uri: String,
            source: reqwest::Error,
        },

        #[snafu(display("Error {} when {}ing '{}': {}", code, method, uri, response_body))]
        Response {
            method: String,
            uri: String,
            code: StatusCode,
            response_body: String,
        },

        #[snafu(display(
            "Unable to read response body when {}ing '{}' (code {}) - {}",
            method,
            uri,
            code,
            source
        ))]
        ResponseBody {
            method: String,
            uri: String,
            code: StatusCode,
            source: reqwest::Error,
        },

        #[snafu(display("Deserialization error: {}", source))]
        Serde { source: serde_json::Error },

        #[snafu(display("Timed out fetching data from IMDS: {}", source))]
        TimeoutFetchIMDS { source: tokio::time::error::Elapsed },

        #[snafu(display("Timed out fetching IMDSv2 session token: {}", source))]
        TimeoutFetchToken { source: tokio::time::error::Elapsed },

        #[snafu(display("IMDSv2 session token is invalid or expired."))]
        TokenInvalid,
    }
}

pub use error::Error;
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod test {
    use super::*;
    use httptest::{matchers::*, responders::*, Expectation, Server};

    #[tokio::test]
    async fn fetch_imds() {
        let server = Server::run();
        let base_uri = format!("http://{}", server.addr());
        let token = "some+token";
        let schema_version = "latest";
        let target = "meta-data/instance-type";
        let response_code = 200;
        let response_body = "m5.large";
        server.expect(
            Expectation::matching(request::method_path("PUT", "/latest/api/token"))
                .times(1)
                .respond_with(
                    status_code(200)
                        .append_header("X-aws-ec2-metadata-token-ttl-seconds", "60")
                        .body(token),
                ),
        );
        server.expect(
            Expectation::matching(request::method_path(
                "GET",
                format!("/{}/{}", schema_version, target),
            ))
            .times(1)
            .respond_with(
                status_code(response_code)
                    .append_header("X-aws-ec2-metadata-token", token)
                    .body(response_body),
            ),
        );
        let mut imds_client = ImdsClient::new_impl(base_uri);
        let imds_data = imds_client
            .fetch_imds(schema_version, target)
            .await
            .unwrap();
        let imds_token = imds_client.read_token().await.unwrap().unwrap();
        assert_eq!(imds_token, token);
        assert_eq!(imds_data, Some(response_body.as_bytes().to_vec()));
    }

    #[tokio::test]
    async fn fetch_imds_notfound() {
        let server = Server::run();
        let base_uri = format!("http://{}", server.addr());
        let token = "some+token";
        let schema_version = "latest";
        let target = "meta-data/instance-type";
        let response_code = 404;
        server.expect(
            Expectation::matching(request::method_path("PUT", "/latest/api/token"))
                .times(1)
                .respond_with(
                    status_code(200)
                        .append_header("X-aws-ec2-metadata-token-ttl-seconds", "60")
                        .body(token),
                ),
        );
        server.expect(
            Expectation::matching(request::method_path(
                "GET",
                format!("/{}/{}", schema_version, target),
            ))
            .times(1)
            .respond_with(
                status_code(response_code).append_header("X-aws-ec2-metadata-token", token),
            ),
        );
        let mut imds_client = ImdsClient::new_impl(base_uri);
        let imds_data = imds_client
            .fetch_imds(schema_version, target)
            .await
            .unwrap();
        assert_eq!(imds_data, None);
    }

    #[tokio::test]
    async fn fetch_imds_unauthorized() {
        let server = Server::run();
        let base_uri = format!("http://{}", server.addr());
        let token = "some+token";
        let schema_version = "latest";
        let target = "meta-data/instance-type";
        let response_code = 401;
        let retry_timeout = Duration::from_secs(2);
        server.expect(
            Expectation::matching(request::method_path("PUT", "/latest/api/token"))
                .times(2..)
                .respond_with(
                    status_code(200)
                        .append_header("X-aws-ec2-metadata-token-ttl-seconds", "60")
                        .body(token),
                ),
        );
        server.expect(
            Expectation::matching(request::method_path(
                "GET",
                format!("/{}/{}", schema_version, target),
            ))
            .times(2..)
            .respond_with(
                status_code(response_code).append_header("X-aws-ec2-metadata-token", token),
            ),
        );
        let mut imds_client = ImdsClient::new_impl(base_uri).with_timeout(retry_timeout);
        assert!(imds_client
            .fetch_imds(schema_version, target)
            .await
            .is_err());
    }

    #[tokio::test]
    async fn fetch_imds_timeout() {
        let server = Server::run();
        let base_uri = format!("http://{}", server.addr());
        let token = "some+token";
        let schema_version = "latest";
        let target = "meta-data/instance-type";
        let response_code = 408;
        let retry_timeout = Duration::from_secs(2);
        server.expect(
            Expectation::matching(request::method_path("PUT", "/latest/api/token"))
                .times(1)
                .respond_with(
                    status_code(200)
                        .append_header("X-aws-ec2-metadata-token-ttl-seconds", "60")
                        .body(token),
                ),
        );
        server.expect(
            Expectation::matching(request::method_path(
                "GET",
                format!("/{}/{}", schema_version, target),
            ))
            .times(2..)
            .respond_with(
                status_code(response_code).append_header("X-aws-ec2-metadata-token", token),
            ),
        );
        let mut imds_client = ImdsClient::new_impl(base_uri).with_timeout(retry_timeout);
        assert!(imds_client
            .fetch_imds(schema_version, target)
            .await
            .is_err());
    }

    #[tokio::test]
    async fn fetch_token_timeout() {
        let server = Server::run();
        let base_uri = format!("http://{}", server.addr());
        let retry_timeout = Duration::from_secs(2);
        let response_code = 408;
        server.expect(
            Expectation::matching(request::method_path("PUT", "/latest/api/token"))
                .times(2..)
                .respond_with(status_code(response_code)),
        );
        let client = Client::new();
        assert!(fetch_token(&client, &base_uri, &retry_timeout)
            .await
            .is_err());
    }

    #[tokio::test]
    async fn fetch_string() {
        let server = Server::run();
        let base_uri = format!("http://{}", server.addr());
        let token = "some+token";
        let end_target = "meta-data/instance-type";
        let response_code = 200;
        let response_body = "m5.large";
        server.expect(
            Expectation::matching(request::method_path("PUT", "/latest/api/token"))
                .times(1)
                .respond_with(
                    status_code(200)
                        .append_header("X-aws-ec2-metadata-token-ttl-seconds", "60")
                        .body(token),
                ),
        );
        server.expect(
            Expectation::matching(request::method_path(
                "GET",
                format!("/{}/{}", PINNED_SCHEMA, end_target),
            ))
            .times(1)
            .respond_with(
                status_code(response_code)
                    .append_header("X-aws-ec2-metadata-token", token)
                    .body(response_body),
            ),
        );
        let mut imds_client = ImdsClient::new_impl(base_uri);
        let imds_data = imds_client.fetch_string(end_target).await.unwrap();
        assert_eq!(imds_data, Some(response_body.to_string()));
    }

    #[tokio::test]
    async fn fetch_bytes() {
        let server = Server::run();
        let base_uri = format!("http://{}", server.addr());
        let token = "some+token";
        let end_target = "dynamic/instance-identity/document";
        let response_code = 200;
        let response_body = r#"{"region" : "us-west-2"}"#;
        server.expect(
            Expectation::matching(request::method_path("PUT", "/latest/api/token"))
                .times(1)
                .respond_with(
                    status_code(200)
                        .append_header("X-aws-ec2-metadata-token-ttl-seconds", "60")
                        .body(token),
                ),
        );
        server.expect(
            Expectation::matching(request::method_path(
                "GET",
                format!("/{}/{}", PINNED_SCHEMA, end_target),
            ))
            .times(1)
            .respond_with(
                status_code(response_code)
                    .append_header("X-aws-ec2-metadata-token", token)
                    .body(response_body),
            ),
        );
        let mut imds_client = ImdsClient::new_impl(base_uri);
        let imds_data = imds_client.fetch_bytes(end_target).await.unwrap();
        assert_eq!(imds_data, Some(response_body.as_bytes().to_vec()));
    }

    #[tokio::test]
    async fn fetch_userdata() {
        let server = Server::run();
        let base_uri = format!("http://{}", server.addr());
        let token = "some+token";
        let response_code = 200;
        let response_body = r#"settings.motd = "Welcome to Bottlerocket!""#;
        server.expect(
            Expectation::matching(request::method_path("PUT", "/latest/api/token"))
                .times(1)
                .respond_with(
                    status_code(200)
                        .append_header("X-aws-ec2-metadata-token-ttl-seconds", "60")
                        .body(token),
                ),
        );
        server.expect(
            Expectation::matching(request::method_path(
                "GET",
                format!("/{}/user-data", PINNED_SCHEMA),
            ))
            .times(1)
            .respond_with(
                status_code(response_code)
                    .append_header("X-aws-ec2-metadata-token", token)
                    .body(response_body),
            ),
        );
        let mut imds_client = ImdsClient::new_impl(base_uri);
        let imds_data = imds_client.fetch_userdata().await.unwrap();
        assert_eq!(imds_data, Some(response_body.as_bytes().to_vec()));
    }

    #[tokio::test]
    async fn fetch_hostname() {
        let server = Server::run();
        let base_uri = format!("http://{}", server.addr());
        let token = "some+token";
        let response_code = 200;
        let response_body =
            r#"ip-10-0-13-37.example.com. ip-10-0-13-37.eu-central-1.compute.internal"#;
        server.expect(
            Expectation::matching(request::method_path("PUT", "/latest/api/token"))
                .times(1)
                .respond_with(
                    status_code(200)
                        .append_header("X-aws-ec2-metadata-token-ttl-seconds", "60")
                        .body(token),
                ),
        );
        server.expect(
            Expectation::matching(request::method_path(
                "GET",
                format!("/{}/meta-data/local-hostname", PINNED_SCHEMA),
            ))
            .times(1)
            .respond_with(
                status_code(response_code)
                    .append_header("X-aws-ec2-metadata-token", token)
                    .body(response_body),
            ),
        );
        let mut imds_client = ImdsClient::new_impl(base_uri);
        let hostname = imds_client.fetch_hostname().await.unwrap();
        assert_eq!(hostname, Some(String::from("ip-10-0-13-37.example.com")));
    }

    #[test]
    fn printable_string_short() {
        let input = "Hello".as_bytes();
        let expected = "Hello".to_string();
        let actual = printable_string(input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn printable_string_binary() {
        let input: [u8; 5] = [0, 254, 1, 0, 4];
        let expected = "<binary>".to_string();
        let actual = printable_string(&input);
        assert_eq!(expected, actual);
    }

    #[test]
    fn printable_string_untruncated() {
        let mut input = String::new();
        for _ in 0..2047 {
            input.push('.');
        }
        let expected = input.clone();
        let actual = printable_string(input.as_bytes());
        assert_eq!(expected, actual);
    }

    #[test]
    fn printable_string_truncated() {
        let mut input = String::new();
        for _ in 0..2048 {
            input.push('.');
        }
        let mut expected = String::new();
        for _ in 0..2034 {
            expected.push('.');
        }
        expected.push_str("<truncated...>");
        let actual = printable_string(input.as_bytes());
        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_public_key_list() {
        let list = r#"0=zero
1=one
2=two"#;
        let parsed_list = build_public_key_targets(list);
        assert_eq!(3, parsed_list.len());
        assert_eq!(
            "meta-data/public-keys/0/openssh-key",
            parsed_list.get(0).unwrap()
        );
        assert_eq!(
            "meta-data/public-keys/1/openssh-key",
            parsed_list.get(1).unwrap()
        );
        assert_eq!(
            "meta-data/public-keys/2/openssh-key",
            parsed_list.get(2).unwrap()
        );
    }
}
