Name: %{_cross_os}glibc
Version: 2.36
Release: 1%{?dist}
Summary: The GNU libc libraries
License: LGPL-2.1-or-later AND (LGPL-2.1-or-later WITH GCC-exception-2.0) AND GPL-2.0-or-later AND (GPL-2.0-or-later WITH GCC-exception-2.0) AND BSD-3-Clause AND ISC
URL: http://www.gnu.org/software/glibc/
Source0: https://ftp.gnu.org/gnu/glibc/glibc-%{version}.tar.xz
Source1: glibc-tmpfiles.conf
Source2: ld.so.conf
Source3: ldconfig-service.conf

# We include this patch as a source file to have more control over how it's
# applied and reverted during the build.
Source99: HACK-only-build-and-install-localedef.patch

# Upstream patches from 2.36 release branch:
# ```
# git checkout origin/release/2.36/master
# git format-patch glibc-2.36..
# ```
Patch0001: 0001-stdlib-Suppress-gcc-diagnostic-that-char8_t-is-a-key.patch
Patch0002: 0002-wcsmbs-Add-missing-test-c8rtomb-test-mbrtoc8-depende.patch
Patch0003: 0003-dlfcn-Pass-caller-pointer-to-static-dlopen-implement.patch
Patch0004: 0004-Update-syscall-lists-for-Linux-5.19.patch
Patch0005: 0005-elf-Replace-strcpy-call-with-memcpy-BZ-29454.patch
Patch0006: 0006-Linux-Terminate-subprocess-on-late-failure-in-tst-pi.patch
Patch0007: 0007-alpha-Fix-generic-brk-system-call-emulation-in-__brk.patch
Patch0008: 0008-socket-Check-lengths-before-advancing-pointer-in-CMS.patch
Patch0009: 0009-NEWS-Add-entry-for-bug-28846.patch
Patch0010: 0010-glibcextract.py-Add-compile_c_snippet.patch
Patch0011: 0011-linux-Use-compile_c_snippet-to-check-linux-pidfd.h-a.patch
Patch0012: 0012-linux-Mimic-kernel-defition-for-BLOCK_SIZE.patch
Patch0013: 0013-linux-Use-compile_c_snippet-to-check-linux-mount.h-a.patch
Patch0014: 0014-linux-Fix-sys-mount.h-usage-with-kernel-headers.patch
Patch0015: 0015-Linux-Fix-enum-fsconfig_command-detection-in-sys-mou.patch
Patch0016: 0016-syslog-Fix-large-messages-BZ-29536.patch
Patch0017: 0017-elf-Call-__libc_early_init-for-reused-namespaces-bug.patch
Patch0018: 0018-Apply-asm-redirections-in-wchar.h-before-first-use.patch
Patch0019: 0019-elf-Restore-how-vDSO-dependency-is-printed-with-LD_T.patch
Patch0020: 0020-syslog-Remove-extra-whitespace-between-timestamp-and.patch
Patch0021: 0021-Add-NEWS-entry-for-CVE-2022-39046.patch
Patch0022: 0022-nscd-Fix-netlink-cache-invalidation-if-epoll-is-used.patch
Patch0023: 0023-resolv-Add-tst-resolv-byaddr-for-testing-reverse-loo.patch
Patch0024: 0024-resolv-Add-tst-resolv-aliases.patch
Patch0025: 0025-resolv-Add-internal-__res_binary_hnok-function.patch
Patch0026: 0026-resolv-Add-the-__ns_samebinaryname-function.patch
Patch0027: 0027-resolv-Add-internal-__ns_name_length_uncompressed-fu.patch
Patch0028: 0028-resolv-Add-DNS-packet-parsing-helpers-geared-towards.patch
Patch0029: 0029-nss_dns-Split-getanswer_ptr-from-getanswer_r.patch
Patch0030: 0030-nss_dns-Rewrite-_nss_dns_gethostbyaddr2_r-and-getans.patch
Patch0031: 0031-nss_dns-Remove-remnants-of-IPv6-address-mapping.patch
Patch0032: 0032-nss_dns-Rewrite-getanswer_r-to-match-getanswer_ptr-b.patch
Patch0033: 0033-nss_dns-In-gaih_getanswer_slice-skip-strange-aliases.patch
Patch0034: 0034-resolv-Add-new-tst-resolv-invalid-cname.patch
Patch0035: 0035-nss_dns-Rewrite-_nss_dns_gethostbyname4_r-using-curr.patch
Patch0036: 0036-resolv-Fix-building-tst-resolv-invalid-cname-for-ear.patch
Patch0037: 0037-NEWS-Note-bug-12154-and-bug-29305-as-fixed.patch
Patch0038: 0038-elf-Run-tst-audit-tlsdesc-tst-audit-tlsdesc-dlopen-e.patch
Patch0039: 0039-elf-Fix-hwcaps-string-size-overestimation.patch

