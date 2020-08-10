Name: %{_cross_os}libnetfilter_queue
Version: 1.0.5
Release: 1%{?dist}
Summary: Library for netfilter queue
License: GPL-2.0-or-later
URL: http://netfilter.org
Source0: https://netfilter.org/projects/libnetfilter_queue/files/libnetfilter_queue-%{version}.tar.bz2
BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}libmnl-devel
BuildRequires: %{_cross_os}libnfnetlink-devel
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

%files
%license COPYING
%{_cross_attribution_file}
%{_cross_libdir}/*.so.*

%files devel
%{_cross_libdir}/*.a
%{_cross_libdir}/*.so
%dir %{_cross_includedir}/libnetfilter_queue
%{_cross_includedir}/libnetfilter_queue/*.h
%{_cross_pkgconfigdir}/*.pc
%exclude %{_cross_libdir}/*.la

%changelog
