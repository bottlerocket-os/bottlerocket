# Bottlerocket Platform API Documentation

This directory contains the design and implementation documentation for building an API-driven Kubernetes platform on top of Bottlerocket's FIPS-compliant variants.

## Overview

The Bottlerocket Platform API project extends Bottlerocket with a Talos-like control plane that provides:
- Complete API-driven management (no SSH access)
- FedRAMP/STIG compliance through FIPS variants
- Multi-infrastructure support (vSphere, bare metal, CloudStack)
- Declarative configuration and GitOps integration
- Autonomous cluster bootstrapping

## Documentation Structure

### 1. [Executive Summary](bottlerocket-platform-executive-summary.md)
High-level overview of the project including:
- Why Bottlerocket was chosen over alternatives
- Architecture approach using Cluster API
- Implementation roadmap
- Risk analysis

### 2. [Platform Design Document](bottlerocket-api-driven-platform-design.md)
Detailed technical design covering:
- Platform Control Agent architecture
- gRPC API specifications
- Machine configuration schemas
- Cluster bootstrap process
- FIPS compliance implementation
- Code examples and implementation details

### 3. [Architecture Deep Dive](api-driven-architecture-deepdive.md)
Advanced concepts and future considerations:
- Comparison with Talos Linux approach
- GitOps-native design patterns
- Policy-driven operations
- Extensibility model

## Quick Start

To implement the Platform API on Bottlerocket:

1. **Build the Platform Control Agent** (see [platform design](bottlerocket-api-driven-platform-design.md#core-components))
2. **Create a custom Bottlerocket variant** with FIPS support
3. **Deploy using the bootstrap process** outlined in the design docs
4. **Manage clusters** via the gRPC API or Cluster API integration

## Project Status

This project is in the design phase. Implementation will proceed in phases:
- Phase 1: Core Platform Agent (Weeks 1-4)
- Phase 2: Bootstrap Mechanism (Weeks 5-8)
- Phase 3: Cluster API Integration (Weeks 9-12)
- Phase 4: Production Features (Weeks 13-16)
- Phase 5: Multi-Infrastructure (Weeks 17-20)

## Contributing

This is a fork of the official Bottlerocket project focused on adding API-driven platform capabilities. Contributions should align with:
- Maintaining FIPS compliance throughout
- Preserving Bottlerocket's security model
- Following the API-first design principle
- Supporting multi-infrastructure deployment

## Related Projects

- [Bottlerocket](https://github.com/bottlerocket-os/bottlerocket) - The base operating system
- [Talos Linux](https://www.talos.dev/) - Inspiration for the API-driven model
- [Cluster API](https://cluster-api.sigs.k8s.io/) - Kubernetes cluster lifecycle management
- [EKS Anywhere](https://anywhere.eks.amazonaws.com/) - AWS's Kubernetes distribution