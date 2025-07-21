use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;

/// FIPS-compliant etcd configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EtcdConfig {
    /// etcd version (e.g., "3.5.10")
    pub version: String,
    
    /// Node configuration
    pub node: NodeConfig,
    
    /// Cluster configuration
    pub cluster: ClusterConfig,
    
    /// Security configuration (FIPS-compliant)
    pub security: SecurityConfig,
    
    /// Storage configuration
    pub storage: StorageConfig,
    
    /// Performance tuning
    pub performance: PerformanceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    /// Unique node ID
    pub id: String,
    
    /// Node name (hostname)
    pub name: String,
    
    /// IP address for peer communication
    pub peer_address: IpAddr,
    
    /// Port for peer communication (default: 2380)
    pub peer_port: u16,
    
    /// IP address for client communication
    pub client_address: IpAddr,
    
    /// Port for client communication (default: 2379)
    pub client_port: u16,
    
    /// Initial cluster state ("new" or "existing")
    pub initial_cluster_state: String,
    
    /// Data directory path
    pub data_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterConfig {
    /// Cluster token for initial bootstrap
    pub cluster_token: String,
    
    /// Initial cluster members (name=peer_url)
    pub initial_cluster: HashMap<String, String>,
    
    /// Enable auto-compaction
    pub auto_compaction_mode: String,
    
    /// Auto-compaction retention (e.g., "24h")
    pub auto_compaction_retention: String,
    
    /// Quota backend bytes
    pub quota_backend_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable TLS for peer communication
    pub peer_tls_enabled: bool,
    
    /// Path to peer certificate
    pub peer_cert_file: String,
    
    /// Path to peer private key
    pub peer_key_file: String,
    
    /// Path to peer CA certificate
    pub peer_ca_file: String,
    
    /// Enable client certificate authentication for peers
    pub peer_client_cert_auth: bool,
    
    /// Enable TLS for client communication
    pub client_tls_enabled: bool,
    
    /// Path to client certificate
    pub client_cert_file: String,
    
    /// Path to client private key
    pub client_key_file: String,
    
    /// Path to client CA certificate
    pub client_ca_file: String,
    
    /// Enable client certificate authentication
    pub client_cert_auth: bool,
    
    /// FIPS-compliant cipher suites
    pub cipher_suites: Vec<String>,
    
    /// Minimum TLS version (1.2 for FIPS)
    pub min_tls_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// WAL directory (if different from data_dir)
    pub wal_dir: Option<String>,
    
    /// Snapshot count (default: 100000)
    pub snapshot_count: u64,
    
    /// Heartbeat interval (ms)
    pub heartbeat_interval: u64,
    
    /// Election timeout (ms)
    pub election_timeout: u64,
    
    /// Maximum snapshots to retain
    pub max_snapshots: u32,
    
    /// Maximum WAL files to retain
    pub max_wals: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Maximum concurrent requests
    pub max_request_bytes: u64,
    
    /// gRPC keepalive min time
    pub grpc_keepalive_min_time: String,
    
    /// gRPC keepalive interval
    pub grpc_keepalive_interval: String,
    
    /// gRPC keepalive timeout
    pub grpc_keepalive_timeout: String,
}

impl Default for EtcdConfig {
    fn default() -> Self {
        Self {
            version: "3.5.10".to_string(),
            node: NodeConfig {
                id: String::new(),
                name: String::new(),
                peer_address: "0.0.0.0".parse().unwrap(),
                peer_port: 2380,
                client_address: "0.0.0.0".parse().unwrap(),
                client_port: 2379,
                initial_cluster_state: "new".to_string(),
                data_dir: "/var/lib/etcd".to_string(),
            },
            cluster: ClusterConfig {
                cluster_token: String::new(),
                initial_cluster: HashMap::new(),
                auto_compaction_mode: "periodic".to_string(),
                auto_compaction_retention: "24h".to_string(),
                quota_backend_bytes: 8 * 1024 * 1024 * 1024, // 8GB
            },
            security: SecurityConfig {
                peer_tls_enabled: true,
                peer_cert_file: "/etc/kubernetes/pki/etcd/peer.crt".to_string(),
                peer_key_file: "/etc/kubernetes/pki/etcd/peer.key".to_string(),
                peer_ca_file: "/etc/kubernetes/pki/etcd/ca.crt".to_string(),
                peer_client_cert_auth: true,
                client_tls_enabled: true,
                client_cert_file: "/etc/kubernetes/pki/etcd/server.crt".to_string(),
                client_key_file: "/etc/kubernetes/pki/etcd/server.key".to_string(),
                client_ca_file: "/etc/kubernetes/pki/etcd/ca.crt".to_string(),
                client_cert_auth: true,
                cipher_suites: vec![
                    "TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256".to_string(),
                    "TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384".to_string(),
                    "TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256".to_string(),
                    "TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384".to_string(),
                    "TLS_RSA_WITH_AES_128_GCM_SHA256".to_string(),
                    "TLS_RSA_WITH_AES_256_GCM_SHA384".to_string(),
                ],
                min_tls_version: "1.2".to_string(),
            },
            storage: StorageConfig {
                wal_dir: None,
                snapshot_count: 100000,
                heartbeat_interval: 100,
                election_timeout: 1000,
                max_snapshots: 5,
                max_wals: 5,
            },
            performance: PerformanceConfig {
                max_request_bytes: 1024 * 1024, // 1MB
                grpc_keepalive_min_time: "5s".to_string(),
                grpc_keepalive_interval: "2h".to_string(),
                grpc_keepalive_timeout: "20s".to_string(),
            },
        }
    }
}

