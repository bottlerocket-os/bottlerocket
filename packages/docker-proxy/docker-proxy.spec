%global goproject github.com/docker
%global gorepo libnetwork
%global goimport %{goproject}/%{gorepo}
%global commit 872f0a83c98add6cae255c8859e29532febc0039

%global _dwz_low_mem_die_limit 0

Name: %{_cross_os}docker-proxy
Version: 18.09.6
Release: 1%{?dist}
Summary: Docker CLI
License: ASL 2.0
URL: https://%{goimport}
Source0: https://%{goimport}/archive/%{commit}/%{gorepo}-%{commit}.tar.gz
Patch1: 0001-bridge-Fix-hwaddr-set-race-between-us-and-udev.patch

BuildRequires: git
BuildRequires: gcc-%{_cross_target}
BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}golang
Requires: %{_cross_os}glibc

%description
%{summary}.

%prep
%autosetup -Sgit -n %{gorepo}-%{commit} -p1
mkdir -p GOPATH/src/%{goproject}
ln -s %{_builddir}/%{gorepo}-%{commit} GOPATH/src/%{goimport}

%build
cd GOPATH/src/%{goimport}
export CC="%{_cross_target}-gcc"
export GOPATH="${PWD}/GOPATH"
export GOARCH="%{_cross_go_arch}"
export PKG_CONFIG_PATH="%{_cross_pkgconfigdir}"
export BUILDTAGS="rpm_crashtraceback"
go build -buildmode pie -tags="${BUILDTAGS}" -o docker-proxy %{goimport}/cmd/proxy

%install
install -d %{buildroot}%{_cross_bindir}
install -p -m 0755 docker-proxy %{buildroot}%{_cross_bindir}

%files
%{_cross_bindir}/docker-proxy

%changelog
