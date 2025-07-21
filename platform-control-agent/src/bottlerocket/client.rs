use anyhow::{Context, Result};
use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::{Method, Request};
use hyper_util::client::legacy::Client;
use hyperlocal::{UnixClientExt, UnixConnector, Uri as HyperlocalUri};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, info};

/// Client for interacting with the Bottlerocket Settings API
pub struct BottlerocketClient {
    /// Unix socket client for production
    unix_client: Option<Client<UnixConnector, Full<Bytes>>>,
    /// HTTP client for development/testing
    http_client: Option<reqwest::Client>,
    /// Socket path or HTTP URL
    api_endpoint: String,
    /// Whether we're using Unix sockets
    is_unix_socket: bool,
}

impl BottlerocketClient {
    /// Create a new Bottlerocket API client
    pub fn new(api_endpoint: &str) -> Result<Self> {
        let is_unix_socket = api_endpoint.starts_with("unix://");
        
        if is_unix_socket {
            // Extract socket path from unix:// URL
            let socket_path = api_endpoint
                .strip_prefix("unix://")
                .ok_or_else(|| anyhow::anyhow!("Invalid Unix socket URL"))?;
            
            // Warn if socket doesn't exist (but don't fail - it might be created later)
            if !Path::new(socket_path).exists() {
                tracing::warn!(
                    "Unix socket does not exist yet: {}. Connection attempts will fail until socket is created.",
                    socket_path
                );
            }
            
            Ok(Self {
                unix_client: Some(Client::unix()),
                http_client: None,
                api_endpoint: socket_path.to_string(),
                is_unix_socket: true,
            })
        } else {
            // Use regular HTTP client for development
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .context("Failed to build HTTP client")?;
            
            Ok(Self {
                unix_client: None,
                http_client: Some(client),
                api_endpoint: api_endpoint.to_string(),
                is_unix_socket: false,
            })
        }
    }

    /// Get current settings
    pub async fn get_settings(&self) -> Result<Settings> {
        debug!("Fetching current settings from Bottlerocket API");
        
        let response_body = if self.is_unix_socket {
            self.unix_request(Method::GET, "/settings", None).await?
        } else {
            self.http_get("/settings").await?
        };

        let settings = serde_json::from_str::<Settings>(&response_body)
            .context("Failed to parse settings response")?;

        Ok(settings)
    }

    /// Apply new settings
    pub async fn set_settings(&self, settings: &Settings) -> Result<()> {
        info!("Applying new settings to Bottlerocket");
        
        // In development mode, skip actual API call if unix socket doesn't exist
        if self.is_unix_socket && std::env::var("SKIP_UNIX_SOCKET").is_ok() {
            tracing::warn!("SKIP_UNIX_SOCKET is set, simulating successful settings apply");
            return Ok(());
        }
        
        let body = serde_json::to_string(settings)?;
        
        if self.is_unix_socket {
            self.unix_request(Method::PATCH, "/settings", Some(body)).await?;
        } else {
            self.http_patch("/settings", &body).await?;
        }

        info!("Settings applied successfully");
        Ok(())
    }

    /// Reboot the system
    pub async fn reboot(&self) -> Result<()> {
        info!("Initiating system reboot");
        
        if self.is_unix_socket {
            self.unix_request(Method::POST, "/actions/reboot", None).await?;
        } else {
            self.http_post("/actions/reboot", "").await?;
        }

        Ok(())
    }

    /// Get OS information
    pub async fn get_os_info(&self) -> Result<OsInfo> {
        debug!("Fetching OS information");
        
        // In development mode with skip flag, return mock data
        if self.is_unix_socket && std::env::var("SKIP_UNIX_SOCKET").is_ok() {
            return Ok(OsInfo {
                arch: std::env::consts::ARCH.to_string(),
                build_id: "dev-build-001".to_string(),
                pretty_name: "Bottlerocket OS (Development)".to_string(),
                variant_id: "dev-variant".to_string(),
                version_id: "1.16.0-dev".to_string(),
            });
        }
        
        let response_body = if self.is_unix_socket {
            self.unix_request(Method::GET, "/os", None).await?
        } else {
            self.http_get("/os").await?
        };

        let os_info = serde_json::from_str::<OsInfo>(&response_body)
            .context("Failed to parse OS info response")?;

        Ok(os_info)
    }

    /// Make a request over Unix socket
    async fn unix_request(
        &self,
        method: Method,
        path: &str,
        body: Option<String>,
    ) -> Result<String> {
        let client = self.unix_client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Unix client not initialized"))?;
        
        let uri: hyper::Uri = HyperlocalUri::new(&self.api_endpoint, path).into();
        
        let body_bytes = if let Some(content) = body {
            Full::new(Bytes::from(content))
        } else {
            Full::new(Bytes::new())
        };
        
        let request = Request::builder()
            .method(method)
            .uri(uri)
            .header("Content-Type", "application/json")
            .body(body_bytes)
            .context("Failed to build request")?;
        
        let mut response = client.request(request).await
            .context("Failed to send request to Unix socket")?;
        
        let status = response.status();
        
        // Collect the response body
        let mut body_data = Vec::new();
        while let Some(frame_result) = response.frame().await {
            let frame = frame_result.context("Failed to read response frame")?;
            if let Some(chunk) = frame.data_ref() {
                body_data.extend_from_slice(chunk);
            }
        }
        
        let body_str = String::from_utf8_lossy(&body_data);
        
        if !status.is_success() {
            return Err(anyhow::anyhow!(
                "Request failed with status {}: {}",
                status,
                body_str
            ));
        }
        
        Ok(body_str.to_string())
    }

