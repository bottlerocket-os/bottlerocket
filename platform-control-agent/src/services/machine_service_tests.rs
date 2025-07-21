#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::api::{*, machine_service_server::MachineService as ServiceTrait};
    use crate::bottlerocket::client::BottlerocketClient;
    use crate::persistence::StateManager;
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use tonic::{Request, Code};
    use mockall::mock;
    use std::collections::HashMap;

    // Mock for BottlerocketClient
    mock! {
        BRClient {
            fn new(url: &str) -> Result<Self>;
            fn get_settings(&self) -> impl std::future::Future<Output = Result<crate::bottlerocket::client::Settings>> + Send;
            fn set_settings(&self, settings: &crate::bottlerocket::client::Settings) -> impl std::future::Future<Output = Result<()>> + Send;
            fn commit(&self) -> impl std::future::Future<Output = Result<()>> + Send;
            fn apply(&self) -> impl std::future::Future<Output = Result<()>> + Send;
            fn reboot(&self) -> impl std::future::Future<Output = Result<()>> + Send;
        }
        
        impl Clone for BRClient {
            fn clone(&self) -> Self;
        }
    }

    fn create_test_service() -> (MachineServiceImpl, Arc<RwLock<Option<MachineConfig>>>) {
        let current_config = Arc::new(RwLock::new(None));
        let br_client = Arc::new(BottlerocketClient::new("http://test").unwrap());
        let state_manager = Arc::new(StateManager::new(
            Some("./test_state"),
            current_config.clone(),
        ).unwrap());
        
        let service = MachineServiceImpl::new(br_client, state_manager);
        (service, current_config)
    }

    #[tokio::test]
    async fn test_apply_configuration_success() {
        let (service, current_config) = create_test_service();
        
        let config = MachineConfig {
            version: "v1.0.0".to_string(),
            r#type: 1, // Worker
            cluster: Some(machine_config::Cluster {
                name: "test-cluster".to_string(),
                endpoint: "https://k8s.example.com:6443".to_string(),
                ca_certificate: "test-cert".to_string(),
                bootstrap_token: "test-token".to_string(),
                dns_ip: "10.96.0.10".to_string(),
                dns_domain: "cluster.local".to_string(),
            }),
            network: None,
            security: None,
            kubernetes: None,
            host_containers: HashMap::new(),
            storage: None,
        };
        
        let request = Request::new(ApplyConfigurationRequest {
            config: Some(config.clone()),
        });
        
        let response = service.apply_configuration(request).await;
        assert!(response.is_ok());
        
        let resp = response.unwrap().into_inner();
        assert!(resp.success);
        assert_eq!(resp.message, "Configuration applied successfully");
        
        // Verify config was saved
        let saved_config = current_config.read().await;
        assert!(saved_config.is_some());
        assert_eq!(saved_config.as_ref().unwrap().version, "v1.0.0");
    }

    #[tokio::test]
    async fn test_apply_configuration_no_config() {
        let (service, _) = create_test_service();
        
        let request = Request::new(ApplyConfigurationRequest {
            config: None,
        });
        
        let response = service.apply_configuration(request).await;
        assert!(response.is_err());
        assert_eq!(response.unwrap_err().code(), Code::InvalidArgument);
    }

    #[tokio::test]
    async fn test_get_configuration() {
        let (service, current_config) = create_test_service();
        
        // Set a config
        let config = MachineConfig {
            version: "v1.0.0".to_string(),
            r#type: 1,
            cluster: None,
            network: None,
            security: None,
            kubernetes: None,
            host_containers: HashMap::new(),
            storage: None,
        };
        *current_config.write().await = Some(config.clone());
        
        let request = Request::new(GetConfigurationRequest {});
        let response = service.get_configuration(request).await;
        
        assert!(response.is_ok());
        let resp = response.unwrap().into_inner();
        assert_eq!(resp.config.unwrap().version, "v1.0.0");
    }

    #[tokio::test]
    async fn test_get_configuration_not_configured() {
        let (service, _) = create_test_service();
        
        let request = Request::new(GetConfigurationRequest {});
        let response = service.get_configuration(request).await;
        
        assert!(response.is_err());
        assert_eq!(response.unwrap_err().code(), Code::NotFound);
    }

    #[tokio::test]
    async fn test_get_status() {
        let (service, current_config) = create_test_service();
        
        // Set a config
        let config = MachineConfig {
            version: "v1.0.0".to_string(),
            r#type: 1,
            cluster: None,
            network: None,
            security: None,
            kubernetes: None,
            host_containers: HashMap::new(),
            storage: None,
        };
        *current_config.write().await = Some(config);
        
        let request = Request::new(GetStatusRequest {});
        let response = service.get_status(request).await;
        
        assert!(response.is_ok());
        let status = response.unwrap().into_inner();
        assert_eq!(status.state, machine_status::State::Configured as i32);
        assert!(!status.machine_id.is_empty());
        assert!(status.uptime_seconds > 0);
        assert!(status.resources.is_some());
    }

    #[tokio::test]
    async fn test_reset() {
        let (service, current_config) = create_test_service();
        
        // Set a config
        let config = MachineConfig {
            version: "v1.0.0".to_string(),
            r#type: 1,
            cluster: None,
            network: None,
            security: None,
            kubernetes: None,
            host_containers: HashMap::new(),
            storage: None,
        };
        *current_config.write().await = Some(config);
        
        let request = Request::new(ResetRequest {
            graceful: true,
            clear_data: true,
        });
        
        let response = service.reset(request).await;
        assert!(response.is_ok());
        
        let resp = response.unwrap().into_inner();
        assert!(resp.success);
        
        // Verify config was cleared
        let saved_config = current_config.read().await;
        assert!(saved_config.is_none());
    }

    #[tokio::test]
    async fn test_reboot() {
        let (service, _) = create_test_service();
        
        let request = Request::new(RebootRequest {
            graceful: true,
            delay_seconds: 5,
        });
        
        let response = service.reboot(request).await;
        assert!(response.is_ok());
        
        let resp = response.unwrap().into_inner();
        assert!(resp.success);
        assert_eq!(resp.message, "Reboot scheduled");
    }

    #[tokio::test]
    async fn test_upgrade() {
        let (service, _) = create_test_service();
        
        let request = Request::new(UpgradeRequest {
            target_version: "v2.0.0".to_string(),
            force: false,
        });
        
        // This will fail in dev mode, but that's expected
        let response = service.upgrade(request).await;
        assert!(response.is_err());
    }

    #[tokio::test]
    async fn test_stream_events() {
        let (service, _) = create_test_service();
        
        // Initialize event system for testing
        let _ = crate::events::EventSystem::init(Some("./test_events")).await;
        
        // Publish some test events
        crate::events::publish_event(
            crate::events::EventType::SystemStartup,
            crate::events::EventData::SystemLifecycle {
                action: "test".to_string(),
                reason: None,
            },
        );
        
        let request = Request::new(StreamEventsRequest {
            filters: Some(EventFilters {
                event_types: vec!["SystemStartup".to_string()],
                since_timestamp: None,
            }),
        });
        
        // We can't easily test streaming in unit tests, but we can verify the method doesn't panic
        let response = service.stream_events(request);
        assert!(response.is_ok());
        
        // Clean up
        std::fs::remove_dir_all("./test_events").ok();
    }

    #[tokio::test]
    async fn test_validate_configuration() {
        let (service, _) = create_test_service();
        
        // Valid config
        let config = MachineConfig {
            version: "v1.0.0".to_string(),
            r#type: 1,
            cluster: Some(machine_config::Cluster {
                name: "test-cluster".to_string(),
                endpoint: "https://k8s.example.com:6443".to_string(),
                ca_certificate: "test-cert".to_string(),
                bootstrap_token: "test-token".to_string(),
                dns_ip: "10.96.0.10".to_string(),
                dns_domain: "cluster.local".to_string(),
            }),
            network: None,
            security: None,
            kubernetes: None,
            host_containers: HashMap::new(),
            storage: None,
        };
        
        let request = Request::new(ValidateConfigurationRequest {
            config: Some(config),
        });
        
        let response = service.validate_configuration(request).await;
        assert!(response.is_ok());
        
        let resp = response.unwrap().into_inner();
        assert!(resp.valid);
        assert!(resp.errors.is_empty());
    }

    #[tokio::test]
    async fn test_validate_configuration_invalid() {
        let (service, _) = create_test_service();
        
        // Invalid config - missing required cluster fields
        let config = MachineConfig {
            version: "v1.0.0".to_string(),
            r#type: 1,
            cluster: Some(machine_config::Cluster {
                name: "".to_string(), // Invalid: empty name
                endpoint: "not-a-url".to_string(), // Invalid: not a URL
                ca_certificate: "".to_string(), // Invalid: empty cert
                bootstrap_token: "".to_string(),
                dns_ip: "invalid-ip".to_string(), // Invalid: not an IP
                dns_domain: "".to_string(),
            }),
            network: None,
            security: None,
            kubernetes: None,
            host_containers: HashMap::new(),
            storage: None,
        };
        
        let request = Request::new(ValidateConfigurationRequest {
            config: Some(config),
        });
        
        let response = service.validate_configuration(request).await;
        assert!(response.is_ok());
        
        let resp = response.unwrap().into_inner();
        assert!(!resp.valid);
        assert!(!resp.errors.is_empty());
    }

    // Cleanup helper
    impl Drop for MachineServiceImpl {
        fn drop(&mut self) {
            // Clean up test directories
            std::fs::remove_dir_all("./test_state").ok();
        }
    }
}