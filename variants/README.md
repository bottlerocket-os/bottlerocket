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

## Variants

### aws-k8s-1.15: Kubernetes 1.15 node

The [aws-k8s-1.15](aws-k8s-1.15/Cargo.toml) variant includes the packages needed to run a Kubernetes node in AWS.
It supports self-hosted clusters and clusters managed by [EKS](https://aws.amazon.com/eks/).

This variant is compatible with Kubernetes 1.15, 1.16, and 1.17 clusters.

### aws-k8s-1.16: Kubernetes 1.16 node

The [aws-k8s-1.16](aws-k8s-1.16/Cargo.toml) variant includes the packages needed to run a Kubernetes node in AWS.
It supports self-hosted clusters and clusters managed by [EKS](https://aws.amazon.com/eks/).

This variant is compatible with Kubernetes 1.16, 1.17, and 1.18 clusters.

### aws-k8s-1.17: Kubernetes 1.17 node

The [aws-k8s-1.17](aws-k8s-1.17/Cargo.toml) variant includes the packages needed to run a Kubernetes node in AWS.
It supports self-hosted clusters and clusters managed by [EKS](https://aws.amazon.com/eks/).

This variant is compatible with Kubernetes 1.17, 1.18, and 1.19 clusters.

### aws-k8s-1.18: Kubernetes 1.18 node

The [aws-k8s-1.18](aws-k8s-1.18/Cargo.toml) variant includes the packages needed to run a Kubernetes node in AWS.
It supports self-hosted clusters and clusters managed by [EKS](https://aws.amazon.com/eks/).

This variant is compatible with Kubernetes 1.18, 1.19, and 1.20 clusters.

### aws-k8s-1.19: Kubernetes 1.19 node

The [aws-k8s-1.19](aws-k8s-1.19/Cargo.toml) variant includes the packages needed to run a Kubernetes node in AWS.
It supports self-hosted clusters and clusters managed by [EKS](https://aws.amazon.com/eks/).

This variant is compatible with Kubernetes 1.19, 1.20, and 1.21 clusters.

### aws-ecs-1: Amazon ECS container instance

The [aws-ecs-1](aws-ecs-1/Cargo.toml) variant includes the packages needed to run an [Amazon ECS](https://ecs.aws)
container instance in AWS.

### aws-dev: Development build

The [aws-dev](aws-dev/Cargo.toml) variant has useful packages for local development of the OS.
It includes tools for troubleshooting as well as Docker for running containers.

## Development

Say we want to create `my-variant`, a custom build of Bottlerocket that runs `my-agent`.

### Structure
This listing shows the directory structure of our sample variant.

```
variants/my-variant
├── Cargo.toml
├── build.rs
└── lib.rs
```

Each variant has a `Cargo.toml` file that lists the packages to install.

It also includes a `build.rs` [build script](https://doc.rust-lang.org/cargo/reference/build-scripts.html) which tells Cargo to invoke our [buildsys](../tools/buildsys/) tool.
Artifacts for the variant are built as a side effect of Cargo running the script.

It has an empty `lib.rs` for the actual crate, since Cargo expects some Rust code to build.

### Cargo.toml

Our sample variant has the following manifest.

```
[package]
name = "my-variant"
version = "0.1.0"
edition = "2018"
publish = false
build = "build.rs"

[package.metadata.build-variant]
included-packages = [
    "release",
    "my-agent",
]

[lib]
path = "lib.rs"
```

The [package.metadata](https://doc.rust-lang.org/cargo/reference/manifest.html#the-metadata-table-optional) table is ignored by Cargo and interpreted by our `buildsys` tool.

It contains an `included-packages` list which specifies the packages to install when building the image.
Variants should almost always include the `release` package.
This pulls in the other core packages and includes essential configuration and services.

Be sure to include `publish = false` for all packages, as these are not standard crates and should never appear on [crates.io](https://crates.io/).

### build.rs

We use the same build script for all variants.

```
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

### lib.rs

We use the same Rust code for all variants.

```
// not used
```

### Next Steps

To build your variant, run the following command in the top-level Bottlerocket directory.
```
cargo make -e BUILDSYS_VARIANT=my-variant
```

This will build all packages first, not just the ones needed by your variant.
