# imdsclient

Current version: 0.1.0

`imdsclient` provides high-level methods to interact with the AWS Instance Metadata Service (IMDS).

The library uses IMDSv2 (session-oriented) requests over a pinned schema to guarantee compatibility.
Session tokens are fetched automatically and refreshed if the request receives a `401` response.
If an IMDS token fetch or query fails, the library will continue to retry with a fibonacci backoff
strategy until it is successful or times out. The default timeout is 300s to match the ifup timeout
set in wicked.service, but can configured using `.with_timeout` during client creation.

Each public method is explicitly targeted and return either bytes or a `String`.

For example, if we need a piece of metadata, like `instance_type`, a method `fetch_instance_type`,
will create an IMDSv2 session _(if one does not already exist)_ and send a request to:

`http://169.254.169.254/2021-01-03/meta-data/instance-type`

The result is returned as a `String` _(ex. m5.large)_.

## Colophon

This text was generated from `README.tpl` using [cargo-readme](https://crates.io/crates/cargo-readme), and includes the rustdoc from `src/lib.rs`.
