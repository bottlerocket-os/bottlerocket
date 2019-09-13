Name: %{_cross_os}libnfnetlink
Version: 1.0.1
Release: 1%{?dist}
Summary: Library for netfilter netlink
License: GPLv2+
URL: http://netfilter.org
Source0: http://netfilter.org/projects/libnfnetlink/files/libnfnetlink-%{version}.tar.bz2
BuildRequires: gcc-%{_cross_target}
BuildRequires: %{_cross_os}glibc-devel
Requires: %{_cross_os}glibc

%description
%{summary}.

%package devel
Summary: Files for development using the library for netfilter netlink
Requires: %{name}

%description devel
%{summary}.

%prep
%autosetup -n libnfnetlink-%{version} -p1

%build
%cross_configure \
  --enable-static

%make_build

%install
%make_install

%files
%{_cross_libdir}/*.so.*

%files devel
%{_cross_libdir}/*.a
%{_cross_libdir}/*.so
%dir %{_cross_includedir}/libnfnetlink
%{_cross_includedir}/libnfnetlink/*.h
%{_cross_pkgconfigdir}/*.pc
%exclude %{_cross_libdir}/*.la

%changelog
