%global goproject github.com/docker
%global gorepo engine
%global goimport %{goproject}/%{gorepo}

# Docker's remote repository location does not match its canonical
# import path, so we define macros for that as well.
%global dorepo docker
%global doimport %{goproject}/%{dorepo}

%global gover 18.09.9
%global rpmver %{gover}

%global _dwz_low_mem_die_limit 0

Name: %{_cross_os}docker-%{gorepo}
Version: %{rpmver}
Release: 1%{?dist}
Summary: Docker engine
License: ASL 2.0
URL: https://%{goimport}
Source0: https://%{goimport}/archive/v%{gover}/%{gorepo}-%{gover}.tar.gz
Source1: docker.service
Source2: docker.socket
Source3: docker-sysusers.conf
Source4: daemon.json
Source5: docker-tmpfiles.conf

BuildRequires: git
BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}libseccomp-devel
BuildRequires: %{_cross_os}systemd-devel
Requires: %{_cross_os}containerd
Requires: %{_cross_os}libseccomp
Requires: %{_cross_os}iptables
Requires: %{_cross_os}systemd

%description
%{summary}.

%prep
%autosetup -Sgit -n %{gorepo}-%{gover} -p1
%cross_go_setup %{gorepo}-%{gover} %{goproject} %{doimport}

%build
%cross_go_configure %{doimport}
BUILDTAGS="journald rpm_crashtraceback selinux seccomp"
BUILDTAGS+=" exclude_graphdriver_btrfs"
BUILDTAGS+=" exclude_graphdriver_devicemapper"
BUILDTAGS+=" exclude_graphdriver_vfs"
BUILDTAGS+=" exclude_graphdriver_zfs"
export BUILDTAGS
go build -buildmode pie -tags="${BUILDTAGS}" -o dockerd %{doimport}/cmd/dockerd

%install
install -d %{buildroot}%{_cross_bindir}
install -p -m 0755 dockerd %{buildroot}%{_cross_bindir}

install -d %{buildroot}%{_cross_unitdir}
install -p -m 0644 %{S:1} %{buildroot}%{_cross_unitdir}/docker.service
install -p -m 0644 %{S:2} %{buildroot}%{_cross_unitdir}/docker.socket

install -d %{buildroot}%{_cross_sysusersdir}
install -p -m 0644 %{S:3} %{buildroot}%{_cross_sysusersdir}/docker.conf

install -d %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}/docker
install -p -m 0644 %{S:4} %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}/docker/daemon.json

install -d %{buildroot}%{_cross_tmpfilesdir}
install -p -m 0644 %{S:5} %{buildroot}%{_cross_tmpfilesdir}/docker.conf

%files
%{_cross_bindir}/dockerd
%{_cross_unitdir}/docker.service
%{_cross_unitdir}/docker.socket
%{_cross_sysusersdir}/docker.conf
%{_cross_factorydir}%{_cross_sysconfdir}/docker
%{_cross_tmpfilesdir}/docker.conf

%changelog
