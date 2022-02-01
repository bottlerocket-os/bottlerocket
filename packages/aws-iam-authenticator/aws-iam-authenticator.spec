%global goproject github.com/kubernetes-sigs
%global gorepo aws-iam-authenticator
%global goimport %{goproject}/%{gorepo}

%global gover 0.5.3
%global rpmver %{gover}

%global _dwz_low_mem_die_limit 0

Name: %{_cross_os}%{gorepo}
Version: %{rpmver}
Release: 1%{?dist}
Summary: AWS IAM authenticator
License: Apache-2.0
URL: https://%{goimport}
Source0: https://%{goimport}/archive/v%{gover}/%{gorepo}-%{gover}.tar.gz
Source1000: clarify.toml

# 0.5.3 release did not include necessary vendored dependencies.
Patch0001: aws-iam-authenticator-0.5.3-Update-vendored-go-dependencies.patch

BuildRequires: git
BuildRequires: %{_cross_os}glibc-devel

%description
%{summary}.

%prep
%autosetup -Sgit -n %{gorepo}-%{gover} -p1

%build
%set_cross_go_flags
go build -buildmode=pie -ldflags="${GOLDFLAGS}" -o aws-iam-authenticator ./cmd/aws-iam-authenticator

%install
install -d %{buildroot}%{_cross_bindir}
install -p -m 0755 aws-iam-authenticator %{buildroot}%{_cross_bindir}

%cross_scan_attribution --clarify %{S:1000} go-vendor vendor

%files
%license LICENSE
%{_cross_attribution_file}
%{_cross_attribution_vendor_dir}
%{_cross_bindir}/aws-iam-authenticator

%changelog
