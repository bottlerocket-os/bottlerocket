%global goproject github.com/aws
%global gorepo rolesanywhere-credential-helper
%global goimport %{goproject}/%{gorepo}

%global gover 1.1.1
%global rpmver %{gover}

%global _dwz_low_mem_die_limit 0

Name: %{_cross_os}aws-signing-helper
Version: %{rpmver}
Release: 1%{?dist}
Summary: AWS signing helper for IAM Roles Anywhere support
License: Apache-2.0
URL: https://github.com/aws/rolesanywhere-credential-helper

Source: rolesanywhere-credential-helper-v%{gover}.tar.gz
Source1: bundled-rolesanywhere-credential-helper-v%{gover}.tar.gz

BuildRequires: %{_cross_os}glibc-devel
Requires: %{name}(binaries)

%description
%{summary}.

%package bin
Summary: AWS signing helper binaries
Provides: %{name}(binaries)
Requires: (%{_cross_os}image-feature(no-fips) and %{name})
Conflicts: (%{_cross_os}image-feature(fips) or %{name}-fips-bin)

%description bin
%{summary}.

%package fips-bin
Summary: AWS signing helper binaries, FIPS edition
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

go build -ldflags "-X 'main.Version=${gover}' ${GOLDFLAGS}" -o aws-signing-helper main.go
gofips build -ldflags "-X 'main.Version=${gover}' ${GOLDFLAGS}" -o fips/aws-signing-helper main.go

%install
install -d %{buildroot}%{_cross_bindir}
install -p -m 0755 aws-signing-helper %{buildroot}%{_cross_bindir}/aws_signing_helper
ln -sf aws_signing_helper %{buildroot}%{_cross_bindir}/aws-signing-helper

install -d %{buildroot}%{_cross_fips_bindir}
install -p -m 0755 fips/aws-signing-helper %{buildroot}%{_cross_fips_bindir}/aws_signing_helper
ln -sf aws_signing_helper %{buildroot}%{_cross_fips_bindir}/aws-signing-helper

%cross_scan_attribution go-vendor vendor

%files
%license LICENSE
%{_cross_attribution_file}
%{_cross_attribution_vendor_dir}

%files bin
%{_cross_bindir}/aws_signing_helper
%{_cross_bindir}/aws-signing-helper

%files fips-bin
%{_cross_fips_bindir}/aws_signing_helper
%{_cross_fips_bindir}/aws-signing-helper
