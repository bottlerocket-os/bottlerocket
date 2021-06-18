Name: %{_cross_os}libtirpc
Version: 1.3.2
Release: 1%{?dist}
Summary: Library for RPC
License: BSD-3-Clause
URL: http://git.linux-nfs.org/?p=steved/libtirpc.git;a=summary
Source0: http://downloads.sourceforge.net/libtirpc/libtirpc-%{version}.tar.bz2
BuildRequires: %{_cross_os}glibc-devel

%description
%{summary}.

%package devel
Summary: Files for development using the library for RPC
Requires: %{name}

%description devel
%{summary}.

%prep
%autosetup -n libtirpc-%{version} -p1

%build
%cross_configure \
  --enable-static \
  --disable-authdes \
  --disable-gssapi \

%make_build

%install
%make_install

%files
%license COPYING
%{_cross_attribution_file}
%{_cross_libdir}/*.so.*
%exclude %{_cross_mandir}
%exclude %{_cross_sysconfdir}

%files devel
%{_cross_libdir}/*.a
%{_cross_libdir}/*.so
%dir %{_cross_includedir}/tirpc
%{_cross_includedir}/tirpc/*
%{_cross_pkgconfigdir}/*.pc
%exclude %{_cross_libdir}/*.la

%changelog
