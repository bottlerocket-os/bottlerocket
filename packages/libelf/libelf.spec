Name: %{_cross_os}libelf
Version: 0.189
Release: 1%{?dist}
Summary: Library for ELF files
License: GPL-2.0-or-later OR LGPL-3.0-or-later
URL: https://sourceware.org/elfutils/
Source0: https://sourceware.org/elfutils/ftp/%{version}/elfutils-%{version}.tar.bz2

BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}libz-devel

%description
%{summary}.

%package devel
Summary: Files for development using the library for ELF files
Requires: %{name}
Requires: %{_cross_os}libz-devel

%description devel
%{summary}.

%prep
%autosetup -n elfutils-%{version} -p1

%build
%cross_configure \
  --without-bzlib \
  --without-lzma \
  --disable-silent-rules \
  --disable-symbol-versioning \
  --disable-nls \
  --disable-progs \
  --disable-debuginfod \
  --disable-libdebuginfod \

%make_build

%install
%make_install

%files
%license COPYING-GPLV2 COPYING-LGPLV3
%{_cross_libdir}/*.so.*
%{_cross_libdir}/libasm-*.so
%{_cross_libdir}/libelf-*.so
%{_cross_libdir}/libdw-*.so
%{_cross_attribution_file}
%exclude %{_cross_mandir}
%exclude %{_cross_bindir}

%files devel
%{_cross_includedir}/*.h
%{_cross_includedir}/elfutils/*.h
%{_cross_libdir}/libasm.so
%{_cross_libdir}/libelf.so
%{_cross_libdir}/libdw.so
%{_cross_libdir}/*.a
%{_cross_libdir}/pkgconfig/*.pc
