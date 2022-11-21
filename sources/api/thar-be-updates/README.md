# thar-be-updates

Current version: 0.1.0

## Introduction

thar-be-updates is a Bottlerocket update dispatcher that serves as an interface for the `apiserver` to issue update commands and monitor update status.

It models the Bottlerocket update process after a state machine and provides several update commands that modifies the update state.
It keeps track of the update state and other stateful update information in a update status file located at `/run/update-status`

Upon receiving a command not allowed by the update state, thar-be-updates exits immediately with an exit status indicating so.
Otherwise, thar-be-updates forks a child process to spawn the necessary process to do the work.
The parent process immediately returns back to the caller with an exit status of `0`.
The output and status of the command will be written to the update status file.
This allows the caller to synchronously call thar-be-updates without having to wait for a result to come back.

thar-be-updates uses a lockfile to control read/write access to the disks and the update status file.


## Colophon

This text was generated from `README.tpl` using [cargo-readme](https://crates.io/crates/cargo-readme), and includes the rustdoc from `src/main.rs`.
