use anyhow::{Context, Result};
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

/// Client for interacting with the Bottlerocket Settings API
pub struct BottlerocketClient {
    client: Client,
    base_url: Url,
}

impl BottlerocketClient {
    /// Create a new Bottlerocket API client
    pub fn new(api_url: &str) -> Result<Self> {
        let base_url = if api_url.starts_with("unix://") {
            // For Unix socket, we'll use a placeholder URL
            // In production, we'd use a Unix socket-aware HTTP client
            Url::parse("http://localhost/")
                .context("Failed to parse Unix socket URL")?
        } else {
            Url::parse(api_url).context("Failed to parse API URL")?
        };

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self { client, base_url })
    }

    /// Get current settings
    pub async fn get_settings(&self) -> Result<Settings> {
        debug!("Fetching current settings from Bottlerocket API");
        
        let url = self.base_url.join("settings")?;
        let response = self.client
            .get(url)
            .send()
            .await
            .context("Failed to fetch settings")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to get settings: {}",
                response.status()
            ));
        }

        let settings = response
            .json::<Settings>()
            .await
            .context("Failed to parse settings response")?;

        Ok(settings)
    }

    /// Apply new settings
    pub async fn set_settings(&self, settings: &Settings) -> Result<()> {
        info!("Applying new settings to Bottlerocket");
        
        let url = self.base_url.join("settings")?;
        let response = self.client
            .patch(url)
            .json(settings)
            .send()
            .await
            .context("Failed to apply settings")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Failed to apply settings: {}",
                error_text
            ));
        }

        info!("Settings applied successfully");
        Ok(())
    }

    /// Reboot the system
    pub async fn reboot(&self) -> Result<()> {
        info!("Initiating system reboot");
        
        let url = self.base_url.join("actions/reboot")?;
        let response = self.client
            .post(url)
            .send()
            .await
            .context("Failed to initiate reboot")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to reboot: {}",
                response.status()
            ));
        }

        Ok(())
    }

    /// Get OS information
    pub async fn get_os_info(&self) -> Result<OsInfo> {
        debug!("Fetching OS information");
        
        let url = self.base_url.join("os")?;
        let response = self.client
            .get(url)
            .send()
            .await
            .context("Failed to fetch OS info")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Failed to get OS info: {}",
                response.status()
            ));
        }

        let os_info = response
            .json::<OsInfo>()
            .await
            .context("Failed to parse OS info response")?;

        Ok(os_info)
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
        assert_eq!(client.base_url.as_str(), "http://localhost:8080/");
        
        // Test Unix socket URL (placeholder)
        let client = BottlerocketClient::new("unix:///run/api.sock").unwrap();
        assert_eq!(client.base_url.as_str(), "http://localhost/");
    }
}