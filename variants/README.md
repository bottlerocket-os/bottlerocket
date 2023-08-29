# Bottlerocket Variants

This document describes what Bottlerocket variants are and how they are built.

In the [Background](#background) section, we discuss the motivation for variants.

In the [Variants](#variants) section, we list the variants that exist today.

In the [Development](#development) section, we provide a short guide for adding a new variant.

## Background

Bottlerocket is purpose-built for hosting containers.
It can run one of several container orchestrator agents.
It is also image-based and does not include a package manager for customization at runtime.

Conceptually, each image could include all orchestrator agents, but that would conflict with our design goals.
We want to keep the footprint of Bottlerocket as small as possible for security and performance reasons.
Instead, we make different variants available for use, each with its own set of software and API settings.

A variant is essentially a list of packages to install, plus a model that defines the API.
The documentation for [packages](../packages/) covers how to create a package.
Information about API settings for variants can be found in the [models](../sources/models/) documentation.

### User data
Bottlerocket variants ingest TOML-formatted [user data](../README.md#using-user-data) from various sources in a predefined order.
All variants first attempt to read user data from `/var/lib/bottlerocket/user-data.toml`.
AWS variants then retrieve user data from IMDS.
VMware variants will attempt to read user data from a mounted CD-ROM (from a file named "user-data" or from an OVF file), and then from VMware's guestinfo interface.

If a setting is defined in more than one source, the value in later sources will override earlier values.
For example, in a VMware variant, settings read from the guestinfo interface will override settings from CD-ROM, and settings from CD-ROM will override settings from the file.

## Variants

### aws-k8s-1.23: Kubernetes 1.23 node

The [aws-k8s-1.23](aws-k8s-1.23/Cargo.toml) variant includes the packages needed to run a Kubernetes node in AWS.
It supports self-hosted clusters and clusters managed by [EKS](https://aws.amazon.com/eks/).

This variant is compatible with Kubernetes 1.23, 1.24, and 1.25 clusters.

### aws-k8s-1.23-nvidia: Kubernetes 1.23 NVIDIA node

The [aws-k8s-1.23-nvidia](aws-k8s-1.23-nvidia/Cargo.toml) variant includes the packages needed to run a Kubernetes node in AWS.
It also includes the required packages to configure containers to leverage NVIDIA GPUs.
It supports self-hosted clusters and clusters managed by [EKS](https://aws.amazon.com/eks/).

This variant is compatible with Kubernetes 1.23, 1.24, and 1.25 clusters.

### aws-k8s-1.24: Kubernetes 1.24 node

The [aws-k8s-1.24](aws-k8s-1.24/Cargo.toml) variant includes the packages needed to run a Kubernetes node in AWS.
It supports self-hosted clusters and clusters managed by [EKS](https://aws.amazon.com/eks/).

This variant is compatible with Kubernetes 1.24, 1.25, and 1.26 clusters.

### aws-k8s-1.24-nvidia: Kubernetes 1.24 NVIDIA node

The [aws-k8s-1.24-nvidia](aws-k8s-1.24-nvidia/Cargo.toml) variant includes the packages needed to run a Kubernetes node in AWS.
It also includes the required packages to configure containers to leverage NVIDIA GPUs.
It supports self-hosted clusters and clusters managed by [EKS](https://aws.amazon.com/eks/).

This variant is compatible with Kubernetes 1.24, 1.25, and 1.26 clusters.

### aws-k8s-1.25: Kubernetes 1.25 node

The [aws-k8s-1.25](aws-k8s-1.25/Cargo.toml) variant includes the packages needed to run a Kubernetes node in AWS.
It supports self-hosted clusters and clusters managed by [EKS](https://aws.amazon.com/eks/).

This variant is compatible with Kubernetes 1.25, 1.26, 1.27, and 1.28 clusters.

### aws-k8s-1.25-nvidia: Kubernetes 1.25 NVIDIA node

The [aws-k8s-1.25-nvidia](aws-k8s-1.25-nvidia/Cargo.toml) variant includes the packages needed to run a Kubernetes node in AWS.
It also includes the required packages to configure containers to leverage NVIDIA GPUs.
It supports self-hosted clusters and clusters managed by [EKS](https://aws.amazon.com/eks/).

This variant is compatible with Kubernetes 1.25, 1.26, 1.27, and 1.28 clusters.

### aws-k8s-1.26: Kubernetes 1.26 node

The [aws-k8s-1.26](aws-k8s-1.26/Cargo.toml) variant includes the packages needed to run a Kubernetes node in AWS.
It supports self-hosted clusters and clusters managed by [EKS](https://aws.amazon.com/eks/).

This variant is compatible with Kubernetes 1.26, 1.27, 1.28, and 1.29 clusters.

### aws-k8s-1.26-nvidia: Kubernetes 1.26 NVIDIA node

The [aws-k8s-1.26-nvidia](aws-k8s-1.26-nvidia/Cargo.toml) variant includes the packages needed to run a Kubernetes node in AWS.
It also includes the required packages to configure containers to leverage NVIDIA GPUs.
It supports self-hosted clusters and clusters managed by [EKS](https://aws.amazon.com/eks/).

This variant is compatible with Kubernetes 1.26, 1.27, 1.28, and 1.29 clusters.

### aws-k8s-1.27: Kubernetes 1.27 node

The [aws-k8s-1.27](aws-k8s-1.27/Cargo.toml) variant includes the packages needed to run a Kubernetes node in AWS.
It supports self-hosted clusters and clusters managed by [EKS](https://aws.amazon.com/eks/).

This variant is compatible with Kubernetes 1.27, 1.28, 1.29, and 1.30 clusters.

### aws-k8s-1.27-nvidia: Kubernetes 1.27 NVIDIA node

The [aws-k8s-1.27-nvidia](aws-k8s-1.27-nvidia/Cargo.toml) variant includes the packages needed to run a Kubernetes node in AWS.
It also includes the required packages to configure containers to leverage NVIDIA GPUs.
It supports self-hosted clusters and clusters managed by [EKS](https://aws.amazon.com/eks/).

This variant is compatible with Kubernetes 1.27, 1.28, 1.29, and 1.30 clusters.

### aws-k8s-1.28: Kubernetes 1.28 node

The [aws-k8s-1.28](aws-k8s-1.28/Cargo.toml) variant includes the packages needed to run a Kubernetes node in AWS.
It supports self-hosted clusters and clusters managed by [EKS](https://aws.amazon.com/eks/).

This variant is compatible with Kubernetes 1.28, 1.29, 1.30, and 1.31 clusters.

### aws-k8s-1.28-nvidia: Kubernetes 1.28 NVIDIA node

The [aws-k8s-1.28-nvidia](aws-k8s-1.28-nvidia/Cargo.toml) variant includes the packages needed to run a Kubernetes node in AWS.
It also includes the required packages to configure containers to leverage NVIDIA GPUs.
It supports self-hosted clusters and clusters managed by [EKS](https://aws.amazon.com/eks/).

This variant is compatible with Kubernetes 1.28, 1.29, 1.30, and 1.31 clusters.

### aws-ecs-1: Amazon ECS container instance

The [aws-ecs-1](aws-ecs-1/Cargo.toml) variant includes the packages needed to run an [Amazon ECS](https://ecs.aws)
container instance in AWS.

### aws-ecs-1-nvidia: Amazon ECS container instance

The [aws-ecs-1-nvidia](aws-ecs-1-nvidia/Cargo.toml) variant includes the packages needed to run an [Amazon ECS](https://ecs.aws)
container instance in AWS.
It also includes the required packages to configure containers to leverage NVIDIA GPUs.

### aws-ecs-2: Amazon ECS container instance

The [aws-ecs-2](aws-ecs-2/Cargo.toml) variant includes the packages needed to run an [Amazon ECS](https://ecs.aws)
container instance in AWS.

### aws-ecs-2-nvidia: Amazon ECS container instance

The [aws-ecs-2-nvidia](aws-ecs-2-nvidia/Cargo.toml) variant includes the packages needed to run an [Amazon ECS](https://ecs.aws)
container instance in AWS.
It also includes the required packages to configure containers to leverage NVIDIA GPUs.

### aws-dev: AWS development build

The [aws-dev](aws-dev/Cargo.toml) variant has useful packages for local development of the OS.
It includes tools for troubleshooting as well as Docker for running containers.
User data will be read from IMDS.

### vmware-dev: VMware development build

The [vmware-dev](vmware-dev/Cargo.toml) variant has useful packages for local development of the OS, and is intended to run as a VMware guest.
It includes tools for troubleshooting as well as Docker for running containers.

### vmware-k8s-1.23: VMware Kubernetes 1.23 node

The [vmware-k8s-1.23](vmware-k8s-1.23/Cargo.toml) variant includes the packages needed to run a Kubernetes worker node as a VMware guest.
It supports self-hosted clusters.

This variant is compatible with Kubernetes 1.23, 1.24, and 1.25 clusters.

### vmware-k8s-1.24: VMware Kubernetes 1.24 node

The [vmware-k8s-1.24](vmware-k8s-1.24/Cargo.toml) variant includes the packages needed to run a Kubernetes worker node as a VMware guest.
It supports self-hosted clusters.

This variant is compatible with Kubernetes 1.24, 1.25, and 1.26 clusters.

### vmware-k8s-1.25: VMware Kubernetes 1.25 node

The [vmware-k8s-1.25](vmware-k8s-1.25/Cargo.toml) variant includes the packages needed to run a Kubernetes worker node as a VMware guest.
It supports self-hosted clusters.

This variant is compatible with Kubernetes 1.25, 1.26, 1.27, and 1.28 clusters.

## vmware-k8s-1.26: VMware Kubernetes 1.26 node

The [vmware-k8s-1.26](vmware-k8s-1.26/Cargo.toml) variant includes the packages needed to run a Kubernetes worker node as a VMware guest.
It supports self-hosted clusters.

This variant is compatible with Kubernetes 1.26, 1.27, 1.28, and 1.29 clusters.

## vmware-k8s-1.27: VMware Kubernetes 1.27 node

The [vmware-k8s-1.27](vmware-k8s-1.27/Cargo.toml) variant includes the packages needed to run a Kubernetes worker node as a VMware guest.
It supports self-hosted clusters.

This variant is compatible with Kubernetes 1.27, 1.28, 1.29, and 1.30 clusters.

## vmware-k8s-1.28: VMware Kubernetes 1.28 node

The [vmware-k8s-1.27](vmware-k8s-1.28/Cargo.toml) variant includes the packages needed to run a Kubernetes worker node as a VMware guest.
It supports self-hosted clusters.

This variant is compatible with Kubernetes 1.28, 1.29, 1.30 and 1.31 clusters.

### metal-dev: Metal development build

The [metal-dev](metal-dev/Cargo.toml) variant has useful packages for local development of the OS and is intended to run bare metal.
It includes tools for troubleshooting as well as Docker for running containers.

### metal-k8s-1.23: Metal Kubernetes 1.23 node

The [metal-k8s-1.23](metal-k8s-1.23/Cargo.toml) variant includes the packages needed to run a Kubernetes node on bare metal.
It supports self-hosted clusters.

This variant is compatible with Kubernetes 1.23, 1.24, and 1.25 clusters.

### metal-k8s-1.24: Metal Kubernetes 1.24 node

The [metal-k8s-1.24](metal-k8s-1.24/Cargo.toml) variant includes the packages needed to run a Kubernetes node on bare metal.
It supports self-hosted clusters.

This variant is compatible with Kubernetes 1.24, 1.25, and 1.26 clusters.

### metal-k8s-1.25: Metal Kubernetes 1.25 node

The [metal-k8s-1.25](metal-k8s-1.25/Cargo.toml) variant includes the packages needed to run a Kubernetes node on bare metal.
It supports self-hosted clusters.

This variant is compatible with Kubernetes 1.25, 1.26, 1.27, and 1.28 clusters.

### metal-k8s-1.26: Metal Kubernetes 1.26 node

The [metal-k8s-1.26](metal-k8s-1.26/Cargo.toml) variant includes the packages needed to run a Kubernetes node on bare metal.
It supports self-hosted clusters.

This variant is compatible with Kubernetes 1.26, 1.27, 1.28, and 1.29 clusters.

### metal-k8s-1.27: Metal Kubernetes 1.27 node

The [metal-k8s-1.27](metal-k8s-1.27/Cargo.toml) variant includes the packages needed to run a Kubernetes node on bare metal.
It supports self-hosted clusters.

This variant is compatible with Kubernetes 1.27, 1.28, 1.29, and 1.30 clusters.

### metal-k8s-1.28: Metal Kubernetes 1.28 node

The [metal-k8s-1.28](metal-k8s-1.28/Cargo.toml) variant includes the packages needed to run a Kubernetes node on bare metal.
It supports self-hosted clusters.

This variant is compatible with Kubernetes 1.28, 1.29, 1.30, and 1.31 clusters.

### Deprecated variants

#### aws-k8s-1.15: Kubernetes 1.15 node

The aws-k8s-1.15 variant included the packages needed to run a Kubernetes node in AWS.
It supported self-hosted clusters and clusters managed by [EKS](https://aws.amazon.com/eks/).

This variant was compatible with Kubernetes 1.15, 1.16, and 1.17 clusters.
It reached end-of-life on May 3, 2021.

Upstream support for Kubernetes 1.15 has ended and this variant will no longer be supported in Bottlerocket releases.

### aws-k8s-1.16: Kubernetes 1.16 node

The aws-k8s-1.16 variant included the packages needed to run a Kubernetes node in AWS.
It supported self-hosted clusters and clusters managed by [EKS](https://aws.amazon.com/eks/).

This variant was compatible with Kubernetes 1.16, 1.17, and 1.18 clusters.
It reached end-of-life on July 25, 2021.

Upstream support for Kubernetes 1.16 has ended and this variant will no longer be supported in Bottlerocket releases.

### aws-k8s-1.17: Kubernetes 1.17 node

The aws-k8s-1.17 variant included the packages needed to run a Kubernetes node in AWS.
It supported self-hosted clusters and clusters managed by [EKS](https://aws.amazon.com/eks/).

This variant was compatible with Kubernetes 1.17, 1.18, and 1.19 clusters.
It reached end-of-life on November 2, 2021.

Upstream support for Kubernetes 1.17 has ended and this variant will no longer be supported in Bottlerocket releases.

### aws-k8s-1.18: Kubernetes 1.18 node

The aws-k8s-1.18 variant included the packages needed to run a Kubernetes node in AWS.
It supported self-hosted clusters and clusters managed by [EKS](https://aws.amazon.com/eks/).

This variant was compatible with Kubernetes 1.18, 1.19, and 1.20 clusters.
It reached end-of-life on March 31st, 2022.

Upstream support for Kubernetes 1.18 has ended and this variant will no longer be supported in Bottlerocket releases.

### aws-k8s-1.19: Kubernetes 1.19 node

The aws-k8s-1.19 variant included the packages needed to run a Kubernetes node in AWS.
It supported self-hosted clusters and clusters managed by [EKS](https://aws.amazon.com/eks/).

This variant was compatible with Kubernetes 1.19, 1.20, and 1.21 clusters.
It reached end-of-life on August 1st, 2022.

Upstream support for Kubernetes 1.19 has ended and this variant will no longer be supported in Bottlerocket releases.

### aws-k8s-1.20: Kubernetes 1.20 node

The aws-k8s-1.20 variant included the packages needed to run a Kubernetes node in AWS.
It supported self-hosted clusters and clusters managed by [EKS](https://aws.amazon.com/eks/).

This variant was compatible with Kubernetes 1.20, 1.21, and 1.22 clusters.
It reached end-of-life on November 1st, 2022.

Upstream support for Kubernetes 1.20 has ended and this variant will no longer be supported in Bottlerocket releases.

### vmware-k8s-1.20: VMware Kubernetes 1.20 node

The vmware-k8s-1.20 variant included the packages needed to run a Kubernetes worker node as a VMware guest.
It supported self-hosted clusters.

This variant was compatible with Kubernetes 1.20, 1.21, and 1.22 clusters.

### aws-k8s-1.21: Kubernetes 1.21 node

The aws-k8s-1.21 variant included the packages needed to run a Kubernetes node in AWS.
It supported self-hosted clusters and clusters managed by [EKS](https://aws.amazon.com/eks/).

This variant was compatible with Kubernetes 1.21, 1.22, and 1.23 clusters.

### aws-k8s-1.21-nvidia: Kubernetes 1.21 NVIDIA node

The aws-k8s-1.21-nvidia variant included the packages needed to run a Kubernetes node in AWS.
It also included the required packages to configure containers to leverage NVIDIA GPUs.
It supported self-hosted clusters and clusters managed by [EKS](https://aws.amazon.com/eks/).
This variant was compatible with Kubernetes 1.21, 1.22, and 1.23 clusters.

### metal-k8s-1.21: Metal Kubernetes 1.21 node

The metal-k8s-1.21 variant included the packages needed to run a Kubernetes node on bare metal.
It supported self-hosted clusters.

This variant was compatible with Kubernetes 1.21, 1.22, and 1.23 clusters.

### vmware-k8s-1.21: VMware Kubernetes 1.21 node

The vmware-k8s-1.21 variant included the packages needed to run a Kubernetes worker node as a VMware guest.
It supported self-hosted clusters.

This variant was compatible with Kubernetes 1.21, 1.22, and 1.23 clusters.

### aws-k8s-1.22: Kubernetes 1.22 node

The aws-k8s-1.22 variant included the packages needed to run a Kubernetes node in AWS.
It supported self-hosted clusters and clusters managed by [EKS](https://aws.amazon.com/eks/).

This variant was compatible with Kubernetes 1.22, 1.23, and 1.24 clusters.

### aws-k8s-1.22-nvidia: Kubernetes 1.22 NVIDIA node

The aws-k8s-1.22-nvidia variant included the packages needed to run a Kubernetes node in AWS.
It also included the required packages to configure containers to leverage NVIDIA GPUs.
It supported self-hosted clusters and clusters managed by [EKS](https://aws.amazon.com/eks/).

This variant was compatible with Kubernetes 1.22, 1.23, and 1.24 clusters.

### metal-k8s-1.22: Metal Kubernetes 1.22 node

The metal-k8s-1.22 variant included the packages needed to run a Kubernetes node on bare metal.
It supported self-hosted clusters.

This variant was compatible with Kubernetes 1.22, 1.23, and 1.24 clusters.

### vmware-k8s-1.22: VMware Kubernetes 1.22 node

The vmware-k8s-1.22 variant included the packages needed to run a Kubernetes worker node as a VMware guest.
It supported self-hosted clusters.

This variant was compatible with Kubernetes 1.22, 1.23, and 1.24 clusters.

## Development

Say we want to create `my-variant`, a custom build of Bottlerocket that runs `my-agent`.

### Structure
This listing shows the directory structure of our sample variant.

```
variants/my-variant
└── Cargo.toml
```

Each variant has a `Cargo.toml` file that lists the packages to install.

It also refers to a `build.rs` [build script](https://doc.rust-lang.org/cargo/reference/build-scripts.html) which tells Cargo to invoke our [buildsys](../tools/buildsys/) tool.
Artifacts for the variant are built as a side effect of Cargo running the script.

It points to `/dev/null` for the actual crate, since Cargo expects some Rust code to build, and is happy with an empty file.

### Cargo.toml

Our sample variant has the following manifest.

```toml
[package]
name = "my-variant"
version = "0.1.0"
edition = "2018"
publish = false
build = "../build.rs"

[package.metadata.build-variant]
included-packages = [
    "release",
    "my-agent",
]

[package.metadata.build-variant.image-layout]
os-image-size-gib = 8
data-image-size-gib = 20
partition-plan = "unified"

[lib]
path = "../variants.rs"

[build-dependencies]
"my-agent" = { path = "../../packages/my-agent" }
"release" = { path = "../../packages/release" }
```

The [package.metadata](https://doc.rust-lang.org/cargo/reference/manifest.html#the-metadata-table-optional) table is ignored by Cargo and interpreted by our `buildsys` tool.

It contains an `included-packages` list which specifies the packages to install when building the image.
In the `[build-dependencies]` section, we specify the packages that need to be built, which is sometimes slightly different than `included-packages`.
This populates the Cargo build graph with all of the RPM packages that need to be built before the variant can be constructed.
Variants should almost always include the `release` package.
This pulls in the other core packages and includes essential configuration and services.

This variant includes the (optional) `image-layout` section, which allows the user to customize the layout of the image they are building.
`os-image-size-gib` is the size of the "OS" disk image in GiB.
`data-image-size-gib` is the size of the "data" disk image in GiB.
Though we've done so here for sake of demonstration, resizing the "data" disk image isn't necessary as it expands to fill the disk on boot.
`partition-plan` is the strategy used for image partitioning, with the options being "split" (the default) or "unified".
The "split" partition strategy has separate volumes for "OS" and "data", while "unified" has "OS" and "data" on a single volume.
See [the documentation](../tools/buildsys/src/manifest.rs) for the defaults and additional details.

Be sure to include `publish = false` for all packages, as these are not standard crates and should never appear on [crates.io](https://crates.io/).

### build.rs

We reuse the same build script for all variants.

```rust
use std::process::{exit, Command};

fn main() -> Result<(), std::io::Error> {
    let ret = Command::new("buildsys").arg("build-variant").status()?;
    if !ret.success() {
        exit(1);
    }
    Ok(())
}
```

If you need a build script with different behavior, the recommended approach is to modify the `buildsys` tool.
The `package.metadata` table can be extended with declarative elements that enable the new feature.

### Next Steps

To build your variant, run the following command in the top-level Bottlerocket directory.
```shell
cargo make -e BUILDSYS_VARIANT=my-variant
```

This will build all packages first, not just the ones needed by your variant.
