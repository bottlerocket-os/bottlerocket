%global goproject github.com/opencontainers
%global gorepo runc
%global goimport %{goproject}/%{gorepo}
%global commit ff819c7e9184c13b7c2607fe6c30ae19403a7aff
%global shortcommit ff819c7

%global gover 1.0.0-rc92
%global rpmver 1.0.0~rc92

%global _dwz_low_mem_die_limit 0

Name: %{_cross_os}%{gorepo}
Version: %{rpmver}
Release: 2.%{shortcommit}%{?dist}
Summary: CLI for running Open Containers
License: Apache-2.0
URL: https://%{goimport}
Source0: https://%{goimport}/archive/%{commit}/%{gorepo}-%{commit}.tar.gz

# TODO: see if this can go upstream
Patch0001: 0001-do-not-label-dev-mqueue.patch

BuildRequires: git
BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}libseccomp-devel
Requires: %{_cross_os}libseccomp

%description
%{summary}.

%prep
%autosetup -Sgit -n %{gorepo}-%{commit} -p1
%cross_go_setup %{gorepo}-%{commit} %{goproject} %{goimport}

%build
%cross_go_configure %{goimport}
export BUILDTAGS="ambient seccomp selinux"
go build -buildmode pie -tags="${BUILDTAGS}" -o bin/runc .

%install
install -d %{buildroot}%{_cross_bindir}
install -p -m 0755 bin/runc %{buildroot}%{_cross_bindir}

%cross_scan_attribution go-vendor vendor

%files
%license LICENSE NOTICE
%{_cross_attribution_file}
%{_cross_attribution_vendor_dir}
%{_cross_bindir}/runc

%changelog
