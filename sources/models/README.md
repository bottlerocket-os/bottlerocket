# models

Current version: 0.1.0

## API models

Bottlerocket has different variants supporting different features and use cases.
Each variant has its own set of software, and therefore needs its own configuration.
We support having an API model for each variant to support these different configurations.

Each model defines a top-level `Settings` structure.
It can use pre-defined structures inside, or custom ones as needed.

This `Settings` essentially becomes the schema for the variant's data store.
`apiserver::datastore` offers serialization and deserialization modules that make it easy to map between Rust types and the data store, and thus, all inputs and outputs are type-checked.

At the field level, standard Rust types can be used, or ["modeled types"](src/modeled_types) that add input validation.

Default values are specified in .toml files in each variant's `defaults.d` directory under [src](src).
(For example, see the [aws-ecs-1 defaults](src/aws-ecs-1/defaults.d/).)
Entries are sorted by filename, and later entries take precedence.

The `#[model]` attribute on Settings and its sub-structs reduces duplication and adds some required metadata; see [its docs](model-derive/) for details.

### aws-k8s-1.23: Kubernetes 1.23

* [Model](src/aws-k8s-1.25/mod.rs)
* [Default settings](src/aws-k8s-1.25/defaults.d/)

### aws-k8s-1.23-nvidia: Kubernetes 1.23 NVIDIA

* [Model](src/aws-k8s-1.25-nvidia/mod.rs)
* [Default settings](src/aws-k8s-1.25-nvidia/defaults.d/)

### aws-k8s-1.24: Kubernetes 1.24

* [Model](src/aws-k8s-1.25/mod.rs)
* [Default settings](src/aws-k8s-1.25/defaults.d/)

### aws-k8s-1.24-nvidia: Kubernetes 1.24 NVIDIA

* [Model](src/aws-k8s-1.25-nvidia/mod.rs)
* [Default settings](src/aws-k8s-1.25-nvidia/defaults.d/)

### aws-k8s-1.25: Kubernetes 1.25

* [Model](src/aws-k8s-1.25/mod.rs)
* [Default settings](src/aws-k8s-1.25/defaults.d/)

### aws-k8s-1.25-nvidia: Kubernetes 1.25 NVIDIA

* [Model](src/aws-k8s-1.25-nvidia/mod.rs)
* [Default settings](src/aws-k8s-1.25-nvidia/defaults.d/)

### aws-k8s-1.26: Kubernetes 1.26

* [Model](src/aws-k8s-1.26/mod.rs)
* [Default settings](src/aws-k8s-1.26/defaults.d/)

### aws-k8s-1.26-nvidia: Kubernetes 1.26 NVIDIA

* [Model](src/aws-k8s-1.26-nvidia/mod.rs)
* [Default settings](src/aws-k8s-1.26-nvidia/defaults.d/)

### aws-k8s-1.27: Kubernetes 1.27

* [Model](src/aws-k8s-1.27/mod.rs)
* [Default settings](src/aws-k8s-1.27/defaults.d/)

### aws-k8s-1.27-nvidia: Kubernetes 1.27 NVIDIA

* [Model](src/aws-k8s-1.27-nvidia/mod.rs)
* [Default settings](src/aws-k8s-1.27-nvidia/defaults.d/)

### aws-ecs-1: Amazon ECS

* [Model](src/aws-ecs-1/mod.rs)
* [Default settings](src/aws-ecs-1/defaults.d/)

### aws-ecs-1-nvidia: Amazon ECS NVIDIA

* [Model](src/aws-ecs-1-nvidia/mod.rs)
* [Default settings](src/aws-ecs-1-nvidia/defaults.d/)

### aws-ecs-2: Amazon ECS

* [Model](src/aws-ecs-1/mod.rs)
* [Default settings](src/aws-ecs-1/defaults.d/)

### aws-ecs-2-nvidia: Amazon ECS NVIDIA

