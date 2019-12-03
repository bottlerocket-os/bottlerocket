%global workspace_name host-ctr

Name: %{_cross_os}%{workspace_name}
Version: 0.0
Release: 0%{?dist}
Summary: Thar host container runner
License: FIXME
BuildRequires: %{_cross_os}glibc-devel
Requires: %{_cross_os}containerd

Source10: host-containerd.service
Source11: host-containerd-tmpfiles.conf
Source12: host-containerd-config.toml

%description
%{summary}.

%prep
%setup -T -c
cp -r %{_builddir}/workspaces/%{workspace_name}/cmd/host-ctr/* .

%build
%set_cross_go_flags
export BUILDTAGS="rpm_crashtraceback"
go build -buildmode=pie -tags="${BUILDTAGS}" -o host-ctr

%install
install -d %{buildroot}%{_cross_bindir}
install -p -m 0755 host-ctr %{buildroot}%{_cross_bindir}

install -d %{buildroot}%{_cross_unitdir}
install -p -m 0644 %{S:10} %{buildroot}%{_cross_unitdir}

install -d %{buildroot}%{_cross_tmpfilesdir}
install -p -m 0644 %{S:11} %{buildroot}%{_cross_tmpfilesdir}/host-containerd.conf

install -d %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}/host-containerd
install -p -m 0644 %{S:12} %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}/host-containerd/config.toml

%files
%{_cross_bindir}/host-ctr
%{_cross_unitdir}/host-containerd.service
%{_cross_tmpfilesdir}/host-containerd.conf
%{_cross_factorydir}%{_cross_sysconfdir}/host-containerd/config.toml

%changelog
