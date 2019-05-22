%global workspace_name api
%global systemd_systemdir %{_cross_libdir}/systemd/system

Name: %{_cross_os}%{workspace_name}
Version: 0.0
Release: 0%{?dist}
Summary: Thar API packages
License: Apache-2.0 AND (Apache-2.0 OR BSL-1.0) AND (Apache-2.0 OR MIT) AND Apache-2.0/MIT AND BSD-2-Clause AND BSD-3-Clause AND CC0-1.0 AND ISC AND MIT AND (MIT OR Apache-2.0) AND MIT/Unlicense AND N/A AND (Unlicense OR MIT) AND Zlib
Source0: %{workspace_name}.crate
Source1: apiserver.service
Source2: moondog.service
%cargo_bundle_crates -n %{workspace_name} -t 0
BuildRequires: gcc-%{_cross_target}
BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}rust
Requires: %{_cross_os}glibc

%description
%{summary}.

%package -n %{_cross_os}apiserver
Summary: Thar API server
%description -n %{_cross_os}apiserver
%{summary}.

%package -n %{_cross_os}moondog
Summary: Thar userdata configuration system
Requires: %{_cross_os}apiserver = %{version}-%{release}
%description -n %{_cross_os}moondog
%{summary}.

%package -n %{_cross_os}thar-be-settings
Summary: Applies changed settings to a Thar system
Requires: %{_cross_os}apiserver = %{version}-%{release}
%description -n %{_cross_os}thar-be-settings
%{summary}.

%prep
%setup -qn %{workspace_name}
%cargo_prep

%build
%cargo_build --all

%check
%cargo_test --all

%install
mkdir -p %{buildroot}/%{systemd_systemdir}
install -m 0644 -t %{buildroot}/%{systemd_systemdir} %{SOURCE1}
install -m 0644 -t %{buildroot}/%{systemd_systemdir} %{SOURCE2}

%cargo_install -p apiserver
%cargo_install -p moondog
%cargo_install -p thar-be-settings

%files -n %{_cross_os}apiserver
%{_cross_bindir}/apiserver
%{systemd_systemdir}/apiserver.service

%files -n %{_cross_os}moondog
%{_cross_bindir}/moondog
%{systemd_systemdir}/moondog.service

%files -n %{_cross_os}thar-be-settings
%{_cross_bindir}/thar-be-settings

%changelog
