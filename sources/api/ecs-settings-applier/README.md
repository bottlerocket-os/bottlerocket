# ecs-settings-applier

Current version: 0.1.0

## Introduction

ecs-settings-applier generates a configuration file for the ECS agent from Bottlerocket settings.

The configuration file for ECS is a JSON-formatted document with conditionally-defined keys and
embedded lists.  The structure and names of fields in the document can be found
[here](https://github.com/aws/amazon-ecs-agent/blob/a250409cf5eb4ad84a7b889023f1e4d2e274b7ab/agent/config/types.go).

## Colophon

This text was generated using from `README.tpl` [cargo-readme](https://crates.io/crates/cargo-readme), and includes the rustdoc from `src/main.rs`.
