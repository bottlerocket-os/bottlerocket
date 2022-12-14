# shibaken

Current version: 0.1.0

## Introduction

shibaken is called by sundog as a setting generator.

shibaken is used to fetch data from the instance metadata service (IMDS) in AWS.

shibaken can:
* Fetch and populate the admin container's user-data with authorized ssh keys from the IMDS.
* Perform boolean queries about the AWS partition in which the host is located.
* Wait in a warm pool until the instance is marked as InService before starting the orchestrator.

(The name "shibaken" comes from the fact that Shiba are small, but agile, hunting dogs.)

## Colophon

This text was generated from `README.tpl` using [cargo-readme](https://crates.io/crates/cargo-readme), and includes the rustdoc from `src/main.rs`.
