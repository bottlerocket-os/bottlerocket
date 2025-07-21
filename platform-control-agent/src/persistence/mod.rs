use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::api::MachineConfig;

/// Default state directory
pub const STATE_DIR: &str = "/var/lib/platform";
const CONFIG_FILE: &str = "config.json";
const BACKUP_FILE: &str = "config.json.backup";
const TEMP_FILE: &str = "config.json.tmp";

/// Versioned configuration for future migrations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedConfig {
    /// Schema version for migrations
    version: u32,
    /// Timestamp of last update
    updated_at: chrono::DateTime<chrono::Utc>,
    /// The actual machine configuration
    config: MachineConfig,
}

impl PersistedConfig {
    const CURRENT_VERSION: u32 = 1;
    
    fn new(config: MachineConfig) -> Self {
        Self {
            version: Self::CURRENT_VERSION,
            updated_at: chrono::Utc::now(),
            config,
        }
    }
}

/// State persistence manager
pub struct StateManager {
    state_dir: PathBuf,
    pub current_config: Arc<RwLock<Option<MachineConfig>>>,
}

impl StateManager {
    /// Create a new state manager
    pub fn new(state_dir: Option<&str>, current_config: Arc<RwLock<Option<MachineConfig>>>) -> Result<Self> {
        let state_dir = PathBuf::from(state_dir.unwrap_or(STATE_DIR));
        
        // Create state directory if it doesn't exist
        if !state_dir.exists() {
            info!("Creating state directory: {}", state_dir.display());
            fs::create_dir_all(&state_dir)
                .with_context(|| format!("Failed to create state directory: {}", state_dir.display()))?;
        }
        
        Ok(Self {
            state_dir,
            current_config,
        })
    }
    
    /// Save configuration to disk atomically
    pub async fn save_config(&self, config: &MachineConfig) -> Result<()> {
        info!("Persisting configuration to disk");
        
        let config_path = self.state_dir.join(CONFIG_FILE);
        let backup_path = self.state_dir.join(BACKUP_FILE);
        let temp_path = self.state_dir.join(TEMP_FILE);
        
        // Create versioned config
        let persisted = PersistedConfig::new(config.clone());
        
        // Serialize to JSON with pretty printing
        let json = serde_json::to_string_pretty(&persisted)
            .context("Failed to serialize configuration")?;
        
        // Write to temporary file first
        debug!("Writing to temporary file: {}", temp_path.display());
        fs::write(&temp_path, &json)
            .with_context(|| format!("Failed to write temporary file: {}", temp_path.display()))?;
        
        // Sync to ensure data is on disk
        let temp_file = fs::OpenOptions::new()
            .read(true)
            .open(&temp_path)
            .context("Failed to open temporary file for sync")?;
        temp_file.sync_all()
            .context("Failed to sync temporary file to disk")?;
        drop(temp_file);
        
        // Backup existing config if it exists
        if config_path.exists() {
            debug!("Backing up existing configuration");
            fs::rename(&config_path, &backup_path)
                .with_context(|| format!("Failed to create backup: {}", backup_path.display()))?;
        }
        
        // Atomic rename
        fs::rename(&temp_path, &config_path)
            .with_context(|| format!("Failed to rename temporary file to: {}", config_path.display()))?;
        
        // Update in-memory state
        let mut current = self.current_config.write().await;
        *current = Some(config.clone());
        
        info!("Configuration persisted successfully");
        Ok(())
    }
    
    /// Load configuration from disk
    pub async fn load_config(&self) -> Result<Option<MachineConfig>> {
        let config_path = self.state_dir.join(CONFIG_FILE);
        
        if !config_path.exists() {
            info!("No saved configuration found");
            return Ok(None);
        }
        
        info!("Loading configuration from: {}", config_path.display());
        
        // Try to load primary config
        match self.load_from_file(&config_path).await {
            Ok(config) => {
                info!("Configuration loaded successfully");
                
                // Update in-memory state
                let mut current = self.current_config.write().await;
                *current = Some(config.clone());
                
                Ok(Some(config))
            }
            Err(e) => {
                error!("Failed to load primary configuration: {}", e);
                
                // Try backup if primary fails
                let backup_path = self.state_dir.join(BACKUP_FILE);
                if backup_path.exists() {
                    warn!("Attempting to load backup configuration");
                    match self.load_from_file(&backup_path).await {
                        Ok(config) => {
                            warn!("Loaded configuration from backup");
                            
                            // Save as primary
                            if let Err(e) = self.save_config(&config).await {
                                error!("Failed to restore backup as primary: {}", e);
                            }
                            
                            Ok(Some(config))
                        }
                        Err(e) => {
                            error!("Failed to load backup configuration: {}", e);
                            Err(anyhow::anyhow!("Both primary and backup configurations are corrupted"))
                        }
                    }
                } else {
                    Err(e)
                }
            }
        }
    }
    
