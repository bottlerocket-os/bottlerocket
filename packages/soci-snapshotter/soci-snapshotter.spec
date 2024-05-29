%global gorepo soci-snapshotter
%global gover 0.5.0
%global rpmver %{gover}

Name: %{_cross_os}soci-snapshotter
Version: %{gover}
Release: 1%{?dist}
Summary: A containerd snapshotter plugin which enables lazy loading for OCI images.
License: Apache-2.0
URL: https://github.com/awslabs/soci-snapshotter
Source0: https://github.com/awslabs/soci-snapshotter/archive/refs/tags/v0.5.0.tar.gz
Source1: bundled-v%{gover}.tar.gz
Source2: bundled-cmd.tar.gz
Source101: soci-snapshotter.service
Source1000: clarify.toml

BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}libz-devel
Requires: %{name}(binaries)

%description
%{summary}.

%package bin
Summary: Remote management agent binaries
Provides: %{name}(binaries)
Requires: (%{_cross_os}image-feature(no-fips) and %{name})
Conflicts: (%{_cross_os}image-feature(fips) or %{name}-fips-bin)

%description bin
%{summary}.

%package fips-bin
Summary: Remote management agent binaries, FIPS edition
Provides: %{name}(binaries)
Requires: (%{_cross_os}image-feature(fips) and %{name})
Conflicts: (%{_cross_os}image-feature(no-fips) or %{name}-bin)

%description fips-bin
%{summary}.

%prep
%setup -n %{gorepo}-%{gover} -q
%setup -T -D -n %{gorepo}-%{gover} -b 1 -q
%setup -T -D -n %{gorepo}-%{gover} -b 2 -q

%build
%set_cross_go_flags

go build -C cmd -ldflags="${GOLDFLAGS}" -o "../out/soci-snapshotter-grpc" ./soci-snapshotter-grpc
go build -C cmd -ldflags="${GOLDFLAGS}" -o "../out/soci" ./soci

gofips build -C cmd -ldflags="${GOLDFLAGS}" -o "../out/fips/soci-snapshotter-grpc" ./soci-snapshotter-grpc
gofips build -C cmd -ldflags="${GOLDFLAGS}" -o "../out/fips/soci" ./soci

%install
install -d %{buildroot}%{_cross_bindir}
install -d %{buildroot}%{_cross_fips_bindir}
install -d %{buildroot}%{_cross_unitdir}
install -p -m 0755 out/soci-snapshotter-grpc %{buildroot}%{_cross_bindir}
install -p -m 0755 out/soci %{buildroot}%{_cross_bindir}
install -p -m 0755 out/fips/soci-snapshotter-grpc %{buildroot}%{_cross_fips_bindir}
install -p -m 0755 out/fips/soci %{buildroot}%{_cross_fips_bindir}
install -D -p -m 0644 %{S:101} %{buildroot}%{_cross_unitdir}

%cross_scan_attribution --clarify %{S:1000} go-vendor vendor

%files
%license LICENSE NOTICE.md
%{_cross_unitdir}/soci-snapshotter.service
%{_cross_attribution_vendor_dir}
%{_cross_attribution_file}

%files bin
%{_cross_bindir}/soci-snapshotter-grpc
%{_cross_bindir}/soci

%files fips-bin
%{_cross_fips_bindir}/soci-snapshotter-grpc
%{_cross_fips_bindir}/soci

%changelog
