%global project moby
%global repo github.com/moby/%{project}
%global goorg github.com/docker
%global goimport %{goorg}/docker

%global gover 19.03.14
%global rpmver %{gover}
%global gitrev 9dc6525e6118a25fab2be322d1914740ea842495

%global source_date_epoch 1363394400

%global _dwz_low_mem_die_limit 0

Name: %{_cross_os}docker-engine
Version: %{rpmver}
Release: 1%{?dist}
Summary: Docker engine
License: Apache-2.0
URL: https://%{repo}
Source0: https://%{repo}/archive/v%{gover}/%{project}-%{gover}.tar.gz
Source1: docker.service
Source2: docker.socket
Source3: docker-sysusers.conf
Source4: daemon.json
Source5: docker-tmpfiles.conf
Source1000: clarify.toml

# Bottlerocket-specific - Privileged containers should receive SELinux labels
# https://github.com/bottlerocket-os/bottlerocket/issues/1011
Patch0001: 0001-bottlerocket-privileged-shouldn-t-disable-SELinux.patch

BuildRequires: git
BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}libseccomp-devel
BuildRequires: %{_cross_os}systemd-devel
Requires: %{_cross_os}containerd
Requires: %{_cross_os}libseccomp
Requires: %{_cross_os}iptables
Requires: %{_cross_os}systemd
Requires: %{_cross_os}procps

%description
%{summary}.

%prep
%autosetup -Sgit -n %{project}-%{gover} -p1
%cross_go_setup %{project}-%{gover} %{goorg} %{goimport}

%build
%cross_go_configure %{goimport}
BUILDTAGS="autogen journald selinux seccomp"
BUILDTAGS+=" exclude_graphdriver_btrfs"
BUILDTAGS+=" exclude_graphdriver_devicemapper"
BUILDTAGS+=" exclude_graphdriver_vfs"
BUILDTAGS+=" exclude_graphdriver_zfs"
export BUILDTAGS
export VERSION=%{gover}
export GITCOMMIT=%{gitrev}
export BUILDTIME=$(date -u -d "@%{source_date_epoch}" --rfc-3339 ns 2> /dev/null | sed -e 's/ /T/')
export PLATFORM="Docker Engine - Community"
chmod +x ./hack/make/.go-autogen
./hack/make/.go-autogen
go build -buildmode=pie -ldflags=-linkmode=external -tags="${BUILDTAGS}" -o dockerd %{goimport}/cmd/dockerd

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

%cross_scan_attribution --clarify %{S:1000} go-vendor vendor

%files
%{_cross_attribution_file}
%{_cross_attribution_vendor_dir}
%{_cross_bindir}/dockerd
%{_cross_unitdir}/docker.service
%{_cross_unitdir}/docker.socket
%{_cross_sysusersdir}/docker.conf
%{_cross_factorydir}%{_cross_sysconfdir}/docker
%{_cross_tmpfilesdir}/docker.conf

%changelog
