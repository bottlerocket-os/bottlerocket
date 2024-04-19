# dogtag

Current version: 0.1.0

dogtag detects the hostname of a bottlerocket server/instance. It's used to generate settings.network.hostname.
To accomplish this, it uses a set of standalone binaries in /usr/libexec/hostname-detectors that detect the hostname via different methods.

Currently, bottlerocket ships with two hostname detector binaries:

20-imds - Fetches hostname from EC2 Instance Metadata Service
10-reverse-dns - Uses reverse DNS lookup to resolve the hostname

dogtag runs the detectors in /usr/libexec/hostname-detectors in reverse alphanumerical order until one of them returns a hostname,
at which point it will exit early and print the returned hostname to stdout. If none of the detectors detect the hostname the
ip address is returned.

## Colophon

This text was generated from `README.tpl` using [cargo-readme](https://crates.io/crates/cargo-readme), and includes the rustdoc from `src/main.rs`.
