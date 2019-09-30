We currently package Rust binaries built upstream.
Here's how to update to a new version, for example to 1.38:

Import the Rust signing key if you don't have it:

```
curl https://static.rust-lang.org/rust-key.gpg.ascii | gpg --import
```

Get and check the artifacts.
**Make sure to update the versions at the top!**

```
RUST_VERSION=1.38.0
CARGO_VERSION=0.39.0
RELEASE_DATE=2019-09-26

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

for file in *.sha256; do sha256sum -c $file && echo OK || echo ERROR; done
for file in *.asc; do gpg --verify $file && echo OK || echo ERROR; done
```

Confirm that the curl, sha256sum, and gpg commands finished and said the files were OK.
Then you can remove the verification files:

```
rm *.asc *.sha256
```

Finally, update the packaging information.
In `packages/rust/`:

```
sed -i \
   -e "s/^Version: .*/Version: ${RUST_VERSION}/" \
   -e "s/^%global cargo_version .*/%global cargo_version ${CARGO_VERSION}/" \
   rust.spec

sed -i -e '/After this point, the file is generated/q' Cargo.toml
sha512sum *.tar.xz | awk '{print "\n[[package.metadata.build-package.external-files]]\nurl = \"https://static.rust-lang.org/dist/" $2 "\"\nsha512 = \"" $1 "\""}' >> Cargo.toml
```

Now you can remove the artifacts:

```
rm *.tar*
```

Then prepare a commit and a PR!
See #309 for an example.
