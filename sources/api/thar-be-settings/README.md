# thar-be-settings

Current version: 0.1.0

## Background

thar-be-settings is a simple configuration applier.
Its job is to update configuration files and restart services, as necessary, to make the system reflect any changes to settings.

In the normal ("specific keys") mode, it's intended to be called by the Bottlerocket API server after a settings commit.
It's told the keys that changed, and then queries metadata APIs to determine which services and configuration files are affected by changes to those keys.
Detailed data is then fetched for the relevant services and configuration files.
Configuration file data from the API includes paths to template files for each configuration file, along with the final path to write.
It then renders the templates and rewrites the affected configuration files.
Service data from the API includes any commands needed to restart services affected by configuration file changes, which are run here.

In the standalone ("all keys") mode, it queries the API for all services and configuration files, then renders and rewrites all configuration files and restarts all services.

## Colophon

This text was generated using [cargo-readme](https://crates.io/crates/cargo-readme), and includes the rustdoc from `src/lib.rs`.
