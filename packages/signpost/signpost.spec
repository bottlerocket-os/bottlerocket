%global crate_name signpost

Name: %{_cross_os}%{crate_name}
Version: 0.0
Release: 0%{?dist}
Summary: Thar GPT priority querier/switcher
# cargo-license output:
# Apache-2.0/MIT (9): autocfg, serde, serde_derive, proc-macro-hack, syn, proc-macro2, serde_plain, quote, unicode-xid
# MIT (3): build_const, gptman, bincode
# MIT OR Apache-2.0 (5): hex-literal, snafu-derive, crc, hex-literal-impl, snafu
# N/A (1): signpost
# Unlicense OR MIT (1): byteorder
License: Unknown AND (Apache-2.0 OR MIT) AND MIT AND (MIT OR Unlicense)
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
