/*!
`imdsclient` provides high-level methods to interact with the AWS Instance Metadata Service (IMDS).

The library uses IMDSv2 (session-oriented) requests over a pinned schema to guarantee compatibility.
Session tokens are fetched automatically and refreshed if the request receives a `401` response.

Each public method is explicitly targeted and return either bytes or a `String`.

For example, if we need a piece of metadata, like `instance_type`, a method `fetch_instance_type`,
will create an IMDSv2 session _(if one does not already exist)_ and send a request to:

`http://169.254.169.254/2021-01-03/meta-data/instance-type`

The result is returned as a `String` _(ex. m5.large)_.
*/

#![deny(rust_2018_idioms)]

use http::StatusCode;
use log::{debug, info, trace, warn};
use reqwest::Client;
use serde_json::Value;
use snafu::{ensure, OptionExt, ResultExt};
use std::time::Duration;
use tokio::time;

const BASE_URI: &str = "http://169.254.169.254";
const PINNED_SCHEMA: &str = "2021-01-03";

// Currently only able to get fetch session tokens from `latest`
const SESSION_TARGET: &str = "latest/api/token";

/// A client for making IMDSv2 queries.
/// It obtains a session token when it is first instantiated and is reused between helper functions.
pub struct ImdsClient {
    client: Client,
    imds_base_uri: String,
    session_token: String,
}

impl ImdsClient {
    pub async fn new() -> Result<Self> {
        Self::new_impl(BASE_URI.to_string()).await
    }

    async fn new_impl(imds_base_uri: String) -> Result<Self> {
        let client = Client::new();
        let session_token = fetch_token(&client, &imds_base_uri).await?;
        Ok(Self {
            client,
            imds_base_uri,
            session_token,
        })
    }

    /// Gets `user-data` from IMDS. The user-data may be either a UTF-8 string or compressed bytes.
    pub async fn fetch_userdata(&mut self) -> Result<Option<Vec<u8>>> {
        self.fetch_imds(PINNED_SCHEMA, "user-data").await
    }

    /// Returns the region described in the identity document.
    pub async fn fetch_region(&mut self) -> Result<Option<String>> {
        let target = "dynamic/instance-identity/document";
        let response = match self.fetch_bytes(target).await? {
            Some(response) => response,
            None => return Ok(None),
        };
        let identity_document: Value = serde_json::from_slice(&response).context(error::Serde)?;
        let region = identity_document
            .get("region")
            .and_then(|value| value.as_str())
            .map(|region| region.to_string());
        Ok(region)
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
        // Get the mac address for the primary network interface
        let mac = self
            .fetch_mac_addresses()
            .await?
            .context(error::MacAddresses)?
            .first()
            .context(error::MacAddresses)?
            .clone();

        // Get the IPv6 addresses associated with the primary network interface
        let ipv6_address_target = format!("meta-data/network/interfaces/macs/{}/ipv6s", mac);

        let ipv6_address = self
            .fetch_string(&ipv6_address_target)
            .await?
            .map(|ipv6_addresses| ipv6_addresses.lines().next().map(|s| s.to_string()))
            .flatten();
        Ok(ipv6_address)
    }

    /// Gets the instance-type from instance metadata.
    pub async fn fetch_instance_type(&mut self) -> Result<Option<String>> {
        let instance_type_target = "meta-data/instance-type";
        self.fetch_string(&instance_type_target).await
    }

    /// Returns a list of public ssh keys skipping any keys that do not start with 'ssh'.
    pub async fn fetch_public_ssh_keys(&mut self) -> Result<Option<Vec<String>>> {
        info!("Fetching list of available public keys from IMDS");
        // Returns a list of available public keys as '0=my-public-key'
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
                .context(error::KeyNotFound { target })?;
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
                String::from_utf8(response_body).context(error::NonUtf8Response)?,
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
        let mut attempt: u8 = 0;
        let max_attempts: u8 = 3;
        loop {
            attempt += 1;
            ensure!(attempt <= max_attempts, error::FailedFetchIMDS { attempt });
            if attempt > 1 {
                time::sleep(Duration::from_secs(1)).await;
            }
            let response = self
                .client
                .get(&uri)
                .header("X-aws-ec2-metadata-token", &self.session_token)
                .send()
                .await
                .context(error::Request {
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
                        .context(error::ResponseBody {
                            method: "GET",
                            uri: &uri,
                            code,
                        })?
                        .to_vec();

                    let response_str = printable_string(&response_body);
                    trace!("Response: {:?}", response_str);

                    return Ok(Some(response_body));
                }

                // IMDS returns 404 if no user data is given, or if IMDS is disabled
                StatusCode::NOT_FOUND => return Ok(None),

                // IMDS returns 401 if the session token is expired or invalid
                StatusCode::UNAUTHORIZED => {
                    info!("Session token is invalid or expired");
                    self.refresh_token().await?;
                    info!("Refreshed session token");
                    continue;
                }

                StatusCode::REQUEST_TIMEOUT => {
                    info!("Retrying request");
                    continue;
                }

                code => {
                    let response_body = response
                        .bytes()
                        .await
                        .context(error::ResponseBody {
                            method: "GET",
                            uri: &uri,
                            code,
                        })?
                        .to_vec();

                    let response_str = printable_string(&response_body);

                    trace!("Response: {:?}", response_str);

                    return error::Response {
                        method: "GET",
                        uri: &uri,
                        code,
                        response_body: response_str,
                    }
                    .fail();
                }
            }
        }
    }