* [Model](src/aws-ecs-1-nvidia/mod.rs)
* [Default settings](src/aws-ecs-1-nvidia/defaults.d/)

### aws-dev: AWS development build

* [Model](src/aws-dev/mod.rs)
* [Default settings](src/aws-dev/defaults.d/)

### vmware-dev: VMware development build

* [Model](src/vmware-dev/mod.rs)
* [Default settings](src/vmware-dev/defaults.d/)

### vmware-k8s-1.23: VMware Kubernetes 1.23

* [Model](src/vmware-k8s-1.27/mod.rs)
* [Default settings](src/vmware-k8s-1.27/defaults.d/)

### vmware-k8s-1.24: VMware Kubernetes 1.24

* [Model](src/vmware-k8s-1.27/mod.rs)
* [Default settings](src/vmware-k8s-1.27/defaults.d/)

### vmware-k8s-1.25: VMware Kubernetes 1.25

* [Model](src/vmware-k8s-1.27/mod.rs)
* [Default settings](src/vmware-k8s-1.27/defaults.d/)

### vmware-k8s-1.26: VMware Kubernetes 1.26

* [Model](src/vmware-k8s-1.27/mod.rs)
* [Default settings](src/vmware-k8s-1.27/defaults.d/)

### vmware-k8s-1.27: VMware Kubernetes 1.27

* [Model](src/vmware-k8s-1.27/mod.rs)
* [Default settings](src/vmware-k8s-1.27/defaults.d/)

### metal-dev: Metal development build

* [Model](src/metal-dev/mod.rs)
* [Default settings](src/metal-dev/defaults.d/)

### metal-k8s-1.23: Metal Kubernetes 1.23

* [Model](src/metal-k8s-1.27/mod.rs)
* [Default settings](src/metal-k8s-1.27/defaults.d/)

### metal-k8s-1.24: Metal Kubernetes 1.24

* [Model](src/metal-k8s-1.27/mod.rs)
* [Default settings](src/metal-k8s-1.27/defaults.d/)

### metal-k8s-1.25: Metal Kubernetes 1.25

* [Model](src/metal-k8s-1.27/mod.rs)
* [Default settings](src/metal-k8s-1.27/defaults.d/)

### metal-k8s-1.26: Metal Kubernetes 1.26

* [Model](src/metal-k8s-1.27/mod.rs)
* [Default settings](src/metal-k8s-1.27/defaults.d/)

### metal-k8s-1.27: Metal Kubernetes 1.27

* [Model](src/metal-k8s-1.27/mod.rs)
* [Default settings](src/metal-k8s-1.27/defaults.d/)

## This directory

We use `build.rs` to symlink the proper API model source code for Cargo to build.
We determine the "proper" model by using the `VARIANT` environment variable.

If a developer is doing a local `cargo build`, they need to set `VARIANT`.

When building with the Bottlerocket build system, `VARIANT` is based on `BUILDSYS_VARIANT` from the top-level `Makefile.toml`, which can be overridden on the command line with `cargo make -e BUILDSYS_VARIANT=bla`.

Note: when building with the build system, we can't create the symlink in the source directory during a build - the directories are owned by `root`, but we're `builder`.
We can't use a read/write bind mount with current Docker syntax.
To get around this, in the top-level `Dockerfile`, we mount a "cache" directory at `src/variant` that we can modify, and create a `current` symlink inside.
The code in `src/lib.rs` then imports the requested model using `variant/current`.

Note: for the same reason, we symlink `variant/mod.rs` to `variant_mod.rs`.
Rust needs a `mod.rs` file to understand that a directory is part of the module structure, so we have to have `variant/mod.rs`.
`variant/` is the cache mount that starts empty, so we have to store the file elsewhere and link it in.

Note: all models share the same `Cargo.toml`.

## Colophon

This text was generated from `README.tpl` using [cargo-readme](https://crates.io/crates/cargo-readme), and includes the rustdoc from `src/lib.rs`.
