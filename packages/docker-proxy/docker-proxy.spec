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
BuildRequires: %{_cross_os}glibc-devel

%description
%{summary}.

%prep
%autosetup -Sgit -n %{gorepo}-%{commit} -p1
%cross_go_setup %{gorepo}-%{commit} %{goproject} %{goimport}

%build
%cross_go_configure %{goimport}
export BUILDTAGS="rpm_crashtraceback"
go build -buildmode pie -tags="${BUILDTAGS}" -o docker-proxy %{goimport}/cmd/proxy

%install
install -d %{buildroot}%{_cross_bindir}
install -p -m 0755 docker-proxy %{buildroot}%{_cross_bindir}

%files
%{_cross_bindir}/docker-proxy

%changelog
