pub mod service;
pub mod config;
pub mod client;
pub mod static_pod;

#[cfg(test)]
mod tests;

pub use service::EtcdService;
pub use config::EtcdConfig;
pub use static_pod::generate_static_pod_manifest;

// Re-export proto types
pub use crate::proto::etcd::*;