    /// Load configuration from a specific file
    async fn load_from_file(&self, path: &Path) -> Result<MachineConfig> {
        let contents = fs::read_to_string(path)
            .with_context(|| format!("Failed to read configuration file: {}", path.display()))?;
        
        let persisted: PersistedConfig = serde_json::from_str(&contents)
            .with_context(|| format!("Failed to parse configuration from: {}", path.display()))?;
        
        // Check version for future migration support
        if persisted.version > PersistedConfig::CURRENT_VERSION {
            return Err(anyhow::anyhow!(
                "Configuration version {} is newer than supported version {}",
                persisted.version,
                PersistedConfig::CURRENT_VERSION
            ));
        }
        
        // Future: Add migration logic here if version < CURRENT_VERSION
        
        Ok(persisted.config)
    }
    
    /// Get the current configuration path
    pub fn config_path(&self) -> PathBuf {
        self.state_dir.join(CONFIG_FILE)
    }

    /// Health check for state persistence
    pub async fn health_check(&self) -> Result<()> {
        // Check if state directory is accessible
        if !self.state_dir.exists() {
            return Err(anyhow::anyhow!("State directory does not exist"));
        }

        // Try to write a test file
        let test_file = self.state_dir.join(".health_check");
        std::fs::write(&test_file, "ok")?;
        std::fs::remove_file(&test_file)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::collections::HashMap;
    
    #[tokio::test]
    async fn test_save_and_load_config() {
        // Create temporary directory
        let temp_dir = TempDir::new().unwrap();
        let current_config = Arc::new(RwLock::new(None));
        
        // Create state manager
        let manager = StateManager::new(
            Some(temp_dir.path().to_str().unwrap()),
            current_config.clone()
        ).unwrap();
        
        // Create test config
        let config = MachineConfig {
            version: "1.0.0".to_string(),
            r#type: 1, // MACHINE_TYPE_CONTROL_PLANE
            cluster: Some(crate::api::machine_config::Cluster {
                name: "test-cluster".to_string(),
                endpoint: "https://test.example.com:6443".to_string(),
                ca_certificate: "test-ca-cert".to_string(),
                bootstrap_token: String::new(),
                dns_ip: "10.96.0.10".to_string(),
                dns_domain: "cluster.local".to_string(),
            }),
            network: None,
            security: None,
            kubernetes: None,
            host_containers: HashMap::new(),
            storage: None,
        };
        
        // Save config
        manager.save_config(&config).await.unwrap();
        
        // Verify file exists
        assert!(manager.config_path().exists());
        
        // Load config
        let loaded = manager.load_config().await.unwrap();
        assert!(loaded.is_some());
        
        let loaded_config = loaded.unwrap();
        assert_eq!(loaded_config.version, config.version);
        assert_eq!(loaded_config.cluster.as_ref().unwrap().name, "test-cluster");
    }
    
    #[tokio::test]
    async fn test_backup_on_save() {
        let temp_dir = TempDir::new().unwrap();
        let current_config = Arc::new(RwLock::new(None));
        let manager = StateManager::new(
            Some(temp_dir.path().to_str().unwrap()),
            current_config.clone()
        ).unwrap();
        
        // Save first config
        let config1 = MachineConfig {
            version: "1.0.0".to_string(),
            r#type: 0,
            cluster: None,
            network: None,
            security: None,
            kubernetes: None,
            host_containers: HashMap::new(),
            storage: None,
        };
        manager.save_config(&config1).await.unwrap();
        
        // Save second config
        let config2 = MachineConfig {
            version: "2.0.0".to_string(),
            r#type: 0,
            cluster: None,
            network: None,
            security: None,
            kubernetes: None,
            host_containers: HashMap::new(),
            storage: None,
        };
        manager.save_config(&config2).await.unwrap();
        
        // Check backup exists
        let backup_path = temp_dir.path().join(BACKUP_FILE);
        assert!(backup_path.exists());
        
        // Load backup and verify it's the first config
        let backup_contents = fs::read_to_string(backup_path).unwrap();
        let backup_persisted: PersistedConfig = serde_json::from_str(&backup_contents).unwrap();
        assert_eq!(backup_persisted.config.version, "1.0.0");
    }
    
    #[tokio::test]
    async fn test_load_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let current_config = Arc::new(RwLock::new(None));
        let manager = StateManager::new(
            Some(temp_dir.path().to_str().unwrap()),
            current_config
        ).unwrap();
        
        let result = manager.load_config().await.unwrap();
        assert!(result.is_none());
    }
}