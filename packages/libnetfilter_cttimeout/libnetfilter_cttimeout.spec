Name: %{_cross_os}libnetfilter_cttimeout
Version: 1.0.0
Release: 1%{?dist}
Summary: Library for netfilter cttimeout
License: GPLv2+
URL: http://netfilter.org
Source0: https://netfilter.org/projects/libnetfilter_cttimeout/files/libnetfilter_cttimeout-%{version}.tar.bz2
BuildRequires: gcc-%{_cross_target}
BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}libmnl-devel
Requires: %{_cross_os}glibc
Requires: %{_cross_os}libmnl

%description
%{summary}.

%package devel
Summary: Files for development using the library for netfilter cttimeout
Requires: %{name}

%description devel
%{summary}.

%prep
%autosetup -n libnetfilter_cttimeout-%{version} -p1

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
%dir %{_cross_includedir}/libnetfilter_cttimeout
%{_cross_includedir}/libnetfilter_cttimeout/*.h
%{_cross_pkgconfigdir}/*.pc
%exclude %{_cross_libdir}/*.la

%changelog
