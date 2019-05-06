%global crate_name ripgrep

Name: %{_cross_os}%{crate_name}
Version: 11.0.1
Release: 1%{?dist}
Summary: Search tool
# cargo-license output:
# Apache-2.0 OR BSL-1.0 (1): ryu
# Apache-2.0/MIT (47): crossbeam-utils, smallvec, crossbeam-channel, log, rand_isaac, pkg-config, glob, winapi, quote, unicode-width, rand_jitter, base64, remove_dir_all, serde_derive, jemallocator, autocfg, bitflags, memmap, thread_local, encoding_rs, ucd-util, num_cpus, regex-syntax, rand_core, regex, rand, tempfile, jemalloc-sys, cfg-if, itoa, rand_pcg, bytecount, rand_chacha, proc-macro2, serde_json, rand_xorshift, unicode-xid, rand_os, winapi-x86_64-pc-windows-gnu, serde, cc, syn, winapi-i686-pc-windows-gnu, rand_core, fnv, rand_hc, lazy_static
# BSD-2-Clause (1): cloudabi
# ISC (1): rdrand
# MIT (8): strsim, redox_syscall, fs_extra, clap, termion, atty, textwrap, redox_termios
# MIT OR Apache-2.0 (3): bstr, libc, encoding_rs_io
# MIT/Unlicense (19): aho-corasick, regex-automata, grep-printer, grep-searcher, pcre2, utf8-ranges, grep-regex, grep, grep-matcher, grep-cli, memchr, walkdir, pcre2-sys, grep-pcre2, same-file, ignore, winapi-util, wincolor, globset
# N/A (1): fuchsia-cprng
# Unlicense OR MIT (3): termcolor, byteorder, ripgrep
#
# fuchsia-cprng is BSD: https://fuchsia.googlesource.com/fuchsia/+/master/LICENSE
License: ASL 2.0 and (ASL 2.0 or BSL 1.0) and MIT and Unlicense and BSD and ISC
Source0: https://github.com/BurntSushi/ripgrep/archive/%{version}/%{crate_name}-%{version}.tar.gz
%cargo_bundle_crates -n %{crate_name}-%{version} -t 0
BuildRequires: gcc-%{_cross_target}
BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}rust
Requires: %{_cross_os}glibc

%description
%{summary}.

%prep
%setup -qn %{crate_name}-%{version}
%cargo_prep

%build
%cargo_build

%install
%cargo_install

%files
%{_cross_bindir}/rg

%changelog
