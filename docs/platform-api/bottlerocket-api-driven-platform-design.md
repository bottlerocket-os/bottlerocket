# Bottlerocket API-Driven Platform Design
## Building a Talos-Like Control Plane on FIPS-Compliant Bottlerocket

### Executive Summary

This document outlines the design and implementation of an API-driven Kubernetes platform using Bottlerocket's FIPS-compliant variants as the foundation. By adding a privileged host container layer, we achieve Talos-like operational semantics while maintaining FedRAMP compliance through native FIPS support.

### Key Innovation

Rather than building a new operating system or forking Bottlerocket, we leverage Bottlerocket's host container capability to add a control plane layer that provides:
- Complete API-driven management (no SSH)
- Declarative machine configuration
- Autonomous cluster bootstrapping
- Multi-infrastructure support (vSphere, bare metal, cloud)
- Full FIPS 140-3 compliance throughout the stack

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                   External Management Layer                  │
│                                                             │
│  • kubectl (Cluster API)                                    │
│  • platformctl (Machine API)                               │
│  • gitops-controller (Declarative Config)                  │
│                                                             │
└──────────────────────────┬──────────────────────────────────┘
                           │ mTLS (FIPS-compliant)
┌──────────────────────────▼──────────────────────────────────┐
│              Bottlerocket Node (FIPS Variant)               │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │     Platform Control Agent (Host Container)          │   │
│  │                                                      │   │
│  │  • gRPC API Server (port 50000)                    │   │
│  │  • Machine Configuration Controller                 │   │
│  │  • Cluster Bootstrap Orchestrator                   │   │
│  │  • Certificate Management                           │   │
│  │  • Bottlerocket API Client                          │   │
│  └──────────────────┬──────────────────────────────────┘   │
│                     │ Unix Socket                           │
│  ┌──────────────────▼──────────────────────────────────┐   │
│  │         Bottlerocket Settings API (Native)          │   │
│  │                                                      │   │
│  │  • Immutable root filesystem (dm-verity)           │   │
│  │  • FIPS-validated kernel crypto                    │   │
│  │  • SELinux enforcing mode                          │   │
│  │  • Automated OS updates (A/B partitions)           │   │
│  └──────────────────┬──────────────────────────────────┘   │
│                     │                                       │
│  ┌──────────────────▼──────────────────────────────────┐   │
│  │           Kubernetes Components (FIPS)               │   │
│  │                                                      │   │
│  │  • kubelet with FIPS crypto providers              │   │
│  │  • containerd with FIPS-compliant runtimes         │   │
│  │  • etcd with FIPS TLS cipher suites                │   │
│  │  • Static pods for bootstrap phase                  │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. Platform Control Agent

The Platform Control Agent is a privileged host container that provides the API-driven interface to Bottlerocket:

```rust
// src/main.rs - Platform Control Agent
use tonic::{transport::Server, Request, Response, Status};
use bottlerocket_api_client::Client as BottlerocketClient;

pub struct PlatformControlService {
    br_client: BottlerocketClient,
    cluster_state: Arc<Mutex<ClusterState>>,
    cert_manager: CertificateManager,
}

#[tonic::async_trait]
impl MachineService for PlatformControlService {
    // Apply machine configuration
    async fn apply_configuration(
        &self,
        request: Request<MachineConfigRequest>,
    ) -> Result<Response<MachineConfigResponse>, Status> {
        let config = request.into_inner();
        
        // Validate configuration
        self.validate_fips_compliance(&config)?;
        
        // Translate to Bottlerocket settings
        let br_settings = self.translate_to_bottlerocket(config)?;
        
        // Apply via Bottlerocket API
        self.br_client
            .set_settings(br_settings)
            .await
            .map_err(|e| Status::internal(format!("Failed to apply settings: {}", e)))?;
        
        Ok(Response::new(MachineConfigResponse {
            success: true,
            message: "Configuration applied successfully".to_string(),
        }))
    }
    
    // Bootstrap cluster formation
    async fn bootstrap_cluster(
        &self,
        request: Request<BootstrapRequest>,
    ) -> Result<Response<BootstrapResponse>, Status> {
        let bootstrap_config = request.into_inner();
        
        // Generate cluster PKI
        let pki = self.cert_manager.generate_cluster_pki(
            &bootstrap_config.cluster_name,
            &bootstrap_config.sans,
        )?;
        
        // Configure etcd static pod
        self.configure_etcd_bootstrap(&pki).await?;
        
        // Configure control plane components
        self.configure_control_plane(&pki, &bootstrap_config).await?;
        
        Ok(Response::new(BootstrapResponse {
            cluster_id: pki.cluster_id,
            ca_certificate: pki.ca_cert,
            join_token: self.generate_join_token().await?,
        }))
    }
}
```

