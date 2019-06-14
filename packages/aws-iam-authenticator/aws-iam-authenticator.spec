%global goproject github.com/kubernetes-sigs
%global gorepo aws-iam-authenticator
%global goimport %{goproject}/%{gorepo}

%global gover 0.4.0
%global rpmver %{gover}

%global _dwz_low_mem_die_limit 0

Name: %{_cross_os}%{gorepo}
Version: %{rpmver}
Release: 1%{?dist}
Summary: AWS IAM authenticator
License: ASL 2.0
URL: https://%{goimport}
Source0: https://%{goimport}/archive/v%{gover}/%{gorepo}-%{gover}.tar.gz
BuildRequires: git
BuildRequires: gcc-%{_cross_target}
BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}golang
Requires: %{_cross_os}glibc

%description
%{summary}.

%prep
%autosetup -Sgit -n %{gorepo}-%{gover} -p1
mkdir -p GOPATH/src/%{goproject}
ln -s %{_builddir}/%{gorepo}-%{gover} GOPATH/src/%{goimport}

%build
cd GOPATH/src/%{goimport}
export CC="%{_cross_target}-gcc"
export GOPATH="${PWD}/GOPATH"
export GOARCH="%{_cross_go_arch}"
export PKG_CONFIG_PATH="%{_cross_pkgconfigdir}"
export BUILDTAGS="rpm_crashtraceback"
go build -buildmode pie -tags="${BUILDTAGS}" -o aws-iam-authenticator %{goimport}/cmd/aws-iam-authenticator

%install
install -d %{buildroot}%{_cross_bindir}
install -p -m 0755 aws-iam-authenticator %{buildroot}%{_cross_bindir}

%files
%{_cross_bindir}/aws-iam-authenticator

%changelog
