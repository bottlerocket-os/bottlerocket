# ec2-identity-doc-user-data-provider

Current version: 0.1.0

## Introduction

User data provider binary used to generate user data from data in the EC2 instance identity document.

Currently used only to fetch the AWS region. Falls back to IMDS if the region is not found in the instance identity document.

## Colophon

This text was generated using [cargo-readme](https://crates.io/crates/cargo-readme), and includes the rustdoc from `src/main.rs`.
