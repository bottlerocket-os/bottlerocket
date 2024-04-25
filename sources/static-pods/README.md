# static-pods

Current version: 0.1.0

## Background

static-pods ensures static pods are running as defined in settings.

It queries for all existing static pod settings, then configures the system as follows:
* If the pod is enabled, it creates the manifest file in the pod manifest path that kubelet is
  configured to read from and populates the file with the base64-decoded manifest setting value.
* If the pod is enabled and the manifest file already exists, it overwrites the existing manifest
  file with the base64-decoded manifest setting value.
* If the pod is disabled, it ensures the manifest file is removed from the pod manifest path.

## Colophon

This text was generated using [cargo-readme](https://crates.io/crates/cargo-readme), and includes the rustdoc from `src/static_pods.rs`.
