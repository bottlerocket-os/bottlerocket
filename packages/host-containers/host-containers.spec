%global workspace_name host-containers
%global systemd_systemdir %{_cross_libdir}/systemd/system

Name: %{_cross_os}%{workspace_name}
Version: 0.0
Release: 0%{?dist}
Summary: Thar host container management
License: FIXME
BuildRequires: gcc-%{_cross_target}
BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}golang
Requires: %{_cross_os}glibc

%description
%{summary}.

%prep
%setup -T -c
cp -r %{_builddir}/workspaces/%{workspace_name}/cmd/host-ctr/* .

%build
%set_cross_go_flags
GOPATH=%{buildroot} go build -mod=vendor -v -o host-ctr

%install
install -d %{buildroot}%{_cross_bindir}
install -p -m 0755 host-ctr %{buildroot}%{_cross_bindir}

%files
%{_cross_bindir}/host-ctr

%changelog
