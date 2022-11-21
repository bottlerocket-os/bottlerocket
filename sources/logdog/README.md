# logdog

Current version: 0.1.0

## Introduction

`logdog` is a program that gathers logs from various places on a Bottlerocket host and combines them
into a tarball for easy export.

Usage example:

```shell
$ logdog
logs are at: /var/log/support/bottlerocket-logs.tar.gz
```

## Logs

For the log requests used to gather logs, please see the following:

* [log_request](src/log_request.rs)
* [logdog.common.conf](conf/logdog.common.conf)
* And the variant-specific files in [conf](conf/), one of which is selected by [build.rs](build.rs)
based on the value of the `VARIANT` environment variable at build time.


## Colophon

This text was generated from `README.tpl` using [cargo-readme](https://crates.io/crates/cargo-readme), and includes the rustdoc from `src/main.rs`.
