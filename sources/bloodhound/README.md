# bloodhound

Current version: 0.1.0

## Introduction

Bloodhound is a command line orchestrator for running a set of compliance
checks. This can be used to run CIS benchmark compliance, though it can be extended
to perform any kind of check that adheres to the expected checker interface.

Checks are performed and their results are provided in an overall report.
The checker report can be written to a file, or viewed from stdout.
By default the report is provided in a human readable text format, but can also
be generated as JSON to make it easy to consume programmatically for integrating
into further compliance automation.

## Usage

Bloodhound is ultimately intended to be used through the Bottlerocket `apiclient`
interface.
If executing directly, run `bloodhound --help` for usage information.

## Colophon

This text was generated from `README.tpl` using [cargo-readme](https://crates.io/crates/cargo-readme), and includes the rustdoc from `src/main.rs`.
