%global goproject github.com/kubernetes
%global gorepo cloud-provider-aws
%global goimport %{goproject}/%{gorepo}

%global gover 1.29.0
# %%global gover 1.30.0
%global rpmver %{gover}

%global _dwz_low_mem_die_limit 0

Name: %{_cross_os}ecr-credential-provider-1.30
Version: %{rpmver}
Release: 1%{?dist}
Summary: Amazon ECR credential provider
License: Apache-2.0
URL: https://github.com/kubernetes/cloud-provider-aws

Source: cloud-provider-aws-%{gover}.tar.gz
Source1: bundled-cloud-provider-aws-%{gover}.tar.gz
Source1000: clarify.toml

BuildRequires: %{_cross_os}glibc-devel
Requires: %{name}(binaries)

%description
%{summary}.

%package bin
Summary: Amazon ECR credential provider binaries
Provides: %{name}(binaries)
Requires: (%{_cross_os}image-feature(no-fips) and %{name})
Conflicts: (%{_cross_os}image-feature(fips) or %{name}-fips-bin)

%description bin
%{summary}.

%package fips-bin
Summary: Amazon ECR credential provider binaries, FIPS edition
Provides: %{name}(binaries)
Requires: (%{_cross_os}image-feature(fips) and %{name})
Conflicts: (%{_cross_os}image-feature(no-fips) or %{name}-bin)

%description fips-bin
%{summary}.

%prep
%setup -n %{gorepo}-%{gover} -q
%setup -T -D -n %{gorepo}-%{gover} -b 1 -q

%build
%set_cross_go_flags

go build -ldflags="${GOLDFLAGS}" -o=ecr-credential-provider cmd/ecr-credential-provider/*.go
gofips build -ldflags="${GOLDFLAGS}" -o=fips/ecr-credential-provider cmd/ecr-credential-provider/*.go

%install
install -d %{buildroot}%{_cross_libexecdir}/kubernetes/kubelet/plugins
install -p -m 0755 ecr-credential-provider %{buildroot}%{_cross_libexecdir}/kubernetes/kubelet/plugins

install -d %{buildroot}%{_cross_fips_libexecdir}/kubernetes/kubelet/plugins
install -p -m 0755 fips/ecr-credential-provider %{buildroot}%{_cross_fips_libexecdir}/kubernetes/kubelet/plugins

%cross_scan_attribution --clarify %{S:1000} go-vendor vendor

%files
%license LICENSE
%{_cross_attribution_file}
%{_cross_attribution_vendor_dir}

%files bin
%{_cross_libexecdir}/kubernetes/kubelet/plugins/ecr-credential-provider

%files fips-bin
%{_cross_fips_libexecdir}/kubernetes/kubelet/plugins/ecr-credential-provider
