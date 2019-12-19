Name: %{_cross_os}libnftnl
Version: 1.1.5
Release: 1%{?dist}
Summary: Library for nftables netlink
License: GPLv2+
URL: http://netfilter.org/projects/libnftnl/
Source0: http://netfilter.org/projects/libnftnl/files/libnftnl-%{version}.tar.bz2
BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}libmnl-devel
Requires: %{_cross_os}libmnl

%description
%{summary}.

%package devel
Summary: Files for development using the library for nftables netlink
Requires: %{name}

%description devel
%{summary}.

%prep
%autosetup -n libnftnl-%{version} -p1

%build
%cross_configure \
  --disable-silent-rules \
  --enable-static \
  --without-json-parsing \

%make_build

%install
%make_install

%files
%{_cross_libdir}/*.so.*

%files devel
%{_cross_libdir}/*.a
%{_cross_libdir}/*.so
%dir %{_cross_includedir}/libnftnl
%{_cross_includedir}/libnftnl/*.h
%{_cross_pkgconfigdir}/*.pc
%exclude %{_cross_libdir}/*.la

%changelog
