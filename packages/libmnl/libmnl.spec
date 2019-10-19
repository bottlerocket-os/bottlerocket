Name: %{_cross_os}libmnl
Version: 1.0.4
Release: 1%{?dist}
Summary: Library for netlink
License: LGPLv2+
URL: http://netfilter.org/projects/libmnl
Source0: http://netfilter.org/projects/libmnl/files/libmnl-%{version}.tar.bz2
BuildRequires: %{_cross_os}glibc-devel
Requires: %{_cross_os}glibc

%description
%{summary}.

%package devel
Summary: Files for development using the library for netlink
Requires: %{name}

%description devel
%{summary}.

%prep
%autosetup -n libmnl-%{version} -p1

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
%dir %{_cross_includedir}/libmnl
%{_cross_includedir}/libmnl/*.h
%{_cross_pkgconfigdir}/*.pc
%exclude %{_cross_libdir}/*.la

%changelog
