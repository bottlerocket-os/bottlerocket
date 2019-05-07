%global crate_name signpost

Name: %{_cross_os}%{crate_name}
Version: 0.0
Release: 0%{?dist}
Summary: Thar GPT priority querier/switcher
# cargo-license output:
# Apache-2.0 (1): signpost
# Apache-2.0 OR MIT (1): uuid
# Apache-2.0/MIT (17): rand_core, rand_isaac, winapi-i686-pc-windows-gnu, rand_pcg, rand_os, bitflags, autocfg, rand_hc, rand_jitter, cfg-if, winapi-x86_64-pc-windows-gnu, rand_xorshift, rand_chacha, log, rand_core, winapi, rand
# BSD-2-Clause (1): cloudabi
# ISC (1): rdrand
# MIT (2): build_const, gpt
# MIT OR Apache-2.0 (2): libc, crc
# N/A (1): fuchsia-cprng
#
# fuchsia-cprng is BSD: https://fuchsia.googlesource.com/fuchsia/+/master/LICENSE
License: ASL 2.0 and (ASL 2.0 or MIT) and BSD and ISC
Source0: %{crate_name}.crate
%cargo_bundle_crates -n %{crate_name} -t 0
BuildRequires: gcc-%{_cross_target}
BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}rust
Requires: %{_cross_os}glibc

%description
%{summary}.

%prep
%setup -qn %{crate_name}
%cargo_prep

%build
%cargo_build

%install
%cargo_install

%files
%{_cross_bindir}/signpost

%changelog