# Fedora patches
Patch1001: glibc-cs-path.patch

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
%autosetup -Sgit -n glibc-%{version} -p1

%global glibc_configure %{shrink: \
BUILDFLAGS="-O2 -g -Wp,-D_GLIBCXX_ASSERTIONS -fstack-clash-protection" \
CFLAGS="${BUILDFLAGS}" CPPFLAGS="" CXXFLAGS="${BUILDFLAGS}" \
../configure \
  --prefix="%{_cross_prefix}" \
  --sysconfdir="%{_cross_sysconfdir}" \
  --localstatedir="%{_cross_localstatedir}" \
  --enable-bind-now \
  --enable-shared \
  --enable-stack-protector=strong \
  --disable-crypt \
  --disable-multi-arch \
  --disable-profile \
  --disable-systemtap \
  --disable-timezone-tools \
  --disable-tunables \
  --without-cvs \
  --without-gd \
  --without-selinux
  %{nil}}

%build

# First build the host tools we need, namely `localedef`. Apply a patch from
# Buildroot that allows us to build just this program and not everything.
patch -p1 < %{S:99}

mkdir build
pushd build
%glibc_configure
make %{?_smp_mflags} -O -r locale/others
mv locale/localedef %{_builddir}/localedef
popd

# Remove the previous build, revert the patch, and verify that the tree is
# clean, since we don't want to contaminate our target build.
rm -rf build
patch -p1 -R < %{S:99}
git diff --quiet

# Now build for the target. This is what will end up in the package, except
# for the C.UTF-8 locale, which we need `localedef` to generate.
mkdir build
pushd build
%glibc_configure \
  --target="%{_cross_target}" \
  --host="%{_cross_target}" \
  --build="%{_build}" \
  --with-headers="%{_cross_includedir}" \
  --enable-kernel="5.4.0"
make %{?_smp_mflags} -O -r
popd

%install
pushd build
make -j1 install_root=%{buildroot} install
# By default, LOCALEDEF refers to the target binary, and is invoked by the
# dynamic linker that was just built for the target. Neither will run on a
# build host with a different architecture. The locale format is compatible
# across architectures but not across glibc versions, so we can't rely on
# the binary in the SDK and must use the one we built earlier.
make -j1 install_root=%{buildroot} install-files-C.UTF-8/UTF-8 -C ../localedata objdir="$(pwd)" \
  LOCALEDEF="I18NPATH=. GCONV_PATH=$(pwd)/../iconvdata LC_ALL=C %{_builddir}/localedef"
popd

install -d %{buildroot}%{_cross_tmpfilesdir}
install -d %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}
install -d %{buildroot}%{_cross_unitdir}/ldconfig.service.d

install -p -m 0644 %{S:1} %{buildroot}%{_cross_tmpfilesdir}/glibc.conf
install -p -m 0644 %{S:2} %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}/ld.so.conf
install -p -m 0644 %{S:3} %{buildroot}%{_cross_unitdir}/ldconfig.service.d/ldconfig.conf

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
%exclude %{_cross_bindir}/gencat
%exclude %{_cross_bindir}/iconv
%exclude %{_cross_bindir}/ld.so
%exclude %{_cross_bindir}/ldd
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

%dir %{_cross_libdir}/locale
%dir %{_cross_libdir}/locale/C.utf8
%{_cross_libdir}/locale/C.utf8/LC_*

%dir %{_cross_datadir}/i18n
%dir %{_cross_datadir}/i18n/charmaps
%dir %{_cross_datadir}/i18n/locales
%dir %{_cross_datadir}/locale
%{_cross_datadir}/locale/locale.alias
%exclude %{_cross_datadir}/i18n/charmaps/*
%exclude %{_cross_datadir}/i18n/locales/*
%exclude %{_cross_datadir}/locale/*
%exclude %{_cross_localstatedir}/db/Makefile

%dir %{_cross_factorydir}
%{_cross_factorydir}%{_cross_sysconfdir}/ld.so.conf

%dir %{_cross_unitdir}/ldconfig.service.d
%{_cross_libdir}/systemd/system/ldconfig.service.d/ldconfig.conf

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
