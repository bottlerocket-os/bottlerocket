%global workspace_name growpart
%global workspace_dir %{_builddir}/workspaces/%{workspace_name}
%undefine _debugsource_packages

Name: %{_cross_os}%{workspace_name}
Version: 0.0
Release: 0%{?dist}
Summary: Tool to grow partitions
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
%cargo_build --path %{workspace_dir}

%install
install -d %{buildroot}%{_cross_sbindir}
install -p -m 0755 bin/growpart %{buildroot}%{_cross_sbindir}

%files
%{_cross_sbindir}/growpart

%changelog