    /// Fetches a new session token and adds it to the current ImdsClient.
    async fn refresh_token(&mut self) -> Result<()> {
        self.session_token = fetch_token(&self.client, &self.imds_base_uri).await?;
        Ok(())
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
async fn fetch_token(client: &Client, imds_base_uri: &str) -> Result<String> {
    let uri = format!("{}/{}", imds_base_uri, SESSION_TARGET);
    let mut attempt: u8 = 0;
    let max_attempts: u8 = 3;
    loop {
        attempt += 1;
        ensure!(attempt <= max_attempts, error::FailedFetchToken { attempt });
        if attempt > 1 {
            time::sleep(Duration::from_secs(5)).await;
        }
        let response = client
            .put(&uri)
            .header("X-aws-ec2-metadata-token-ttl-seconds", "60")
            .send()
            .await
            .context(error::Request {
                method: "PUT",
                uri: &uri,
            })?;

        let code = response.status();
        if code == StatusCode::OK {
            return response.text().await.context(error::ResponseBody {
                method: "PUT",
                uri: &uri,
                code,
            });
        } else {
            info!("Retrying token request");
            continue;
        }
    }
}

mod error {
    use http::StatusCode;
    use snafu::Snafu;

    // Extracts the status code from a reqwest::Error and converts it to a string to be displayed
    fn get_status_code(source: &reqwest::Error) -> String {
        source
            .status()
            .as_ref()
            .map(|i| i.as_str())
            .unwrap_or("Unknown")
            .to_string()
    }

    #[derive(Debug, Snafu)]
    #[snafu(visibility = "pub(super)")]

    pub enum Error {
        #[snafu(display("Response '{}' from '{}': {}", get_status_code(source), uri, source))]
        BadResponse { uri: String, source: reqwest::Error },

        #[snafu(display("IMDS fetch failed after {} attempts", attempt))]
        FailedFetchIMDS { attempt: u8 },

        #[snafu(display("Failed to fetch IMDSv2 session token after {} attempts", attempt))]
        FailedFetchToken { attempt: u8 },

        #[snafu(display("IMDS session failed: {}", source))]
        FailedSession { source: reqwest::Error },

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
    }
}

pub use error::Error;
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod test {
    use super::*;
    use httptest::{matchers::*, responders::*, Expectation, Server};

    #[tokio::test]
    async fn new_imds_client() {
        let server = Server::run();
        let base_uri = format!("http://{}", server.addr());
        let token = "some+token";
        server.expect(
            Expectation::matching(request::method_path("PUT", "/latest/api/token"))
                .times(1)
                .respond_with(
                    status_code(200)
                        .append_header("X-aws-ec2-metadata-token-ttl-seconds", "60")
                        .body(token),
                ),
        );
        let imds_client = ImdsClient::new_impl(base_uri).await.unwrap();
        assert_eq!(imds_client.session_token, token);
    }

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
        let mut imds_client = ImdsClient::new_impl(base_uri).await.unwrap();
        let imds_data = imds_client
            .fetch_imds(schema_version, target)
            .await
            .unwrap();
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
        let mut imds_client = ImdsClient::new_impl(base_uri).await.unwrap();
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
        server.expect(
            Expectation::matching(request::method_path("PUT", "/latest/api/token"))
                .times(4)
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
            .times(3)
            .respond_with(
                status_code(response_code).append_header("X-aws-ec2-metadata-token", token),
            ),
        );
        let mut imds_client = ImdsClient::new_impl(base_uri).await.unwrap();
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
            .times(3)
            .respond_with(
                status_code(response_code).append_header("X-aws-ec2-metadata-token", token),
            ),
        );
        let mut imds_client = ImdsClient::new_impl(base_uri).await.unwrap();
        assert!(imds_client
            .fetch_imds(schema_version, target)
            .await
            .is_err());
    }

    #[tokio::test]
    async fn fetch_token_timeout() {
        let server = Server::run();
        let base_uri = format!("http://{}", server.addr());
        let response_code = 408;
        server.expect(
            Expectation::matching(request::method_path("PUT", "/latest/api/token"))
                .times(3)
                .respond_with(status_code(response_code)),
        );
        let client = Client::new();
        assert!(fetch_token(&client, &base_uri).await.is_err());
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
        let mut imds_client = ImdsClient::new_impl(base_uri).await.unwrap();
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
        let mut imds_client = ImdsClient::new_impl(base_uri).await.unwrap();
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
        let mut imds_client = ImdsClient::new_impl(base_uri).await.unwrap();
        let imds_data = imds_client.fetch_userdata().await.unwrap();
        assert_eq!(imds_data, Some(response_body.as_bytes().to_vec()));
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