### 2. Host Container Configuration

The Platform Control Agent runs as a superpowered host container with access to the Bottlerocket API:

```toml
# user-data.toml for platform nodes
[settings.host-containers.platform-control]
enabled = true
source = "your-registry.io/platform-control:v1.0.0-fips"
superpowered = true

# Mount points for API access and certificates
[settings.host-containers.platform-control.mounts]
api-socket = { source = "/run/api.sock", destination = "/run/api.sock" }
certs-dir = { source = "/etc/platform/certs", destination = "/etc/platform/certs" }
static-pods = { source = "/etc/kubernetes/static-pods", destination = "/etc/kubernetes/static-pods" }

# Environment configuration
[settings.host-containers.platform-control.environment]
PLATFORM_ROLE = "control-plane"
PLATFORM_CLUSTER_ENDPOINT = "https://api.cluster.local:6443"
PLATFORM_FIPS_MODE = "enforcing"
RUST_LOG = "info"

# User data for initial bootstrap
user-data = "BASE64_ENCODED_BOOTSTRAP_CONFIG"
```

### 3. Machine Configuration API

The platform exposes a declarative API for machine configuration:

```protobuf
// api/machine.proto
syntax = "proto3";

package platform.machine.v1alpha1;

service MachineService {
  rpc ApplyConfiguration(MachineConfigRequest) returns (MachineConfigResponse);
  rpc GetConfiguration(GetConfigRequest) returns (MachineConfig);
  rpc Reset(ResetRequest) returns (ResetResponse);
  rpc Reboot(RebootRequest) returns (RebootResponse);
  rpc Upgrade(UpgradeRequest) returns (UpgradeResponse);
}

message MachineConfig {
  string version = 1;
  MachineType type = 2;
  
  message Cluster {
    string name = 1;
    string endpoint = 2;
    string ca_certificate = 3;
    string bootstrap_token = 4;
  }
  Cluster cluster = 3;
  
  message Network {
    repeated NetworkInterface interfaces = 1;
    repeated string nameservers = 2;
  }
  Network network = 4;
  
  message Security {
    bool fips_enabled = 1;
    string selinux_mode = 2;
    map<string, string> kernel_parameters = 3;
    repeated string audit_rules = 4;
  }
  Security security = 5;
  
  message Features {
    map<string, string> kubelet_args = 1;
    map<string, ContainerSpec> host_containers = 2;
    repeated StaticPod static_pods = 3;
  }
  Features features = 6;
}

enum MachineType {
  CONTROL_PLANE = 0;
  WORKER = 1;
}
```

### 4. Cluster Bootstrap Process

The platform implements autonomous cluster bootstrapping without external dependencies:

```go
// pkg/bootstrap/cluster.go
package bootstrap

type ClusterBootstrapper struct {
    platformClient PlatformClient
    certManager    CertificateManager
}

func (cb *ClusterBootstrapper) InitializeCluster(config BootstrapConfig) error {
    // Phase 1: Elect initial leader using deterministic algorithm
    leader := cb.electInitialLeader(config.Nodes)
    
    // Phase 2: Generate cluster PKI on leader
    if leader.IsLocal() {
        pki, err := cb.certManager.GenerateClusterPKI(config)
        if err != nil {
            return fmt.Errorf("failed to generate PKI: %w", err)
        }
        
        // Distribute certificates to other control plane nodes
        for _, node := range config.ControlPlaneNodes {
            if node.ID != leader.ID {
                err := cb.distributeCertificates(node, pki)
                if err != nil {
                    return fmt.Errorf("failed to distribute certs to %s: %w", node.ID, err)
                }
            }
        }
    }
    
    // Phase 3: Bootstrap etcd cluster
    etcdConfig := cb.generateEtcdConfig(config.ControlPlaneNodes, pki)
    for _, node := range config.ControlPlaneNodes {
        err := cb.platformClient.ApplyStaticPod(node, "etcd", etcdConfig)
        if err != nil {
            return fmt.Errorf("failed to configure etcd on %s: %w", node.ID, err)
        }
    }
    
    // Phase 4: Bootstrap Kubernetes control plane
    k8sConfig := cb.generateControlPlaneConfig(config, pki)
    for _, node := range config.ControlPlaneNodes {
        err := cb.platformClient.ApplyStaticPods(node, k8sConfig)
        if err != nil {
            return fmt.Errorf("failed to configure k8s on %s: %w", node.ID, err)
        }
    }
    
    // Phase 5: Wait for cluster health
    return cb.waitForClusterHealth(config.ControlPlaneNodes)
}
```

