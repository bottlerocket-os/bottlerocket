Name: %{_cross_os}glibc
Version: 2.31
Release: 1%{?dist}
Summary: The GNU libc libraries
License: LGPL-2.1-or-later AND (LGPL-2.1-or-later WITH GCC-exception-2.0) AND GPL-2.0-or-later AND (GPL-2.0-or-later WITH GCC-exception-2.0) AND BSD-3-Clause AND ISC
URL: http://www.gnu.org/software/glibc/
Source0: https://ftp.gnu.org/gnu/glibc/glibc-%{version}.tar.xz
Source1: glibc-tmpfiles.conf
Patch1: glibc-cs-path.patch
Patch2: glibc-c-utf8-locale.patch
Patch1001: 1001-move-ldconfig-cache-to-ephemeral-storage.patch

%description
%{summary}.

%package devel
Summary: Files for development using the GNU libc libraries.
Requires: %{name}

%description devel
%{summary}.

%prep
%autosetup -n glibc-%{version} -p1

%build
mkdir build
cd build
BUILDFLAGS="-O2 -g -Wp,-D_GLIBCXX_ASSERTIONS -fstack-clash-protection"
CFLAGS="${BUILDFLAGS}" CPPFLAGS="" CXXFLAGS="${BUILDFLAGS}" \
../configure \
  --prefix="%{_cross_prefix}" \
  --sysconfdir="%{_cross_sysconfdir}" \
  --localstatedir="%{_cross_localstatedir}" \
  --target="%{_cross_target}" \
  --host="%{_cross_target}" \
  --build="%{_build}" \
  --with-headers="%{_cross_includedir}" \
  --enable-bind-now \
  --enable-kernel="5.4.0" \
  --enable-shared \
  --enable-stack-protector=strong \
  --enable-static-pie \
  --disable-crypt \
  --disable-multi-arch \
  --disable-profile \
  --disable-systemtap \
  --disable-timezone-tools \
  --disable-tunables \
  --without-cvs \
  --without-gd \
  --without-selinux \

make %{?_smp_mflags} -O -r

%install
make -j1 install_root=%{buildroot} install -C build

mkdir -p %{buildroot}%{_cross_tmpfilesdir}
install -d %{buildroot}%{_cross_tmpfilesdir}
install -p -m 0644 %{S:1} %{buildroot}%{_cross_tmpfilesdir}/glibc.conf

truncate -s 0 %{buildroot}%{_cross_libdir}/gconv/gconv-modules
chmod 644 %{buildroot}%{_cross_libdir}/gconv/gconv-modules
truncate -s 0 %{buildroot}%{_cross_libdir}/gconv/gconv-modules.cache
chmod 644 %{buildroot}%{_cross_libdir}/gconv/gconv-modules.cache

truncate -s 0 %{buildroot}%{_cross_datadir}/locale/locale.alias
chmod 644 %{buildroot}%{_cross_datadir}/locale/locale.alias

%files
%license COPYING COPYING.LIB LICENSES
%{_cross_attribution_file}
%{_cross_tmpfilesdir}/glibc.conf
%exclude %{_cross_sysconfdir}/rpc

%{_cross_bindir}/getconf
%{_cross_bindir}/getent
%{_cross_bindir}/ldd
%exclude %{_cross_bindir}/catchsegv
%exclude %{_cross_bindir}/gencat
%exclude %{_cross_bindir}/iconv
%exclude %{_cross_bindir}/locale
%exclude %{_cross_bindir}/localedef
%exclude %{_cross_bindir}/makedb
%exclude %{_cross_bindir}/mtrace
%exclude %{_cross_bindir}/pldd
%exclude %{_cross_bindir}/pcprofiledump
%exclude %{_cross_bindir}/sotruss
%exclude %{_cross_bindir}/sprof
%exclude %{_cross_bindir}/xtrace

%{_cross_sbindir}/ldconfig
%exclude %{_cross_sbindir}/iconvconfig
%exclude %{_cross_sbindir}/nscd
%exclude %{_cross_sbindir}/sln

