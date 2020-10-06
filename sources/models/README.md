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

Default values are specified in [defaults.toml](defaults.toml) and can be overridden by each variant.

The `#[model]` attribute on Settings and its sub-structs reduces duplication and adds some required metadata; see [its docs](model-derive/) for details.

### aws-k8s-1.15: Kubernetes 1.15

* [Model](src/aws-k8s-1.15/mod.rs)
* [Overridden defaults](src/aws-k8s-1.15/override-defaults.toml)

### aws-k8s-1.16: Kubernetes 1.16

* [Model](src/aws-k8s-1.16/mod.rs)
* [Overridden defaults](src/aws-k8s-1.16/override-defaults.toml)

### aws-k8s-1.17: Kubernetes 1.17

* [Model](src/aws-k8s-1.17/mod.rs)
* [Overridden defaults](src/aws-k8s-1.17/override-defaults.toml)

### aws-k8s-1.18: Kubernetes 1.18

* [Model](src/aws-k8s-1.18/mod.rs)
* [Overridden defaults](src/aws-k8s-1.18/override-defaults.toml)

### aws-ecs-1: Amazon ECS

* [Model](src/aws-ecs-1/mod.rs)
* [Overridden defaults](src/aws-ecs-1/override-defaults.toml)

### aws-dev: Development build

* [Model](src/aws-dev/mod.rs)
* [Overridden defaults](src/aws-dev/override-defaults.toml)

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