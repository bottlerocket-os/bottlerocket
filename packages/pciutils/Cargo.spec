[package]
name = "pciutils"
version = "3.9.0"
edition = "2023"
publish = false
build = "build.rs"

[lib]
path = "pkg.rs"

[package.metadata.build-package]
releases-url = "https://mj.ucw.cz/download/linux/pci/"

[[package.metadata.build-package.external-files]]
url = "https://mj.ucw.cz/download/linux/pci/pciutils-3.9.0.tar.gz"
sha512 = "e17225c2adcc21c9ff4253998aec5805ae5e031888fa01841a1ff680796f7515f9dd6e5c2e0588edba854f66f1268ba8e28ae1a2f794574e715fec8a8c8def4f"

# RPM BuildRequires
[build-dependencies]
glibc = { path = "../glibc" }

# RPM Requires
[dependencies]
# `iptables` is only needed at runtime, and is pulled in by `release`.
# iptables = { path = "../iptables" }
