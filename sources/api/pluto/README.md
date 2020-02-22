# pluto

Current version: 0.1.0

## Introduction

pluto is called by sundog to generate settings required by Kubernetes.
This is done dynamically because we require access to dynamic networking
setup information.

It makes calls to IMDS to get meta data:

- Cluster DNS
- Node IP
- POD Infra Container Image

## Colophon 

This text was generated using [cargo-readme](https://crates.io/crates/cargo-readme), and includes the rustdoc from `src/main.rs`.