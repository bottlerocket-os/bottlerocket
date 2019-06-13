%global debug_package %{nil}

%global tiniver 0.18.0

Name: %{_cross_os}docker-init
Version: 18.09.6
Release: 1%{?dist}
Summary: Init for containers
License: MIT
URL: https://github.com/krallin/tini
Source0: https://github.com/krallin/tini/archive/%{tiniver}/tini-%{tiniver}.tar.gz
BuildRequires: cmake
BuildRequires: gcc-%{_cross_target}
BuildRequires: %{_cross_os}glibc-devel

%description
%{summary}.

%prep
%autosetup -n tini-%{tiniver} -p1

%build
%{cross_cmake} .
%make_build tini-static

%install
install -d %{buildroot}%{_cross_bindir}
install -p -m 0755 tini-static %{buildroot}%{_cross_bindir}/docker-init

%files
%{_cross_bindir}/docker-init

%changelog
