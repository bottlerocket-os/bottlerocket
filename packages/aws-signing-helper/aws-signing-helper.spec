%global goproject github.com/aws
%global gorepo rolesanywhere-credential-helper
%global goimport %{goproject}/%{gorepo}

%global gover 1.0.4
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

%description
%{summary}.

%prep
%setup -n %{gorepo}-%{gover} -q
%setup -T -D -n %{gorepo}-%{gover} -b 1 -q

%build
%set_cross_go_flags

go build ${GOFLAGS} -buildmode=pie -ldflags "-X 'main.Version=${gover}' ${GOLDFLAGS}" -o aws-signing-helper cmd/aws_signing_helper/main.go

%install
install -d %{buildroot}%{_cross_bindir}
install -p -m 0755 aws-signing-helper %{buildroot}%{_cross_bindir}/aws_signing_helper
ln -sf aws_signing_helper %{buildroot}%{_cross_bindir}/aws-signing-helper

%cross_scan_attribution go-vendor vendor

%files
%license LICENSE
%{_cross_attribution_file}
%{_cross_attribution_vendor_dir}
%{_cross_bindir}/aws_signing_helper
%{_cross_bindir}/aws-signing-helper
