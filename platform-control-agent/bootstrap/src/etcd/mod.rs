pub mod service;
pub mod config;
pub mod client;

pub use service::EtcdService;
pub use config::EtcdConfig;

// Re-export proto types
pub use crate::proto::etcd::*;