%dir %{_cross_libexecdir}/getconf
%{_cross_libexecdir}/getconf/*

%{_cross_libdir}/ld-*.so
%{_cross_libdir}/ld-linux-*.so.*
%{_cross_libdir}/libBrokenLocale.so.*
%{_cross_libdir}/libBrokenLocale-*.so
%{_cross_libdir}/libSegFault.so
%{_cross_libdir}/libanl.so.*
%{_cross_libdir}/libanl-*.so
%{_cross_libdir}/libc.so.*
%{_cross_libdir}/libc-*.so
%{_cross_libdir}/libdl.so.*
%{_cross_libdir}/libdl-*.so
%{_cross_libdir}/libm.so.*
%{_cross_libdir}/libm-*.so
%{_cross_libdir}/libnss_dns-*.so
%{_cross_libdir}/libnss_dns.so.*
%{_cross_libdir}/libnss_files-*.so
%{_cross_libdir}/libnss_files.so.*
%{_cross_libdir}/libpthread.so.*
%{_cross_libdir}/libpthread-*.so
%{_cross_libdir}/libresolv.so.*
%{_cross_libdir}/libresolv-*.so
%{_cross_libdir}/librt.so.*
%{_cross_libdir}/librt-*.so
%{_cross_libdir}/libthread_db.so.*
%{_cross_libdir}/libthread_db-*.so
%{_cross_libdir}/libutil.so.*
%{_cross_libdir}/libutil-*.so
%if "%{_cross_arch}" == "x86_64"
%{_cross_libdir}/libmvec.so.*
%{_cross_libdir}/libmvec-*.so
%endif
%exclude %{_cross_libdir}/audit/sotruss-lib.so
%exclude %{_cross_libdir}/libmemusage.so
%exclude %{_cross_libdir}/libpcprofile.so
%exclude %{_cross_libdir}/libnsl-*.so
%exclude %{_cross_libdir}/libnsl.so.*
%exclude %{_cross_libdir}/libnss_compat-*.so
%exclude %{_cross_libdir}/libnss_compat.so.*
%exclude %{_cross_libdir}/libnss_db-*.so
%exclude %{_cross_libdir}/libnss_db.so.*
%exclude %{_cross_libdir}/libnss_hesiod-*.so
%exclude %{_cross_libdir}/libnss_hesiod.so.*

%dir %{_cross_libdir}/gconv
%{_cross_libdir}/gconv/gconv-modules
%{_cross_libdir}/gconv/gconv-modules.cache
%exclude %{_cross_libdir}/gconv/*.so

%dir %{_cross_datadir}/i18n/charmaps
%dir %{_cross_datadir}/i18n/locales
%dir %{_cross_datadir}/locale
%{_cross_datadir}/locale/locale.alias
%exclude %{_cross_datadir}/i18n/charmaps/*
%exclude %{_cross_datadir}/i18n/locales/*
%exclude %{_cross_datadir}/locale/*
%exclude %{_cross_localstatedir}/db/Makefile

%files devel
%{_cross_libdir}/*.a
%{_cross_libdir}/*.o
%{_cross_libdir}/libBrokenLocale.so
%{_cross_libdir}/libanl.so
%{_cross_libdir}/libc.so
%{_cross_libdir}/libdl.so
%{_cross_libdir}/libm.so
%{_cross_libdir}/libnss_dns.so
%{_cross_libdir}/libnss_files.so
%{_cross_libdir}/libpthread.so
%{_cross_libdir}/libresolv.so
%{_cross_libdir}/librt.so
%{_cross_libdir}/libthread_db.so
%{_cross_libdir}/libutil.so
%if "%{_cross_arch}" == "x86_64"
%{_cross_libdir}/libmvec.so
%endif
%exclude %{_cross_libdir}/libnss_compat.so
%exclude %{_cross_libdir}/libnss_db.so
%exclude %{_cross_libdir}/libnss_hesiod.so

%dir %{_cross_includedir}/arpa
%dir %{_cross_includedir}/bits
%dir %{_cross_includedir}/gnu
%dir %{_cross_includedir}/net
%dir %{_cross_includedir}/netinet
%dir %{_cross_includedir}/netipx
%dir %{_cross_includedir}/netiucv
%dir %{_cross_includedir}/netpacket
%dir %{_cross_includedir}/netrose
%dir %{_cross_includedir}/nfs
%dir %{_cross_includedir}/protocols
%dir %{_cross_includedir}/rpc
%dir %{_cross_includedir}/scsi
%dir %{_cross_includedir}/sys
%dir %{_cross_includedir}/netash
%dir %{_cross_includedir}/netatalk
%dir %{_cross_includedir}/netax25
%dir %{_cross_includedir}/neteconet
%dir %{_cross_includedir}/netrom
%{_cross_includedir}/*.h
%{_cross_includedir}/*/*

%changelog
