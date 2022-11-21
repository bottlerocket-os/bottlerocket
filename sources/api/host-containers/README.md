# host-containers

Current version: 0.1.0

## Background

host-containers ensures that host containers are running as defined in system settings.

It queries the API for their settings, then configures the system by:
* creating a user-data file in the host container's persistent storage area, if a base64-encoded
  user-data setting is set for the host container.  (The decoded contents are available to the
  container at /.bottlerocket/host-containers/NAME/user-data)
* creating an environment file used by a host-container-specific instance of a systemd service
* ensuring the host container's systemd service is enabled/started or disabled/stopped

## Colophon

This text was generated using [cargo-readme](https://crates.io/crates/cargo-readme), and includes the rustdoc from `src/main.rs`.
