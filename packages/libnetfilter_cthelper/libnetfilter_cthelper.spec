Name: %{_cross_os}libnetfilter_cthelper
Version: 1.0.0
Release: 1%{?dist}
Summary: Library for netfilter cthelper
License: GPLv2
URL: http://netfilter.org
Source0: https://netfilter.org/projects/libnetfilter_cthelper/files/libnetfilter_cthelper-%{version}.tar.bz2
BuildRequires: gcc-%{_cross_target}
BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}libmnl-devel
Requires: %{_cross_os}glibc
Requires: %{_cross_os}libmnl

%description
%{summary}.

%package devel
Summary: Files for development using the library for netfilter cthelper
Requires: %{name}

%description devel
%{summary}.

%prep
%autosetup -n libnetfilter_cthelper-%{version} -p1

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
%{_cross_libdir}/pkgconfig/*.pc
%dir %{_cross_includedir}/libnetfilter_cthelper
%{_cross_includedir}/libnetfilter_cthelper/*.h
%exclude %{_cross_libdir}/*.la

%changelog
