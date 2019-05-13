%global goproject github.com/opencontainers
%global gorepo runc
%global goimport %{goproject}/%{gorepo}

%global gover 1.0.0-rc8
%global rpmver 1.0.0~rc8

%global _dwz_low_mem_die_limit 0

Name: %{_cross_os}%{gorepo}
Version: %{rpmver}
Release: 1%{?dist}
Summary: CLI for running Open Containers
License: ASL 2.0
URL: https://%{goimport}
Source0: https://%{goimport}/archive/v%{gover}/%{gorepo}-%{gover}.tar.gz
BuildRequires: git
BuildRequires: gcc-%{_cross_target}
BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}libseccomp-devel
BuildRequires: %{_cross_os}golang
Requires: %{_cross_os}glibc
Requires: %{_cross_os}libseccomp

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
export LDFLAGS="-X main.version=%{gover}"
export PKG_CONFIG_PATH="%{_cross_pkgconfigdir}"
export BUILDTAGS="rpm_crashtraceback ambient seccomp selinux"
go build -buildmode pie -tags="${BUILDTAGS}" -o bin/runc .

%install
install -d %{buildroot}%{_cross_bindir}
install -p -m 0755 bin/runc %{buildroot}%{_cross_bindir}

%files
%{_cross_bindir}/runc

%changelog