### 5. FIPS Compliance Throughout

All components maintain FIPS compliance:

```yaml
# Static pod for etcd with FIPS configuration
apiVersion: v1
kind: Pod
metadata:
  name: etcd
  namespace: kube-system
spec:
  hostNetwork: true
  containers:
  - name: etcd
    image: registry.fips.io/etcd:v3.5.9-fips
    command:
    - etcd
    - --cert-file=/etc/kubernetes/pki/etcd/server.crt
    - --key-file=/etc/kubernetes/pki/etcd/server.key
    - --cipher-suites=TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256,TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384
    - --tls-min-version=TLS1.2
    env:
    - name: ETCD_CIPHER_SUITES
      value: "TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256,TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384"
    - name: ETCD_FIPS
      value: "true"
    volumeMounts:
    - name: etcd-certs
      mountPath: /etc/kubernetes/pki/etcd
  volumes:
  - name: etcd-certs
    hostPath:
      path: /etc/kubernetes/pki/etcd
```

## Platform Features

### 1. Declarative Configuration

All node configuration is declarative and version-controlled:

```yaml
# cluster-config.yaml
apiVersion: platform.io/v1alpha1
kind: ClusterConfiguration
metadata:
  name: production-cluster
spec:
  version: v1.28.0
  networking:
    podSubnet: 10.244.0.0/16
    serviceSubnet: 10.96.0.0/12
  security:
    fips:
      enabled: true
      enforcing: true
    auditPolicy:
      level: Metadata
      maxAge: 30
      maxBackup: 10
      maxSize: 100
---
apiVersion: platform.io/v1alpha1
kind: MachineConfiguration
metadata:
  name: control-plane-template
spec:
  role: controlPlane
  bottlerocket:
    variant: aws-k8s-1.28-fips
    settings:
      kernel:
        lockdown: integrity
        sysctl:
          "kernel.unprivileged_bpf_disabled": "1"
          "kernel.yama.ptrace_scope": "2"
  hostContainers:
    falco-fips:
      source: registry.fips.io/falco:0.35.1-fips
      enabled: true
    compliance-scanner:
      source: registry.fips.io/openscap:latest-fips
      enabled: true
```

### 2. GitOps Integration

The platform integrates with standard GitOps tooling:

```yaml
# flux/clusters/production/platform-sync.yaml
apiVersion: source.toolkit.fluxcd.io/v1beta2
kind: GitRepository
metadata:
  name: platform-config
  namespace: platform-system
spec:
  interval: 1m
  ref:
    branch: main
  url: https://git.company.com/platform/configs
---
apiVersion: kustomize.toolkit.fluxcd.io/v1beta2
kind: Kustomization
metadata:
  name: platform-nodes
  namespace: platform-system
spec:
  interval: 10m
  path: ./clusters/production
  prune: false  # Never auto-delete nodes
  sourceRef:
    kind: GitRepository
    name: platform-config
  validation: strict
  healthChecks:
    - apiVersion: platform.io/v1alpha1
      kind: MachineConfiguration
      name: "*"
      namespace: platform-system
```

### 3. Multi-Infrastructure Support

The platform abstracts infrastructure differences:

```go
// pkg/infrastructure/provider.go
type InfrastructureProvider interface {
    // Provision a new machine with given configuration
    ProvisionMachine(spec MachineSpec) (*Machine, error)
    
    // Configure machine-specific settings (network, storage)
    ConfigureMachine(machine *Machine, config MachineConfig) error
    
    // Get machine metadata (useful for cloud providers)
    GetMetadata(machine *Machine) (*MachineMetadata, error)
}

// Implementations for each infrastructure type
type VSphereProvider struct {
    vcenter *govmomi.Client
    config  VSphereConfig
}

type BareMetalProvider struct {
    tinkerbellClient *tinkerbell.Client
    ipxeServer       *IPXEServer
}

type CloudStackProvider struct {
    client *cloudstack.Client
    config CloudStackConfig
}
```

