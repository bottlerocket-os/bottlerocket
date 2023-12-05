%global _cross_first_party 1
%undefine _debugsource_packages

Name: %{_cross_os}early-boot-config
Version: 0.0
Release: 0%{?dist}
Summary: early-boot-config
License: Apache-2.0 OR MIT
URL: https://github.com/bottlerocket-os/bottlerocket

Source100: early-boot-config.service

BuildRequires: %{_cross_os}glibc-devel

%description
%{summary}.

%prep
%setup -T -c
%cargo_prep

%build
%cargo_build --manifest-path %{_builddir}/sources/Cargo.toml \
    -p early-boot-config --bin early-boot-config

%install
install -d %{buildroot}%{_cross_bindir}
install -p -m 0755 ${HOME}/.cache/%{__cargo_target}/release/early-boot-config %{buildroot}%{_cross_bindir}

install -d %{buildroot}%{_cross_unitdir}
install -p -m 0644 %{S:100} %{buildroot}%{_cross_unitdir}


%files
%{_cross_bindir}/early-boot-config
%{_cross_unitdir}/early-boot-config.service
