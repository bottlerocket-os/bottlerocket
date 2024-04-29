%global _cross_first_party 1
%undefine _debugsource_packages

Name: %{_cross_os}static-pods
Version: 0.1
Release: 0%{?dist}
Summary: Manages user-defined K*S static pods
License: Apache-2.0 OR MIT
URL: https://github.com/bottlerocket-os/bottlerocket

Source0: static-pods-toml

BuildRequires: %{_cross_os}glibc-devel

%description
%{summary}.

%prep
%setup -T -c
%cargo_prep

%build
mkdir bin

echo "** Compile static-pods agent"
%cargo_build --manifest-path %{_builddir}/sources/Cargo.toml \
    -p static-pods

%install
install -d %{buildroot}%{_cross_bindir}
install -p -m 0755 ${HOME}/.cache/%{__cargo_target}/release/static-pods %{buildroot}%{_cross_bindir}

install -d %{buildroot}%{_cross_templatedir}
install -p -m 0644 %{S:0} %{buildroot}%{_cross_templatedir}

%files
%{_cross_bindir}/static-pods
%{_cross_templatedir}/static-pods-toml
