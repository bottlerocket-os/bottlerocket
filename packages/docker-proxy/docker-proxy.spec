%global goproject github.com/docker
%global gorepo libnetwork
%global goimport %{goproject}/%{gorepo}
# Use the libnetwork commit listed in this file for the docker version we ship:
# https://github.com/moby/moby/blob/DOCKER-VERSION-HERE/vendor.conf
%global commit 0dde5c895075df6e3630e76f750a447cf63f4789

%global _dwz_low_mem_die_limit 0

Name: %{_cross_os}docker-proxy
Version: 20.10.18
Release: 1%{?dist}
Summary: Docker CLI
# mostly Apache-2.0, client/mflag is BSD-3-Clause
License: Apache-2.0 AND BSD-3-Clause
URL: https://%{goimport}
Source0: https://%{goimport}/archive/%{commit}/%{gorepo}-%{commit}.tar.gz
Source1000: clarify.toml

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
go build -buildmode=pie -ldflags="${GOLDFLAGS}" -o docker-proxy %{goimport}/cmd/proxy

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
