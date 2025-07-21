# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build and Development Commands

### Common Build Commands
```bash
# Build the default variant (aws-k8s-1.32)
cargo make

# Build a specific variant
cargo make -e BUILDSYS_VARIANT=aws-k8s-1.31
cargo make -e BUILDSYS_VARIANT=aws-ecs-1
cargo make -e BUILDSYS_VARIANT=vmware-k8s-1.32

# Build for a specific architecture
cargo make -e BUILDSYS_ARCH=aarch64
cargo make -e BUILDSYS_ARCH=x86_64

# Build with limited concurrency
cargo make -e BUILDSYS_JOBS=4
```

### Testing Commands
```bash
# Run unit tests
cargo make unit-tests

# Run unit tests for specific variant/arch
cargo make -e BUILDSYS_VARIANT="aws-ecs-1" -e BUILDSYS_ARCH="x86_64" unit-tests

# Run integration tests (requires testsys setup)
cargo make setup-test
cargo make test
cargo make watch-test
```

### Linting and Code Quality
```bash
# Run all checks
cargo make check

# Individual checks
cargo make check-fmt
cargo make check-clippy
cargo make check-lints
cargo make check-shell
cargo make check-golangci-lint
cargo make check-migrations
cargo make check-licenses
```

### Clean Commands
```bash
cargo make clean          # Clean everything
cargo make clean-sources  # Clean source builds
cargo make clean-packages # Clean package builds
cargo make clean-images   # Clean built images
```

## High-Level Architecture

### Core Design Principles
- **Immutable root filesystem** with dm-verity for integrity checking
- **API-driven configuration** - no SSH by default, all management through APIs
- **Dual partition system** (A/B) for atomic updates and rollback
- **Variant-based builds** for different use cases (EKS, ECS, VMware)
- **Security-first design** with SELinux enforcing, minimal attack surface

### Key Directories
- `sources/` - First-party Rust code including API system, models, and settings
- `packages/` - RPM package definitions for all components
- `variants/` - Variant definitions combining packages and configurations
- `tools/` - Build system (Twoliter), publishing tools (pubsys), and utilities

### Variant System
Each variant is a specific build of Bottlerocket optimized for a use case:
- AWS Kubernetes variants: `aws-k8s-1.27` through `aws-k8s-1.33`
- AWS ECS variants: `aws-ecs-1`, `aws-ecs-2`
- VMware variants: `vmware-k8s-1.28` through `vmware-k8s-1.33`
- NVIDIA variants for GPU support
- FIPS-validated variants available

### API System Architecture
The API system manages all configuration:
1. **Datastore** - Transactional key-value store for settings
2. **API Server** - Unix socket-based HTTP API
3. **Settings Models** - Strongly-typed configuration definitions
4. **Migration System** - Automatic migration between versions
5. **Commit/Apply Pattern** - Two-phase configuration updates

### Settings Organization
Settings are layered from multiple sources:
1. Base defaults
2. Variant-specific defaults in `defaults.d/`
3. User data (cloud-init format)
4. Runtime API changes

### Update System
- Uses TUF (The Update Framework) for secure updates
- Full filesystem images, not package updates
- Automatic rollback on boot failure
- Update operators available for Kubernetes and ECS

### Build System (Twoliter)
- Container-based builds for reproducibility
- Cross-compilation support for x86_64 and aarch64
- SDK and kit system for modular builds
- All tasks defined in `Makefile.toml` using cargo-make

### Security Features
- SELinux in enforcing mode
- dm-verity for filesystem integrity
- Secure boot support
- No default SSH access
- Minimal installed packages
- All first-party code in memory-safe Rust

### Key Environment Variables
- `BUILDSYS_VARIANT` - Which variant to build
- `BUILDSYS_ARCH` - Target architecture
- `BUILDSYS_JOBS` - Parallel build jobs
- `PUBLISH_REGIONS` - AWS regions for AMI publishing