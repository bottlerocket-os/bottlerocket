We currently package Rust binaries built upstream.
Here's how to update to a new version, for example to 1.37:

Import the Rust signing key if you don't have it:

```
curl https://static.rust-lang.org/rust-key.gpg.ascii | gpg --import
```

Get and check the artifacts.
**Make sure to update the versions at the top!**

```
RUST_VERSION=1.37.0
CARGO_VERSION=0.38.0
RELEASE_DATE=2019-08-15

curl -O "https://static.rust-lang.org/dist/${RELEASE_DATE}/rustc-${RUST_VERSION}-x86_64-unknown-linux-gnu.tar.xz"
curl -O "https://static.rust-lang.org/dist/${RELEASE_DATE}/rustc-${RUST_VERSION}-x86_64-unknown-linux-gnu.tar.xz.asc"
curl -O "https://static.rust-lang.org/dist/${RELEASE_DATE}/rustc-${RUST_VERSION}-x86_64-unknown-linux-gnu.tar.xz.sha256"
curl -O "https://static.rust-lang.org/dist/${RELEASE_DATE}/rustc-${RUST_VERSION}-aarch64-unknown-linux-gnu.tar.xz"
curl -O "https://static.rust-lang.org/dist/${RELEASE_DATE}/rustc-${RUST_VERSION}-aarch64-unknown-linux-gnu.tar.xz.asc"
curl -O "https://static.rust-lang.org/dist/${RELEASE_DATE}/rustc-${RUST_VERSION}-aarch64-unknown-linux-gnu.tar.xz.sha256"
curl -O "https://static.rust-lang.org/dist/${RELEASE_DATE}/cargo-${CARGO_VERSION}-x86_64-unknown-linux-gnu.tar.xz"
curl -O "https://static.rust-lang.org/dist/${RELEASE_DATE}/cargo-${CARGO_VERSION}-x86_64-unknown-linux-gnu.tar.xz.asc"
curl -O "https://static.rust-lang.org/dist/${RELEASE_DATE}/cargo-${CARGO_VERSION}-x86_64-unknown-linux-gnu.tar.xz.sha256"
curl -O "https://static.rust-lang.org/dist/${RELEASE_DATE}/cargo-${CARGO_VERSION}-aarch64-unknown-linux-gnu.tar.xz"
curl -O "https://static.rust-lang.org/dist/${RELEASE_DATE}/cargo-${CARGO_VERSION}-aarch64-unknown-linux-gnu.tar.xz.asc"
curl -O "https://static.rust-lang.org/dist/${RELEASE_DATE}/cargo-${CARGO_VERSION}-aarch64-unknown-linux-gnu.tar.xz.sha256"
curl -O "https://static.rust-lang.org/dist/${RELEASE_DATE}/rust-std-${RUST_VERSION}-x86_64-unknown-linux-gnu.tar.xz"
curl -O "https://static.rust-lang.org/dist/${RELEASE_DATE}/rust-std-${RUST_VERSION}-x86_64-unknown-linux-gnu.tar.xz.asc"
curl -O "https://static.rust-lang.org/dist/${RELEASE_DATE}/rust-std-${RUST_VERSION}-x86_64-unknown-linux-gnu.tar.xz.sha256"
curl -O "https://static.rust-lang.org/dist/${RELEASE_DATE}/rust-std-${RUST_VERSION}-aarch64-unknown-linux-gnu.tar.xz"
curl -O "https://static.rust-lang.org/dist/${RELEASE_DATE}/rust-std-${RUST_VERSION}-aarch64-unknown-linux-gnu.tar.xz.asc"
curl -O "https://static.rust-lang.org/dist/${RELEASE_DATE}/rust-std-${RUST_VERSION}-aarch64-unknown-linux-gnu.tar.xz.sha256"

for file in *.sha256; do sha256sum -c $file; done
for file in *.asc; do gpg --verify $file; done
```

Confirm that the sha256sum and gpg output said the files were OK.

In `packages/rust/`:

```
sed -i \
   -e "s/^Version: .*/Version: ${RUST_VERSION}" \
   -e "s/^%global cargo_version .*/%global cargo_version ${CARGO_VERSION}" \
   rust.spec
sha512sum --tag *xz > sources
```

Then prepare a commit and a PR!
