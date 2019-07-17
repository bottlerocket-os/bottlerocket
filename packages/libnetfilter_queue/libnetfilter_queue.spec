Name: %{_cross_os}libnetfilter_queue
Version: 1.0.2
Release: 1%{?dist}
Summary: Library for netfilter queue
License: GPLv2
URL: http://netfilter.org
Source0: https://netfilter.org/projects/libnetfilter_queue/files/libnetfilter_queue-%{version}.tar.bz2
BuildRequires: gcc-%{_cross_target}
BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}libmnl-devel
BuildRequires: %{_cross_os}libnfnetlink-devel
Requires: %{_cross_os}glibc
Requires: %{_cross_os}libmnl
Requires: %{_cross_os}libnfnetlink

%description
%{summary}.

%package devel
Summary: Files for development using the library for netfilter queue
Requires: %{name}

%description devel
%{summary}.

%prep
%autosetup -n libnetfilter_queue-%{version} -p1

%build
%cross_configure \
  --enable-static

%make_build

%install
%make_install
rm %{buildroot}%{_cross_includedir}/internal.h

%files
%{_cross_libdir}/*.so.*

%files devel
%{_cross_libdir}/*.a
%{_cross_libdir}/*.so
%{_cross_libdir}/pkgconfig/*.pc
%dir %{_cross_includedir}/libnetfilter_queue
%{_cross_includedir}/libnetfilter_queue/*.h
%exclude %{_cross_libdir}/*.la

%changelog