impl EtcdConfig {
    /// Generate etcd command line arguments
    pub fn to_args(&self) -> Vec<String> {
        let mut args = vec![
            "/usr/local/bin/etcd".to_string(),
            format!("--name={}", self.node.name),
            format!("--data-dir={}", self.node.data_dir),
            format!("--initial-cluster-state={}", self.node.initial_cluster_state),
            format!("--initial-cluster-token={}", self.cluster.cluster_token),
        ];
        
        // Peer URLs
        args.push(format!(
            "--listen-peer-urls=https://{}:{}",
            self.node.peer_address, self.node.peer_port
        ));
        args.push(format!(
            "--initial-advertise-peer-urls=https://{}:{}",
            self.node.peer_address, self.node.peer_port
        ));
        
        // Client URLs
        args.push(format!(
            "--listen-client-urls=https://{}:{},https://127.0.0.1:{}",
            self.node.client_address, self.node.client_port, self.node.client_port
        ));
        args.push(format!(
            "--advertise-client-urls=https://{}:{}",
            self.node.client_address, self.node.client_port
        ));
        
        // Initial cluster
        let initial_cluster: Vec<String> = self.cluster.initial_cluster
            .iter()
            .map(|(name, url)| format!("{}={}", name, url))
            .collect();
        if !initial_cluster.is_empty() {
            args.push(format!("--initial-cluster={}", initial_cluster.join(",")));
        }
        
        // Security settings
        if self.security.peer_tls_enabled {
            args.push(format!("--peer-cert-file={}", self.security.peer_cert_file));
            args.push(format!("--peer-key-file={}", self.security.peer_key_file));
            args.push(format!("--peer-trusted-ca-file={}", self.security.peer_ca_file));
            if self.security.peer_client_cert_auth {
                args.push("--peer-client-cert-auth=true".to_string());
            }
        }
        
        if self.security.client_tls_enabled {
            args.push(format!("--cert-file={}", self.security.client_cert_file));
            args.push(format!("--key-file={}", self.security.client_key_file));
            args.push(format!("--trusted-ca-file={}", self.security.client_ca_file));
            if self.security.client_cert_auth {
                args.push("--client-cert-auth=true".to_string());
            }
        }
        
        // Storage settings
        args.push(format!("--snapshot-count={}", self.storage.snapshot_count));
        args.push(format!("--heartbeat-interval={}", self.storage.heartbeat_interval));
        args.push(format!("--election-timeout={}", self.storage.election_timeout));
        args.push(format!("--max-snapshots={}", self.storage.max_snapshots));
        args.push(format!("--max-wals={}", self.storage.max_wals));
        
        if let Some(wal_dir) = &self.storage.wal_dir {
            args.push(format!("--wal-dir={}", wal_dir));
        }
        
        // Cluster settings
        args.push(format!("--auto-compaction-mode={}", self.cluster.auto_compaction_mode));
        args.push(format!("--auto-compaction-retention={}", self.cluster.auto_compaction_retention));
        args.push(format!("--quota-backend-bytes={}", self.cluster.quota_backend_bytes));
        
        // Performance settings
        args.push(format!("--max-request-bytes={}", self.performance.max_request_bytes));
        args.push(format!("--grpc-keepalive-min-time={}", self.performance.grpc_keepalive_min_time));
        args.push(format!("--grpc-keepalive-interval={}", self.performance.grpc_keepalive_interval));
        args.push(format!("--grpc-keepalive-timeout={}", self.performance.grpc_keepalive_timeout));
        
        // FIPS cipher suites
        if !self.security.cipher_suites.is_empty() {
            args.push(format!("--cipher-suites={}", self.security.cipher_suites.join(",")));
        }
        
        // Enable v3 API
        args.push("--enable-v2=false".to_string());
        
        // Logging
        args.push("--logger=zap".to_string());
        args.push("--log-level=info".to_string());
        
        args
    }
}