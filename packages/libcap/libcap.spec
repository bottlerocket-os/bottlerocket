Name: %{_cross_os}libcap
Version: 2.50
Release: 1%{?dist}
Summary: Library for getting and setting POSIX.1e capabilities
License: GPL-2.0-only OR BSD-3-Clause
URL: https://sites.google.com/site/fullycapable/
Source0: https://git.kernel.org/pub/scm/libs/libcap/libcap.git/snapshot/libcap-%{version}.tar.gz
BuildRequires: libcap-devel
BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}libattr-devel
Requires: %{_cross_os}libattr

# Local changes.
Patch9001: 9001-dont-test-during-install.patch

%description
%{summary}.

%package devel
Summary: Files for development using the library for getting and setting POSIX.1e capabilities
Requires: %{name}

%description devel
%{summary}.

%prep
%autosetup -n libcap-%{version} -p1

%build
make \
  CC="%{_cross_target}-gcc" CFLAGS="%{_cross_cflags}" \
  BUILD_CC="gcc" BUILD_CFLAGS="%{optflags}" \
  prefix=%{_cross_prefix} lib=%{_cross_lib} \
  LIBDIR=%{_cross_libdir} SBINDIR=%{_cross_sbindir} \
  INCDIR=%{_cross_includedir} MANDIR=%{_cross_mandir} \
  PKGCONFIGDIR=%{_cross_pkgconfigdir} \
  GOLANG=no RAISE_SETFCAP=no PAM_CAP=no \

%install
make install \
  DESTDIR=%{buildroot} \
  CC="%{_cross_target}-gcc" CFLAGS="%{_cross_cflags}" \
  BUILD_CC="gcc" BUILD_CFLAGS="%{optflags}" \
  prefix=%{_cross_prefix} lib=%{_cross_lib} \
  LIBDIR=%{_cross_libdir} SBINDIR=%{_cross_sbindir} \
  INCDIR=%{_cross_includedir} MANDIR=%{_cross_mandir} \
  PKGCONFIGDIR=%{_cross_pkgconfigdir} \
  GOLANG=no RAISE_SETFCAP=no PAM_CAP=no \

chmod +x %{buildroot}%{_cross_libdir}/*.so.*

%files
%license License
%{_cross_attribution_file}
%{_cross_libdir}/*.so.*
%exclude %{_cross_mandir}
%exclude %{_cross_sbindir}

%files devel
%{_cross_libdir}/*.a
%{_cross_libdir}/*.so
%{_cross_includedir}/sys/*.h
%{_cross_pkgconfigdir}/*.pc

%changelog
