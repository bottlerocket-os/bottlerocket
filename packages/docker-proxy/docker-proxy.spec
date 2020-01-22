%global goproject github.com/docker
%global gorepo libnetwork
%global goimport %{goproject}/%{gorepo}
%global commit 48722da498b202dfed2eb4299dfcfbdf8b75392d

%global _dwz_low_mem_die_limit 0

Name: %{_cross_os}docker-proxy
Version: 18.09.9
Release: 1%{?dist}
Summary: Docker CLI
# mostly Apache-2.0, client/mflag is BSD-3-Clause
License: Apache-2.0 AND BSD-3-Clause
URL: https://%{goimport}
Source0: https://%{goimport}/archive/%{commit}/%{gorepo}-%{commit}.tar.gz
Source1000: clarify.toml
Patch1: 0001-bridge-Fix-hwaddr-set-race-between-us-and-udev.patch

BuildRequires: git
BuildRequires: %{_cross_os}glibc-devel

%description
%{summary}.

%prep
%autosetup -Sgit -n %{gorepo}-%{commit} -p1
%cross_go_setup %{gorepo}-%{commit} %{goproject} %{goimport}

cp client/mflag/LICENSE LICENSE.mflag

%build
%cross_go_configure %{goimport}
export BUILDTAGS="rpm_crashtraceback"
go build -buildmode pie -tags="${BUILDTAGS}" -o docker-proxy %{goimport}/cmd/proxy

%install
install -d %{buildroot}%{_cross_bindir}
install -p -m 0755 docker-proxy %{buildroot}%{_cross_bindir}

%cross_scan_attribution --clarify %{S:1000} go-vendor vendor

%files
%license LICENSE LICENSE.mflag
%{_cross_attribution_file}
%{_cross_attribution_vendor_dir}
%{_cross_bindir}/docker-proxy

%changelog