### 4. Update Orchestration

Platform-managed updates with zero downtime:

```go
// pkg/update/orchestrator.go
type UpdateOrchestrator struct {
    platform PlatformClient
    cluster  ClusterClient
}

func (uo *UpdateOrchestrator) UpdateCluster(plan UpdatePlan) error {
    // Validate update path
    if err := uo.validateUpdate(plan); err != nil {
        return fmt.Errorf("update validation failed: %w", err)
    }
    
    // Update control plane nodes
    for _, node := range plan.ControlPlaneNodes {
        // Cordon and drain
        if err := uo.cluster.CordonAndDrain(node); err != nil {
            return fmt.Errorf("failed to drain %s: %w", node.Name, err)
        }
        
        // Apply new configuration
        newConfig := plan.GetConfigForNode(node)
        if err := uo.platform.ApplyConfiguration(node, newConfig); err != nil {
            return fmt.Errorf("failed to update %s: %w", node.Name, err)
        }
        
        // Reboot into new Bottlerocket version
        if err := uo.platform.Reboot(node); err != nil {
            return fmt.Errorf("failed to reboot %s: %w", node.Name, err)
        }
        
        // Wait for node to rejoin
        if err := uo.waitForNodeReady(node); err != nil {
            return fmt.Errorf("node %s failed to rejoin: %w", node.Name, err)
        }
        
        // Uncordon
        if err := uo.cluster.Uncordon(node); err != nil {
            return fmt.Errorf("failed to uncordon %s: %w", node.Name, err)
        }
    }
    
    // Update worker nodes (can be parallelized)
    return uo.updateWorkerNodes(plan)
}
```

## Implementation Roadmap

### Phase 1: Core Platform Agent (Weeks 1-4)
- [ ] Implement basic Platform Control Agent
- [ ] Create gRPC API definitions
- [ ] Build Bottlerocket settings translator
- [ ] Package as FIPS-compliant container

### Phase 2: Bootstrap Mechanism (Weeks 5-8)
- [ ] Implement cluster PKI generation
- [ ] Create etcd bootstrap process
- [ ] Build control plane initialization
- [ ] Test multi-node formation

### Phase 3: Cluster API Integration (Weeks 9-12)
- [ ] Create CAPI provider wrapper
- [ ] Implement machine controller
- [ ] Add cluster controller
- [ ] Test with standard CAPI flows

### Phase 4: Production Features (Weeks 13-16)
- [ ] Add update orchestration
- [ ] Implement backup/restore
- [ ] Create compliance automation
- [ ] Build monitoring integration

### Phase 5: Multi-Infrastructure (Weeks 17-20)
- [ ] Complete vSphere provider
- [ ] Add bare metal support
- [ ] Implement CloudStack provider
- [ ] Create unified testing

## Security Considerations

### FIPS Compliance Chain
1. **Bottlerocket OS**: Uses FIPS-validated kernel crypto
2. **Platform Agent**: Built with FIPS-compliant Go toolchain
3. **Container Images**: All use FIPS-validated base images
4. **TLS Communications**: Only FIPS-approved cipher suites
5. **Storage Encryption**: FIPS-compliant encryption at rest

### Zero Trust Architecture
- No SSH access to nodes
- All access through authenticated API
- mTLS for all component communication
- Audit logging for all operations
- Time-limited bootstrap tokens

## Operational Benefits

### Compared to Traditional Kubernetes
- **No SSH Keys**: Eliminates key management overhead
- **No Configuration Drift**: Immutable OS + declarative config
- **Automated Compliance**: Continuous validation and remediation
- **Simplified Updates**: Coordinated OS and Kubernetes updates

### Compared to Talos Linux
- **FIPS Support**: Native FIPS variants available
- **AWS Integration**: First-class AWS support if needed
- **Proven Base**: Bottlerocket's production maturity
- **Flexibility**: Can still access via break-glass admin container

## Conclusion

This design provides a Talos-like operational experience on top of Bottlerocket's FIPS-compliant foundation. By leveraging host containers and the Bottlerocket API, we achieve:

1. **Complete API-driven management** without SSH
2. **FedRAMP compliance** through FIPS validation
3. **Multi-infrastructure support** through pluggable providers
4. **Operational simplicity** through declarative configuration
5. **Security by default** through immutable infrastructure

The Platform Control Agent acts as a bridge between the cloud-native ecosystem's expectations and Bottlerocket's security-focused design, giving us the best of both worlds.