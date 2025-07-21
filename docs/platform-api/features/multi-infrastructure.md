# Feature: Multi-Infrastructure Support

## Overview
Abstraction layer that enables the platform to run seamlessly across vSphere, bare metal, and Apache CloudStack infrastructure with consistent operational semantics.

## Status
- **Feature Status**: 🟡 In Design
- **Target Release**: v1.2.0
- **Feature Owner**: Platform Team
- **Last Updated**: 2025-01-20

## Summary
Multi-Infrastructure Support provides a unified interface for deploying and managing Bottlerocket-based Kubernetes clusters across different infrastructure providers. It abstracts provider-specific details while exposing necessary customization points.

## Exit Criteria

### 1. Infrastructure Abstraction Layer
**Issue**: [#040](https://github.com/org/repo/issues/040)
- [ ] Provider interface definition
- [ ] Common resource model
- [ ] Network abstraction
- [ ] Storage abstraction
- [ ] Metadata service abstraction

### 2. vSphere Provider
**Issue**: [#041](https://github.com/org/repo/issues/041)
- [ ] VM provisioning via govc/govmomi
- [ ] OVA template management
- [ ] Network configuration
- [ ] Storage provisioning
- [ ] vCenter integration

### 3. Bare Metal Provider
**Issue**: [#042](https://github.com/org/repo/issues/042)
- [ ] PXE boot integration
- [ ] IPMI/BMC management
- [ ] Network boot configuration
- [ ] Hardware discovery
- [ ] Firmware management

### 4. CloudStack Provider
**Issue**: [#043](https://github.com/org/repo/issues/043)
- [ ] CloudStack API integration
- [ ] Template management
- [ ] Network zone support
- [ ] Security group configuration
- [ ] Load balancer integration

### 5. Provider Agnostic Features
**Issue**: [#044](https://github.com/org/repo/issues/044)
- [ ] Unified machine provisioning
- [ ] Cross-provider networking
- [ ] Consistent storage classes
- [ ] Provider migration tools
- [ ] Multi-provider clusters

## Technical Design

### Provider Interface
```go
package infrastructure

type Provider interface {
    // Machine lifecycle
    CreateMachine(spec MachineSpec) (*Machine, error)
    DeleteMachine(machineID string) error
    GetMachine(machineID string) (*Machine, error)
    ListMachines(selector labels.Selector) ([]*Machine, error)
    
    // Power management
    PowerOn(machineID string) error
    PowerOff(machineID string) error
    Reboot(machineID string) error
    
    // Configuration
    SetUserData(machineID string, userData []byte) error
    GetMetadata(machineID string) (*Metadata, error)
    
    // Networking
    AttachNetwork(machineID string, network NetworkSpec) error
    DetachNetwork(machineID string, networkID string) error
    
    // Storage
    AttachVolume(machineID string, volume VolumeSpec) error
    DetachVolume(machineID string, volumeID string) error
}

type MachineSpec struct {
    Name         string
    Provider     string
    Size         string            // Provider-specific size
    Image        string            // Bottlerocket image reference
    Networks     []NetworkSpec
    Volumes      []VolumeSpec
    UserData     []byte
    ProviderSpec runtime.RawExtension // Provider-specific config
}
```

### vSphere Implementation
```go
type VSphereProvider struct {
    client   *govmomi.Client
    config   VSphereConfig
}

type VSphereConfig struct {
    VCenter      string
    Datacenter   string
    Cluster      string
    Network      string
    Datastore    string
    ResourcePool string
    Template     string // Bottlerocket OVA
}

func (v *VSphereProvider) CreateMachine(spec MachineSpec) (*Machine, error) {
    // Decode provider-specific configuration
    var vmSpec VSphereVMSpec
    if err := json.Unmarshal(spec.ProviderSpec.Raw, &vmSpec); err != nil {
        return nil, err
    }
    
    // Clone from template
    vm, err := v.cloneFromTemplate(spec.Name, vmSpec)
    if err != nil {
        return nil, err
    }
    
    // Set user data via guestinfo
    if err := v.setGuestInfo(vm, "guestinfo.userdata", 
        base64.StdEncoding.EncodeToString(spec.UserData)); err != nil {
        return nil, err
    }
    
    // Power on
    if err := v.powerOn(vm); err != nil {
        return nil, err
    }
    
    return v.machineFromVM(vm), nil
}
```

### Bare Metal Implementation
```go
type BareMetalProvider struct {
    tinkerbellClient *tinkerbell.Client
    ipmiClient       *ipmi.Client
    config           BareMetalConfig
}

func (b *BareMetalProvider) CreateMachine(spec MachineSpec) (*Machine, error) {
    // Register hardware in Tinkerbell
    hardware := b.createHardwareSpec(spec)
    if err := b.tinkerbellClient.CreateHardware(hardware); err != nil {
        return nil, err
    }
    
    // Create workflow for Bottlerocket provisioning
    workflow := b.createBottlerocketWorkflow(spec)
    if err := b.tinkerbellClient.CreateWorkflow(workflow); err != nil {
        return nil, err
    }
    
    // Configure PXE boot
    if err := b.configurePXEBoot(spec); err != nil {
        return nil, err
    }
    
    // Power cycle to initiate provisioning
    if err := b.ipmiClient.PowerCycle(spec.Name); err != nil {
        return nil, err
    }
    
    return b.waitForProvisioning(spec.Name)
}
```

### CloudStack Implementation
```go
type CloudStackProvider struct {
    client *cloudstack.Client
    config CloudStackConfig
}

func (c *CloudStackProvider) CreateMachine(spec MachineSpec) (*Machine, error) {
    // Create VM from Bottlerocket template
    params := &cloudstack.DeployVirtualMachineParams{
        ServiceOfferingID: c.getServiceOffering(spec.Size),
        TemplateID:        c.getBottlerocketTemplate(spec.Image),
        ZoneID:            c.config.ZoneID,
        Name:              spec.Name,
        UserData:          base64.StdEncoding.EncodeToString(spec.UserData),
    }
    
    vm, err := c.client.VirtualMachine.DeployVirtualMachine(params)
    if err != nil {
        return nil, err
    }
    
    // Configure networking
    for _, network := range spec.Networks {
        if err := c.attachNetwork(vm.ID, network); err != nil {
            return nil, err
        }
    }
    
    return c.machineFromVM(vm), nil
}
```

### Provider Configuration
```yaml
apiVersion: infrastructure.platform.io/v1alpha1
kind: InfrastructureProvider
metadata:
  name: vsphere-prod
spec:
  type: vsphere
  vsphere:
    vcenter: vcenter.company.com
    datacenter: DC1
    cluster: Cluster1
    defaultNetwork: VM Network
    defaultDatastore: datastore1
    templates:
      bottlerocket-1.16.1: /DC1/vm/templates/bottlerocket-vmware-k8s-1.28-v1.16.1
---
apiVersion: infrastructure.platform.io/v1alpha1
kind: InfrastructureProvider
metadata:
  name: baremetal-prod
spec:
  type: baremetal
  baremetal:
    tinkerbellEndpoint: tinkerbell.company.com:42113
    osieEndpoint: boots.company.com:8080
    ipmiEndpoint: ipmi.company.com
    networkConfig:
      pxeNetwork: 10.0.0.0/24
      dhcpRange: 10.0.0.100-10.0.0.200
---
apiVersion: infrastructure.platform.io/v1alpha1
kind: InfrastructureProvider
metadata:
  name: cloudstack-prod
spec:
  type: cloudstack
  cloudstack:
    endpoint: cloudstack.company.com
    zone: zone1
    defaultOffering: Medium
    defaultNetwork: Guest Network
    templates:
      bottlerocket-1.16.1: bottlerocket-1.16.1-template
```

## Dependencies
- govmomi (vSphere SDK)
- Tinkerbell (bare metal provisioning)
- CloudStack Go SDK
- Network boot infrastructure (DHCP, TFTP, HTTP)

## Risks & Mitigations
| Risk | Impact | Mitigation |
|------|--------|------------|
| Provider API changes | Medium | Version pinning, compatibility layer |
| Network complexity | High | Simplified network model, good defaults |
| Hardware compatibility | High | Hardware compatibility list, testing |
| Provider lock-in | Medium | Clean abstraction, migration tools |

## Implementation Phases

### Phase 1: vSphere (Weeks 1-4)
- Provider interface design
- vSphere implementation
- Integration testing

### Phase 2: Bare Metal (Weeks 5-8)
- Tinkerbell integration
- PXE boot workflow
- Hardware testing

### Phase 3: CloudStack (Weeks 9-12)
- CloudStack provider
- Multi-provider testing
- Documentation

## Success Metrics
- Provisioning time < 10 minutes (all providers)
- Provider switching without application changes
- 99.9% provisioning success rate
- Support for 100+ node clusters

## Testing Requirements
- Provider-specific test environments
- Cross-provider migration tests
- Network failure scenarios
- Scale testing per provider

## Related Features
- [Platform Control Agent](./platform-control-agent.md)
- [Cluster Bootstrap](./cluster-bootstrap.md)
- [Machine Configuration](./machine-configuration.md)
- [CAPI Integration](./capi-integration.md)