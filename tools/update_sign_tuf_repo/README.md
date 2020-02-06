# update_sign_tuf_repo

Current version: 0.1.0

## Introduction

This tool is meant to update an existing TUF repo with new contents and sign the updated contents.
Given a set of environment variables, it will pull down an existing TUF repo and update the manifest, targets.json, snapshot.json, and timestamp.json.
Using a signing key that it pulls down via SSM Secure Parameters, it will sign the updated files, along with any new targets and leave them in a known location to be deployed to a "real" TUF repo at a later step.

## Running

In order the run this code, you must have:
* Current `Thar` code repository (more specifically `Release.toml`, and a trusted `root.json`)
* Built Thar artifacts in a directory (the images that end up in `/build` and suffixed with `.lz4`)
* The metadata and target URLs for an existing TUF repository (most likely in S3)

Currently the code expects the following environment variables to be set:
* `CODEBUILD_SRC_DIR` (subject to change) This is the directory where your `Thar` repository lives
* `ARCH` : architecture for your current set of images (i.e. `x86_64`)
* `FLAVOR` : Variant of Thar for your current set of images (i.e. `aws-k8s`)
* `INPUT_BUILDSYS_ARTIFACTS` : A directory containing the built Thar images
* `METADATA_URL` : Metadata URL for your existing TUF repo
* `TARGET_URL` : Target URL for your existing TUF repo
* `REFRESH_DAYS` : After how many days does metadata expire? (an integer, i.e. `7`)
* `TIMESTAMP_REFRESH_DAYS` : After how many days does `timestamp.json` expire? (an integer, i.e. `7`)
* `SIGNING_ROLE_ARN` : ARN for a role that allows access to signing keys (most likely in another account)
* `SIGNING_KEY_PARAMETER_NAME` : The SSM parameter key name for the signing key

## Output

After a successful run of this code, you will have a directory `/tmp/tuf_out` which will contain `/metadata` and `/target` directories.
All items (other than `manifest.json`) are signed and are suitable for syncing to your "real" TUF repository.

## Colophon

This text was generated using [cargo-readme](https://crates.io/crates/cargo-readme), and includes the rustdoc from `src/main.rs`.
