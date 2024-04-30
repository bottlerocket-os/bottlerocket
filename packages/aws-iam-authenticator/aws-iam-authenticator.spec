%global goproject github.com/kubernetes-sigs
%global gorepo aws-iam-authenticator
%global goimport %{goproject}/%{gorepo}

%global gover 0.6.14
%global rpmver %{gover}

%global _dwz_low_mem_die_limit 0

Name: %{_cross_os}%{gorepo}
Version: %{rpmver}
Release: 1%{?dist}
Summary: AWS IAM authenticator
License: Apache-2.0
URL: https://%{goimport}
Source0: https://%{goimport}/archive/v%{gover}/%{gorepo}-%{gover}.tar.gz
Source1: bundled-aws-iam-authenticator-%{gover}.tar.gz
Source1000: clarify.toml

BuildRequires: git
BuildRequires: %{_cross_os}glibc-devel
Requires: %{name}(binaries)

%description
%{summary}.

%package bin
Summary: AWS IAM authenticator binaries
Provides: %{name}(binaries)
Requires: (%{_cross_os}image-feature(no-fips) and %{name})
Conflicts: (%{_cross_os}image-feature(fips) or %{name}-fips-bin)

%description bin
%{summary}.

%package fips-bin
Summary: AWS IAM authenticator binaries, FIPS edition
Provides: %{name}(binaries)
Requires: (%{_cross_os}image-feature(fips) and %{name})
Conflicts: (%{_cross_os}image-feature(no-fips) or %{name}-bin)

%description fips-bin
%{summary}.

%prep
%setup -n %{gorepo}-%{gover} -q
%setup -T -D -n %{gorepo}-%{version} -b 1

%build
%set_cross_go_flags
go build -ldflags="${GOLDFLAGS}" -o aws-iam-authenticator ./cmd/aws-iam-authenticator
gofips build -ldflags="${GOLDFLAGS}" -o fips/aws-iam-authenticator ./cmd/aws-iam-authenticator

%install
install -d %{buildroot}%{_cross_bindir}
install -p -m 0755 aws-iam-authenticator %{buildroot}%{_cross_bindir}

install -d %{buildroot}%{_cross_fips_bindir}
install -p -m 0755 fips/aws-iam-authenticator %{buildroot}%{_cross_fips_bindir}

%cross_scan_attribution --clarify %{S:1000} go-vendor vendor

%files
%license LICENSE
%{_cross_attribution_file}
%{_cross_attribution_vendor_dir}

%files bin
%{_cross_bindir}/aws-iam-authenticator

%files fips-bin
%{_cross_fips_bindir}/aws-iam-authenticator

%changelog
