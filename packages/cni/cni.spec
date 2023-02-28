%global goproject github.com/containernetworking
%global gorepo cni
%global goimport %{goproject}/%{gorepo}

%global gover 1.1.2
%global rpmver %{gover}

%global _dwz_low_mem_die_limit 0

Name: %{_cross_os}%{gorepo}
Version: %{rpmver}
Release: 1%{?dist}
Summary: Plugins for container networking
License: Apache-2.0
URL: https://%{goimport}
Source0: https://%{goimport}/archive/v%{gover}/%{gorepo}-%{gover}.tar.gz
Source1: bundled-%{gorepo}-%{gover}.tar.gz
BuildRequires: git
BuildRequires: %{_cross_os}glibc-devel
Requires: %{_cross_os}iptables

%description
%{summary}.

%prep
%setup -n %{gorepo}-%{gover} -q
%setup -T -D -n %{gorepo}-%{gover} -b 1 -q

%build
%set_cross_go_flags

go build -buildmode=pie -ldflags="${GOLDFLAGS}" -o "bin/cnitool" %{goimport}/cnitool

%install
install -d %{buildroot}%{_cross_libexecdir}/cni/bin
install -p -m 0755 bin/cnitool %{buildroot}%{_cross_libexecdir}/cni/bin

%cross_scan_attribution go-vendor vendor

%files
%license LICENSE
%{_cross_attribution_file}
%{_cross_attribution_vendor_dir}
%{_cross_libexecdir}/cni/bin/cnitool

%changelog
