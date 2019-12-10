# repo-canary

Current version: 0.1.0

## Introduction

`repo-canary` is a TUF repository canary that validates a specified TUF repository using [tough](https://crates.io/crates/tough).

It validates by loading the repository, checking the metadata files and attempting retrieval of its listed targets.

If any `tough` library error is encountered at any step of the validation process, a non-zero exit code is returned.
Exit codes are mapped to specific `tough` library errors as follows:

| `tough` error             | exit code |
| -------------             |-------    |
| `VerifyTrustedMetadata`   | 64        |
| `VerifyMetadata`          | 65        |
| `VersionMismatch`         | 66        |
| `Transport`               | 67        |
| `ExpiredMetadata`         | 68        |
| `MetaMissing`             | 69        |
| `OlderMetadata`           | 70        |


Other exit code to errors mappings:

| Other errors              | exit code |
| -------------             |-------    |
| Missing target in repo    | 71        |
| Failed to download target | 72        |
| *Metadata about to expire | 73        |

(*: see `--check-upcoming-expiration-days` option in usage info)


## Colophon

This text was generated from `README.tpl` using [cargo-readme](https://crates.io/crates/cargo-readme), and includes the rustdoc from `src/main.rs`.