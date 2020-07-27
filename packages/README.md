# Bottlerocket Packages

This document describes how packages are built for Bottlerocket.

In the [Background](#background) section, we discuss the general approach, the specific technologies in use, and the rationale behind it all.

In the [Development](#development) section, we provide a short guide for adding a new package.

## Background

Like any Linux distribution, Bottlerocket builds on a foundation of open source software, from the Linux kernel through the GNU C Library and more.
Unlike most distributions, it is not designed to be self-hosting.
We want to package and maintain only the software that we eventually ship, and not the software needed to build that software.

Bottlerocket makes extensive use of *cross compilation* as a result.
Cross compilation involves at least two conceptually distinct systems: the *host* and the *target*.
The *host* is another Linux distribution that provides the toolchain and other tools needed to build a modern Linux userspace.
This includes `perl`, `python`, `make`, and `meson`.
The *target* is Bottlerocket.
It includes only the packages found in this directory.

Because Bottlerocket uses image-based updates, it does not need a package manager - yet the packages are defined in RPM spec files and built using RPM.
Why?

The separation of responsibilities between host and target outlined above is not quite enough to achieve the goal of a minimal footprint.
Many of the packages we build contain both the shared libraries we need at runtime as well as the headers we need to build other software for our target.
RPM offers a familiar mechanism for separating the artifacts from a build into different packages.
For example, libseccomp is split into a `libseccomp` package for runtime use, and a `libseccomp-devel` package for building other packages.

With RPM it is idiomatic to define macros to standardize the invocation of scripts like `configure` and tools like `make`.
Since we are cross-compiling, many more environment variables must be set and arguments passed to ensure that builds use the target's toolchain and dependencies rather than the host's.
The spec files make extensive use of project-specific macros for this reason.

Macros also provide a way to ensure policy objectives are applied across all packages.
Examples include stripping debug symbols, collecting software license information, and running security checks.

A key aspect of building RPMs - or any software - is providing a consistent and clean build environment.
Otherwise, a prior build on the same host can change the result in surprising ways.
[mock](https://github.com/rpm-software-management/mock/wiki) is often used for this, either directly or by services such as [Koji](https://fedoraproject.org/wiki/Koji).

Bottlerocket uses Docker and containers to accomplish this instead.
Every package build starts from a container with the [Bottlerocket SDK](https://github.com/bottlerocket-os/bottlerocket-sdk) and zero or more other packages installed as dependencies.
Any source archives and patches needed for the build are copied in, and the binary RPMs are copied out once the build is complete.

Some packages depend on building other packages first.
To construct the dependency graph and invoke each build in order, we leverage [Cargo](https://doc.rust-lang.org/cargo/), the [Rust](https://www.rust-lang.org/) package manager.

## Development

Say we want to package `libwoof`, the C library that provides the reference implementation for the *WOOF* framework.

### Structure
This listing shows the directory structure of our sample package.

```
packages/libwoof/
├── Cargo.toml
├── build.rs
├── pkg.rs
├── libwoof.spec
```

Each package has a `Cargo.toml` file that lists its dependencies, along with metadata such as external files and the expected hashes.

It also includes a `build.rs` [build script](https://doc.rust-lang.org/cargo/reference/build-scripts.html) which tells Cargo to invoke our [buildsys](../tools/buildsys/) tool.
The RPM packages we want are built as a side effect of Cargo running the script.

It has an empty `pkg.rs` for the actual crate, since Cargo expects some Rust code to build.

Finally, it includes a `spec` file that defines the RPM.

### Cargo.toml

Our sample package has the following manifest.

```
[package]
name = "libwoof"
version = "0.1.0"
edition = "2018"
publish = false
build = "build.rs"

[lib]
path = "pkg.rs"

[[package.metadata.build-package.external-files]]
url = "http://downloads.sourceforge.net/libwoof/libwoof-1.0.0.tar.xz"
sha512 = "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e"

[build-dependencies]
glibc = { path = "../glibc" }
libseccomp = { path = "../libseccomp" }
```

The [package.metadata](https://doc.rust-lang.org/cargo/reference/manifest.html#the-metadata-table-optional) table is ignored by Cargo and interpreted by our `buildsys` tool.

It contains an `external-files` list which provides upstream URLs and expected hashes.
These files are, by default, only fetched from our upstream source mirror, using the URL template `https://cache.bottlerocket.aws/{file}/{sha512}/{file}`.
(If `file` is not defined, the text after the last `/` character in `url` is used.)

If your source is not yet available in the upstream source mirror, you can run `cargo make` with `-e BUILDSYS_UPSTREAM_SOURCE_FALLBACK=true`.

The [build-dependencies](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#build-dependencies) table is used to declare a dependency on another crate.
In this case, `libwoof` depends on `glibc` and `libseccomp`.
We want those libraries to be built first, and for this one to be rebuilt when they are modified.

Be sure to include `publish = false` for all packages, as these are not standard crates and should never appear on [crates.io](https://crates.io/).

### build.rs

We use the same build script for all packages.

```
use std::process::{exit, Command};

fn main() -> Result<(), std::io::Error> {
    let ret = Command::new("buildsys").arg("build-package").status()?;
    if !ret.success() {
        exit(1);
    }
    Ok(())
}
```

If you need a build script with different behavior, the recommended approach is to modify the `buildsys` tool.
The `package.metadata` table can be extended with declarative elements that enable the new feature.

### pkg.rs

We use the same Rust code for all packages.

```
// not used
```

### spec

Spec files will vary widely across packages, and a full guide to RPM packaging is out of scope here.

```
Name: %{_cross_os}libwoof
Version: 1.0.0
Release: 1%{?dist}
Summary: Library for woof
License: Apache-2.0 OR MIT
URL: http://sourceforge.net/projects/libwoof/
Source0: http://downloads.sourceforge.net/libwoof/libwoof-1.0.0.tar.xz
BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}libseccomp-devel

%description
%{summary}.

%package devel
Summary: Files for development using the library for woof
Requires: %{name}

%description devel
%{summary}.

%prep
%autosetup -n libwoof-%{version} -p1

%build
%cross_configure

%make_build

%install
%make_install

%files
%license LICENSE
%{_cross_libdir}/*.so.*

%files devel
%{_cross_libdir}/*.so
%dir %{_cross_includedir}/libwoof
%{_cross_includedir}/libwoof/*.h
%{_cross_pkgconfigdir}/*.pc

%changelog
```

Macros start with `%`.
If the macro is specific to Bottlerocket, it will include the `cross` token.
The definitions for most of these can be found in [macros](../macros).
The definition for `%{_cross_variant}` is the Bottlerocket variant being built.

When developing a package on an RPM-based system, you can expand the macros with a command like this.
```
$ PKG=libwoof
$ rpmspec \
  --macros "/usr/lib/rpm/macros:macros/$(uname -m):macros/shared:macros/rust:macros/cargo" \
  --define "_sourcedir packages/${PKG}" \
  --parse packages/${PKG}/${PKG}.spec
```

### Next Steps

After the above files are in place, the package must be added to the `packages` workspace.

Open `packages/Cargo.toml` in your editor and update the `members` list.
```
[workspace]
members = [
    ...
    "libwoof",
    ...
]
```

Next, run `cargo generate-lockfile` to refresh `Cargo.lock`.

To build your package, run the following command in the top-level Bottlerocket directory.
```
cargo make
```

This will build all packages, including your new one.
