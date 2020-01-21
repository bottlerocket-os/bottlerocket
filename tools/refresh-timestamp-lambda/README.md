# refresh-timestamp-lambda

Current version: 0.1.0

## Introduction

This is a lambda function that periodically refreshes a TUF repository's `timestamp.json` metadata file's expiration date and version.

Every time this lambda runs, the expiration date is pushed out by a custom number of days from the current date (defined by the lambda event).

## Compiling & Building

This rust lambda needs to be statically compiled and linked against [musl-libc](https://www.musl-libc.org/).
Currently building with [clux/muslrust](https://github.com/clux/muslrust).

To build, run `make build`.
Then, to zip the lambda bootstrap binary, run `make zip`.

## Setting up the Lambda with CloudFormation

Use `timestamp-signer.yaml` to create an assumable role in the account where the signing key resides. This lets the lambda have access to the signing key.

Use `tuf-repo-access-role.yaml` to create an assumable role in the account where the TUF repository bucket resides. This lets the lambda have access to update `timestamp.json`.

Use `TimestampRefreshLambda.yaml` to create the CFN stack for this lambda.


## Colophon

This text was generated from `README.tpl` using [cargo-readme](https://crates.io/crates/cargo-readme), and includes the rustdoc from `src/main.rs`.