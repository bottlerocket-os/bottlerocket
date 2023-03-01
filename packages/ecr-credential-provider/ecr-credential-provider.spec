%global goproject github.com/kubernetes
%global gorepo cloud-provider-aws
%global goimport %{goproject}/%{gorepo}

%global gover 1.25.3
%global rpmver %{gover}

%global _dwz_low_mem_die_limit 0

%global gitrev 2ca3fc6e5e74e856411e25ae0f26d4c299e2eb3d
%global shortrev %(c=%{gitrev}; echo ${c:0:7})

Name: %{_cross_os}ecr-credential-provider
Version: %{rpmver}
Release: 1%{?dist}
Summary: Container image registry credential provider for AWS ECR
License: Apache-2.0
URL: https://github.com/kubernetes/cloud-provider-aws

Source: cloud-provider-aws-%{gover}.tar.gz
Source1: bundled-cloud-provider-aws-%{gover}.tar.gz
Source1000: clarify.toml

BuildRequires: %{_cross_os}glibc-devel

%description
%{summary}.

%prep
%setup -n %{gorepo}-%{gover} -q
%setup -T -D -n %{gorepo}-%{gover} -b 1 -q

%build
%set_cross_go_flags

go build -buildmode=pie -ldflags="${GOLDFLAGS}" -o=ecr-credential-provider cmd/ecr-credential-provider/*.go

%install
install -d %{buildroot}%{_cross_libexecdir}/kubernetes/kubelet/plugins
install -p -m 0755 ecr-credential-provider %{buildroot}%{_cross_libexecdir}/kubernetes/kubelet/plugins/ecr-credential-provider

%cross_scan_attribution --clarify %{S:1000} go-vendor vendor

%files
%license LICENSE
%{_cross_attribution_file}
%{_cross_attribution_vendor_dir}
%{_cross_libexecdir}/kubernetes/kubelet/plugins/ecr-credential-provider