    /// Make HTTP GET request (for development)
    async fn http_get(&self, path: &str) -> Result<String> {
        let client = self.http_client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("HTTP client not initialized"))?;
        
        let url = format!("{}{}", self.api_endpoint, path);
        let response = client.get(&url).send().await
            .context("Failed to send HTTP request")?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Request failed with status: {}",
                error_text
            ));
        }
        
        response.text().await.context("Failed to read response body")
    }

    /// Make HTTP PATCH request (for development)
    async fn http_patch(&self, path: &str, body: &str) -> Result<String> {
        let client = self.http_client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("HTTP client not initialized"))?;
        
        let url = format!("{}{}", self.api_endpoint, path);
        let response = client.patch(&url)
            .header("Content-Type", "application/json")
            .body(body.to_string())
            .send().await
            .context("Failed to send HTTP request")?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Request failed with status: {}",
                error_text
            ));
        }
        
        response.text().await.context("Failed to read response body")
    }

    /// Make HTTP POST request (for development)
    async fn http_post(&self, path: &str, body: &str) -> Result<String> {
        let client = self.http_client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("HTTP client not initialized"))?;
        
        let url = format!("{}{}", self.api_endpoint, path);
        let response = client.post(&url)
            .header("Content-Type", "application/json")
            .body(body.to_string())
            .send().await
            .context("Failed to send HTTP request")?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Request failed with status: {}",
                error_text
            ));
        }
        
        response.text().await.context("Failed to read response body")
    }
}

/// Bottlerocket settings structure
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Settings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub motd: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kubernetes: Option<KubernetesSettings>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<NetworkSettings>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kernel: Option<KernelSettings>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_containers: Option<HashMap<String, HostContainer>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ntp: Option<NtpSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KubernetesSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_server: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cluster_certificate: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cluster_dns_ip: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cluster_domain: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_labels: Option<HashMap<String, String>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_taints: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hosts: Option<HashMap<String, Vec<String>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lockdown: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sysctl: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostContainer {
    pub enabled: bool,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub superpowered: Option<bool>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_data: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NtpSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_servers: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsInfo {
    pub arch: String,
    pub build_id: String,
    pub pretty_name: String,
    pub variant_id: String,
    pub version_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_serialization() {
        let settings = Settings::default();
        let json = serde_json::to_string(&settings).unwrap();
        assert!(json == "{}");
        
        let settings_with_motd = Settings {
            motd: Some("Hello Bottlerocket".to_string()),
            ..Default::default()
        };
        let json = serde_json::to_string(&settings_with_motd).unwrap();
        assert!(json.contains("motd"));
        assert!(json.contains("Hello Bottlerocket"));
    }

    #[test]
    fn test_kubernetes_settings() {
        let k8s = KubernetesSettings {
            api_server: Some("https://k8s.example.com:6443".to_string()),
            cluster_certificate: Some("base64-cert".to_string()),
            cluster_dns_ip: Some("10.96.0.10".to_string()),
            cluster_domain: Some("cluster.local".to_string()),
            node_labels: Some(HashMap::new()),
            node_taints: Some(HashMap::new()),
        };
        
        let json = serde_json::to_string(&k8s).unwrap();
        assert!(json.contains("api_server"));
        assert!(json.contains("https://k8s.example.com:6443"));
        
        // Test deserialization
        let deserialized: KubernetesSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.api_server, k8s.api_server);
    }

    #[test]
    fn test_os_info_deserialization() {
        let json = r#"{
            "arch": "x86_64",
            "build_id": "test-build-123",
            "pretty_name": "Bottlerocket OS 1.16.0",
            "variant_id": "aws-k8s-1.28",
            "version_id": "1.16.0"
        }"#;
        
        let os_info: OsInfo = serde_json::from_str(json).unwrap();
        assert_eq!(os_info.arch, "x86_64");
        assert_eq!(os_info.build_id, "test-build-123");
        assert_eq!(os_info.variant_id, "aws-k8s-1.28");
    }

    #[test]
    fn test_host_container_settings() {
        let mut containers = HashMap::new();
        containers.insert(
            "admin".to_string(),
            HostContainer {
                enabled: true,
                source: Some("public.ecr.aws/bottlerocket/admin:latest".to_string()),
                superpowered: Some(true),
                user_data: None,
            },
        );
        
        let settings = Settings {
            host_containers: Some(containers),
            ..Default::default()
        };
        
        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("host_containers"));
        assert!(json.contains("admin"));
        assert!(json.contains("superpowered"));
    }

    #[test]
    fn test_client_creation() {
        // Test HTTP URL
        let client = BottlerocketClient::new("http://localhost:8080").unwrap();
        assert!(!client.is_unix_socket);
        assert_eq!(client.api_endpoint, "http://localhost:8080");
        
        // Test Unix socket URL (now returns Ok even if socket doesn't exist)
        let result = BottlerocketClient::new("unix:///run/api.sock");
        assert!(result.is_ok());
        let client = result.unwrap();
        assert!(client.is_unix_socket);
        assert_eq!(client.api_endpoint, "/run/api.sock");
    }

    #[test]
    fn test_unix_socket_path_extraction() {
        // Test extracting path from unix:// URL
        let url = "unix:///run/api.sock";
        let path = url.strip_prefix("unix://").unwrap();
        assert_eq!(path, "/run/api.sock");
    }
}