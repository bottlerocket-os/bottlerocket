%global workspace_name preinit

Name: %{_cross_os}%{workspace_name}
Version: 0.0
Release: 0%{?dist}
Summary: Thar pre-init system setup
License: FIXME
Source0: %{workspace_name}.crate
%cargo_bundle_crates -n %{workspace_name} -t 0
BuildRequires: gcc-%{_cross_target}
BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}rust
Requires: %{_cross_os}glibc

%description
%{summary}.

%package -n %{_cross_os}laika
Summary: Thar pre-init agent
%description -n %{_cross_os}laika
%{summary}.

%prep
%setup -qn %{workspace_name}
%cargo_prep

%build
%cargo_build --all

%check
%cargo_test --all

%install
%cargo_install -p laika
install -d %{buildroot}%{_cross_sbindir}
mv %{buildroot}%{_cross_bindir}/preinit %{buildroot}%{_cross_sbindir}/preinit

%files
%{_cross_sbindir}/preinit

%changelog
