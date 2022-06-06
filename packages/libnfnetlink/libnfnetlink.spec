Name: %{_cross_os}libnfnetlink
Version: 1.0.2
Release: 1%{?dist}
Summary: Library for netfilter netlink
License: GPL-2.0-only
URL: http://netfilter.org
Source0: http://netfilter.org/projects/libnfnetlink/files/libnfnetlink-%{version}.tar.bz2
BuildRequires: %{_cross_os}glibc-devel

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
%license COPYING
%{_cross_attribution_file}
%{_cross_libdir}/*.so.*

%files devel
%{_cross_libdir}/*.a
%{_cross_libdir}/*.so
%dir %{_cross_includedir}/libnfnetlink
%{_cross_includedir}/libnfnetlink/*.h
%{_cross_pkgconfigdir}/*.pc
%exclude %{_cross_libdir}/*.la

%changelog
