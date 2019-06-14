%global goproject github.com/containernetworking
%global gorepo cni
%global goimport %{goproject}/%{gorepo}

%global gover 0.7.1
%global rpmver %{gover}

%global _dwz_low_mem_die_limit 0

Name: %{_cross_os}%{gorepo}
Version: %{rpmver}
Release: 1%{?dist}
Summary: Plugins for container networking
License: ASL 2.0
URL: https://%{goimport}
Source0: https://%{goimport}/archive/v%{gover}/%{gorepo}-%{gover}.tar.gz
BuildRequires: git
BuildRequires: gcc-%{_cross_target}
BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}golang
Requires: %{_cross_os}glibc
Requires: %{_cross_os}iptables

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
go build -buildmode pie -tags="${BUILDTAGS}" -o "bin/cnitool" %{goimport}/cnitool

%install
install -d %{buildroot}%{_cross_libexecdir}/cni/bin
install -p -m 0755 bin/cnitool %{buildroot}%{_cross_libexecdir}/cni/bin

%files
%dir %{_cross_libexecdir}/cni/bin
%{_cross_libexecdir}/cni/bin/cnitool

%changelog
