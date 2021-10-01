Name: %{_cross_os}glibc
Version: 2.34
Release: 1%{?dist}
Summary: The GNU libc libraries
License: LGPL-2.1-or-later AND (LGPL-2.1-or-later WITH GCC-exception-2.0) AND GPL-2.0-or-later AND (GPL-2.0-or-later WITH GCC-exception-2.0) AND BSD-3-Clause AND ISC
URL: http://www.gnu.org/software/glibc/
Source0: https://ftp.gnu.org/gnu/glibc/glibc-%{version}.tar.xz
Source1: glibc-tmpfiles.conf

# Upstream patches from 2.34 release branch:
# ```
# git checkout origin/release/2.34/master
# git format-patch glibc-2.34..
# ```
Patch0001: 0001-ldconfig-avoid-leak-on-empty-paths-in-config-file.patch
Patch0002: 0002-gconv_parseconfdir-Fix-memory-leak.patch
Patch0003: 0003-gaiconf_init-Avoid-double-free-in-label-and-preceden.patch
Patch0004: 0004-copy_and_spawn_sgid-Avoid-double-calls-to-close.patch
Patch0005: 0005-iconv_charmap-Close-output-file-when-done.patch
Patch0006: 0006-Linux-Fix-fcntl-ioctl-prctl-redirects-for-_TIME_BITS.patch
Patch0007: 0007-librt-fix-NULL-pointer-dereference-bug-28213.patch
Patch0008: 0008-librt-add-test-bug-28213.patch
Patch0009: 0009-elf-Fix-missing-colon-in-LD_SHOW_AUXV-output-BZ-2825.patch
Patch0010: 0010-x86-64-Use-testl-to-check-__x86_string_control.patch
Patch0011: 0011-MIPS-Setup-errno-for-f-l-xstat.patch
Patch0012: 0012-support-Add-support_wait_for_thread_exit.patch
Patch0013: 0013-nptl-pthread_kill-pthread_cancel-should-not-fail-aft.patch
Patch0014: 0014-nptl-Fix-race-between-pthread_kill-and-thread-exit-b.patch
Patch0015: 0015-iconvconfig-Fix-behaviour-with-prefix-BZ-28199.patch
Patch0016: 0016-Fix-failing-nss-tst-nss-files-hosts-long-with-local-.patch
Patch0017: 0017-Use-Linux-5.14-in-build-many-glibcs.py.patch
Patch0018: 0018-Update-syscall-lists-for-Linux-5.14.patch
Patch0019: 0019-Update-kernel-version-to-5.14-in-tst-mman-consts.py.patch
Patch0020: 0020-Add-MADV_POPULATE_READ-and-MADV_POPULATE_WRITE-from-.patch
Patch0021: 0021-posix-Fix-attribute-access-mode-on-getcwd-BZ-27476.patch
Patch0022: 0022-nptl-pthread_kill-needs-to-return-ESRCH-for-old-prog.patch
Patch0023: 0023-nptl-Fix-type-of-pthread_mutexattr_getrobust_np-pthr.patch
Patch0024: 0024-support-Add-support_open_dev_null_range.patch
Patch0025: 0025-Use-support_open_dev_null_range-io-tst-closefrom-mis.patch
Patch0026: 0026-nptl-Avoid-setxid-deadlock-with-blocked-signals-in-t.patch

# Fedora patches
Patch1001: glibc-c-utf8-locale-1.patch
Patch1002: glibc-c-utf8-locale-2.patch
Patch1003: glibc-cs-path.patch

# Local patches
Patch9001: 9001-move-ldconfig-cache-to-ephemeral-storage.patch

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

%{_cross_libdir}/ld-linux-*.so.*
%{_cross_libdir}/libBrokenLocale.so.*
%{_cross_libdir}/libSegFault.so
%{_cross_libdir}/libanl.so.*
%{_cross_libdir}/libc.so.*
%{_cross_libdir}/libdl.so.*
%{_cross_libdir}/libm.so.*
%{_cross_libdir}/libnss_dns.so.*
%{_cross_libdir}/libnss_files.so.*
%{_cross_libdir}/libpthread.so.*
%{_cross_libdir}/libresolv.so.*
%{_cross_libdir}/librt.so.*
%{_cross_libdir}/libthread_db.so.*
%{_cross_libdir}/libutil.so.*
%if "%{_cross_arch}" == "x86_64"
%{_cross_libdir}/libmvec.so.*
%endif
%exclude %{_cross_libdir}/audit/sotruss-lib.so
%exclude %{_cross_libdir}/libc_malloc_debug.so.*
%exclude %{_cross_libdir}/libmemusage.so
%exclude %{_cross_libdir}/libpcprofile.so
%exclude %{_cross_libdir}/libnsl.so.*
%exclude %{_cross_libdir}/libnss_compat.so.*
%exclude %{_cross_libdir}/libnss_db.so.*
%exclude %{_cross_libdir}/libnss_hesiod.so.*

%dir %{_cross_libdir}/gconv
%dir %{_cross_libdir}/gconv/gconv-modules.d
%{_cross_libdir}/gconv/gconv-modules
%{_cross_libdir}/gconv/gconv-modules.cache
%exclude %{_cross_libdir}/gconv/*.so
%exclude %{_cross_libdir}/gconv/gconv-modules.d/*.conf

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
%{_cross_libdir}/libm.so
%{_cross_libdir}/libresolv.so
%{_cross_libdir}/libthread_db.so
%if "%{_cross_arch}" == "x86_64"
%{_cross_libdir}/libmvec.so
%endif
%exclude %{_cross_libdir}/libc_malloc_debug.so
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
