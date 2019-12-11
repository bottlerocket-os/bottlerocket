# API models

Thar has different variants supporting different features and use cases.
Each variant has its own set of software, and therefore needs its own configuration.
We support having an API model for each variant to support these different configurations.

## aws-k8s: Kubernetes

* [Model](aws-k8s/lib.rs)
* [Defaults](aws-k8s/defaults.toml)

## aws-dev: Development build

* [Model](aws-dev/lib.rs)
* [Defaults](aws-dev/defaults.toml)

# This directory

We use `build.rs` to symlink the proper API model source code for Cargo to build.
We determine the "proper" model by using the `VARIANT` environment variable.

If a developer is doing a local `cargo build`, they need to set `VARIANT`.

When building with the Thar build system, `VARIANT` is based on `BUILDSYS_VARIANT` from the top-level `Makefile.toml`, which can be overridden on the command line with `cargo make -e BUILDSYS_VARIANT=bla`.

Note: when building with the build system, we can't create the symlink in the source directory during a build - the directories are owned by `root`, but we're `builder`.
We can't use a read/write bind mount with current Docker syntax.
To get around this, in the top-level `Dockerfile`, we mount a "cache" directory at `current` that we can modify.
We set Cargo (via `Cargo.toml`) to look for the source at `current/src`, rather than the default `src`.

Note: all models share the same `Cargo.toml`.
