%global goproject github.com/awslabs
%global gorepo soci-snapshotter
%global goimport %{goproject}/%{gorepo}

%global gover 0.6.0
%global rpmver %{gover}

%global _dwz_low_mem_die_limit 0

Name: %{_cross_os}soci-snapshotter
Version: %{rpmver}
Release: 1%{?dist}
Summary: Amazon ECR credential provider
License: Apache-2.0
URL: https://github.com/awslabs/soci-snapshotter

Source: cloud-provider-aws-%{gover}.tar.gz

BuildRequires: %{_cross_os}glibc-devel
Requires: %{name}(binaries)

%description
%{summary}.

%package bin
Summary: SOCI Snapshotter binaries
Provides: %{name}(binaries)
Requires: (%{_cross_os}image-feature(no-fips) and %{name})
Conflicts: (%{_cross_os}image-feature(fips) or %{name}-fips-bin)

%description bin
%{summary}.

%package fips-bin
Summary: SOCI Snapshotter binaries, FIPS edition
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

go build -ldflags="${GOLDFLAGS}" -o=soci-snapshotter cmd/soci-snapshotter-grpc/*.go
gofips build -ldflags="${GOLDFLAGS}" -o=fips/soci-snapshotter cmd/soci-snapshotter-grpc/*.go

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
%{_cross_bindir}/soci-snapshotter-grpc

%files fips-bin
%{_cross_fips_bindir}/soci-snapshotter-grpc
