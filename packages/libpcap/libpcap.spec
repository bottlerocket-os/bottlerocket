Name: %{_cross_os}libpcap
Version: 1.9.0
Release: 1%{?dist}
Summary: Library for packet capture
License: BSD with advertising
URL: http://www.tcpdump.org
Source0: http://www.tcpdump.org/release/libpcap-%{version}.tar.gz
BuildRequires: gcc-%{_cross_target}
BuildRequires: %{_cross_os}glibc-devel
Requires: %{_cross_os}glibc

%description
%{summary}.

%package devel
Summary: Files for development using the library for packet capture
Requires: %{name}

%description devel
%{summary}.

%prep
%autosetup -n libpcap-%{version} -p1

%build
%cross_configure \
  --enable-static

%make_build

%install
%make_install

%files
%{_cross_libdir}/*.so.*
%exclude %{_cross_bindir}/*
%exclude %{_cross_mandir}/*

%files devel
%{_cross_libdir}/*.a
%{_cross_libdir}/*.so
%{_cross_includedir}/*.h
%dir %{_cross_includedir}/pcap
%{_cross_includedir}/pcap/*.h
%{_cross_pkgconfigdir}/*.pc

%changelog
