use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};
use std::collections::BTreeMap;

use super::config::EtcdConfig;

/// Kubernetes static pod manifest for etcd
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StaticPodManifest {
    api_version: String,
    kind: String,
    metadata: PodMetadata,
    spec: PodSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PodMetadata {
    name: String,
    namespace: String,
    labels: BTreeMap<String, String>,
    annotations: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PodSpec {
    containers: Vec<Container>,
    host_network: bool,
    priority_class_name: String,
    volumes: Vec<Volume>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Container {
    name: String,
    image: String,
    command: Vec<String>,
    env: Vec<EnvVar>,
    liveness_probe: Option<Probe>,
    readiness_probe: Option<Probe>,
    startup_probe: Option<Probe>,
    resources: Resources,
    volume_mounts: Vec<VolumeMount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EnvVar {
    name: String,
    value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Probe {
    http_get: Option<HttpGet>,
    initial_delay_seconds: u32,
    timeout_seconds: u32,
    period_seconds: u32,
    failure_threshold: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HttpGet {
    path: String,
    port: u16,
    scheme: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Resources {
    requests: ResourceList,
    limits: ResourceList,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ResourceList {
    cpu: String,
    memory: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Volume {
    name: String,
    host_path: Option<HostPath>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HostPath {
    path: String,
    #[serde(rename = "type")]
    type_: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VolumeMount {
    name: String,
    mount_path: String,
}

/// Generate etcd static pod manifest
pub fn generate_static_pod_manifest(config: &EtcdConfig) -> Result<String> {
    let manifest = create_manifest(config)?;
    
    // Serialize to YAML
    serde_yaml::to_string(&manifest)
        .context("Failed to serialize static pod manifest")
}

fn create_manifest(config: &EtcdConfig) -> Result<StaticPodManifest> {
    let mut labels = BTreeMap::new();
    labels.insert("component".to_string(), "etcd".to_string());
    labels.insert("tier".to_string(), "control-plane".to_string());
    
    let mut annotations = BTreeMap::new();
    annotations.insert("kubeadm.kubernetes.io/etcd.advertise-client-urls".to_string(), 
                     format!("https://{}:{}", config.node.client_address, config.node.client_port));
    
    let metadata = PodMetadata {
        name: "etcd".to_string(),
        namespace: "kube-system".to_string(),
        labels,
        annotations,
    };
    
    let container = create_container(config)?;
    let volumes = create_volumes();
    
    let spec = PodSpec {
        containers: vec![container],
        host_network: true,
        priority_class_name: "system-node-critical".to_string(),
        volumes,
    };
    
    Ok(StaticPodManifest {
        api_version: "v1".to_string(),
        kind: "Pod".to_string(),
        metadata,
        spec,
    })
}

fn create_container(config: &EtcdConfig) -> Result<Container> {
    let command = config.to_args();
    
    let env = vec![
        EnvVar {
            name: "ETCD_NAME".to_string(),
            value: config.node.name.clone(),
        },
        EnvVar {
            name: "ETCD_DATA_DIR".to_string(),
            value: config.node.data_dir.clone(),
        },
        // FIPS mode environment variable
        EnvVar {
            name: "ETCD_CIPHER_SUITES".to_string(),
            value: config.security.cipher_suites.join(","),
        },
    ];
    
    let liveness_probe = Some(Probe {
        http_get: Some(HttpGet {
            path: "/health?exclude=NOSPACE&serializable=true".to_string(),
            port: config.node.client_port,
            scheme: "HTTPS".to_string(),
        }),
        initial_delay_seconds: 10,
        timeout_seconds: 15,
        period_seconds: 10,
        failure_threshold: 8,
    });
    
    let startup_probe = Some(Probe {
        http_get: Some(HttpGet {
            path: "/health?serializable=false".to_string(),
            port: config.node.client_port,
            scheme: "HTTPS".to_string(),
        }),
        initial_delay_seconds: 10,
        timeout_seconds: 15,
        period_seconds: 10,
        failure_threshold: 24,
    });
    
    let resources = Resources {
        requests: ResourceList {
            cpu: "100m".to_string(),
            memory: "100Mi".to_string(),
        },
        limits: ResourceList {
            cpu: "2".to_string(),
            memory: "8Gi".to_string(),
        },
    };
    
    let volume_mounts = vec![
        VolumeMount {
            name: "etcd-data".to_string(),
            mount_path: config.node.data_dir.clone(),
        },
        VolumeMount {
            name: "etcd-certs".to_string(),
            mount_path: "/etc/kubernetes/pki/etcd".to_string(),
        },
    ];
    
    Ok(Container {
        name: "etcd".to_string(),
        image: format!("public.ecr.aws/bottlerocket/etcd:{}-eks-1-32-10", config.version),
        command,
        env,
        liveness_probe,
        readiness_probe: None,
        startup_probe,
        resources,
        volume_mounts,
    })
}

fn create_volumes() -> Vec<Volume> {
    vec![
        Volume {
            name: "etcd-certs".to_string(),
            host_path: Some(HostPath {
                path: "/etc/kubernetes/pki/etcd".to_string(),
                type_: "DirectoryOrCreate".to_string(),
            }),
        },
        Volume {
            name: "etcd-data".to_string(),
            host_path: Some(HostPath {
                path: "/var/lib/etcd".to_string(),
                type_: "DirectoryOrCreate".to_string(),
            }),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::IpAddr;
    
    #[test]
    fn test_generate_static_pod_manifest() {
        let mut config = EtcdConfig::default();
        config.node.name = "etcd-0".to_string();
        config.node.peer_address = "10.0.0.1".parse::<IpAddr>().unwrap();
        config.node.client_address = "10.0.0.1".parse::<IpAddr>().unwrap();
        config.cluster.cluster_token = "test-cluster".to_string();
        
        let manifest = generate_static_pod_manifest(&config).unwrap();
        
        // Verify it's valid YAML
        let parsed: serde_yaml::Value = serde_yaml::from_str(&manifest).unwrap();
        assert_eq!(parsed["kind"], "Pod");
        assert_eq!(parsed["metadata"]["name"], "etcd");
        assert_eq!(parsed["spec"]["containers"][0]["name"], "etcd");
    }
}