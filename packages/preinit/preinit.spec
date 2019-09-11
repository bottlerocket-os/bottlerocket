%global workspace_name preinit
%global workspace_dir %{_builddir}/workspaces/%{workspace_name}
%undefine _debugsource_packages

Name: %{_cross_os}%{workspace_name}
Version: 0.0
Release: 0%{?dist}
Summary: Thar pre-init system setup
License: FIXME
BuildRequires: gcc-%{_cross_target}
BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}rust
Requires: %{_cross_os}glibc

%description
%{summary}.

%prep
%setup -T -c
%cargo_prep

%build
%cargo_build --path %{workspace_dir}/laika

%install
install -d %{buildroot}%{_cross_sbindir}
install -p -m 0755 bin/preinit %{buildroot}%{_cross_sbindir}

%files
%{_cross_sbindir}/preinit

%changelog
