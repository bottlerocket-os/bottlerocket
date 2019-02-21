# Disable debug symbol extraction and packaging.
%global debug_package %{nil}
%global __strip /bin/true
%global _build_id_links none

%global brver 2018.11.1
%global binver 2.31.1
%global gccver 8.2.0
%global gccmaj 8

Name: %{_cross_os}sdk
Version: %{brver}
Release: 1%{?dist}
Summary: Thar SDK
License: GPLv2+ and GPLv3+ and LGPLv3+ and GFDL and MIT
URL: https://github.com/buildroot/buildroot
Source0: https://github.com/buildroot/buildroot/archive/%{brver}/buildroot-%{brver}.tar.gz
Source1: https://ftp.gnu.org/gnu/binutils/binutils-%{binver}.tar.xz
Source2: https://ftp.gnu.org/gnu/bison/bison-3.0.4.tar.xz
Source3: https://ftp.gnu.org/gnu/gawk/gawk-4.2.1.tar.xz
Source4: https://ftp.gnu.org/gnu/gcc/gcc-%{gccver}/gcc-%{gccver}.tar.xz
Source5: http://sources.buildroot.org/glibc/glibc-glibc-2.28-50-gb8dd0f42780a3133c02f064a2c0c5c4e7ab61aaa.tar.gz
Source6: https://ftp.gnu.org/gnu/gmp/gmp-6.1.2.tar.xz
Source7: http://isl.gforge.inria.fr/isl-0.18.tar.xz
Source8: https://cdn.kernel.org/pub/linux/kernel/v4.x/linux-4.14.86.tar.xz
Source9: http://download.savannah.gnu.org/releases/lzip/lzip-1.20.tar.gz
Source10: https://ftp.gnu.org/gnu/m4/m4-1.4.18.tar.xz
Source11: https://ftp.gnu.org/gnu/mpc/mpc-1.0.3.tar.gz
Source12: https://ftp.gnu.org/gnu/mpfr/mpfr-3.1.6.tar.xz
Source13: https://ftp.gnu.org/gnu/tar/tar-1.29.cpio.gz
Source100: sdk-%{_cross_arch}-defconfig
Patch1: 0001-disable-shared-for-host-builds-of-gmp-isl-mpc-mpfr.patch
Patch2: 0002-allow-unknown-vendor-name-for-toolchain.patch
Patch3: 0003-add-TOOLS_DIR-and-SYSROOT_DIR-to-control-output.patch
Patch4: 0004-build-binutils-with-TOOLS_DIR-and-SYSROOT_DIR.patch
Patch5: 0005-build-gcc-with-TOOLS_DIR-and-SYSROOT_DIR.patch
BuildRequires: bc
BuildRequires: perl-ExtUtils-MakeMaker
BuildRequires: python
BuildRequires: rsync
BuildRequires: wget

%description
%{summary}.

# Packages containing binaries meant to execute on the host system
# are kept as architecture-specific, since we will install and run
# them on systems of that type. Packages containing libraries for the
# target system are marked as "noarch", since although they can be
# installed, they are not native, and the resulting binaries must be
# executed elsewhere.

%package -n binutils-%{_cross_target}
Summary: Binary utilities for %{_cross_target}
Version: %{binver}
License: GPLv3+

%description -n binutils-%{_cross_target}
%{summary}.

%package -n gcc-%{_cross_target}
Summary: GNU C compiler for %{_cross_target}
Version: %{gccver}
Requires: binutils-%{_cross_target}%{?_isa} = %{binver}
License: GPLv3+

%description -n gcc-%{_cross_target}
%{summary}.

%package -n gcc-c++-%{_cross_target}
Summary: GNU C++ compiler for %{_cross_target}
Version: %{gccver}
Requires: gcc-%{_cross_target}%{?_isa} = %{gccver}
Requires: libatomic-%{_cross_target} = %{gccver}
Requires: libitm-%{_cross_target} = %{gccver}
%if "%{_cross_arch}" == "x86_64"
Requires: libquadmath-%{_cross_target} = %{gccver}
Requires: libmpx-%{_cross_target} = %{gccver}
%endif
Requires: libstdc++-%{_cross_target} = %{gccver}
License: GPLv3+

%description -n gcc-c++-%{_cross_target}
%{summary}.

%package -n libstdc++-%{_cross_target}
Summary: GNU Standard C++ library for %{_cross_target}
Version: %{gccver}
BuildArch: noarch
Requires: gcc-%{_cross_target} = %{gccver}
License: GPLv3+

%description -n libstdc++-%{_cross_target}
%{summary}.

%package -n libatomic-%{_cross_target}
Summary: GNU Atomic library for %{_cross_target}
Version: %{gccver}
BuildArch: noarch
Requires: gcc-%{_cross_target} = %{gccver}
License: GPLv3+

%description -n libatomic-%{_cross_target}
%{summary}.

%package -n libitm-%{_cross_target}
Summary: GNU Transactional Memory library for %{_cross_target}
Version: %{gccver}
BuildArch: noarch
Requires: gcc-%{_cross_target} = %{gccver}
License: GPLv3+

%description -n libitm-%{_cross_target}
%{summary}.

%package -n libsanitizer-%{_cross_target}
Summary: Sanitizer libraries for %{_cross_target}
Version: %{gccver}
BuildArch: noarch
Requires: gcc-%{_cross_target} = %{gccver}
License: GPLv3+

%description -n libsanitizer-%{_cross_target}
%{summary}.

%if "%{_cross_arch}" == "x86_64"
%package -n libquadmath-%{_cross_target}
Summary: GNU Quad-Precision Math library for %{_cross_target}
Version: %{gccver}
BuildArch: noarch
Requires: gcc-%{_cross_target} = %{gccver}
License: GPLv3+

%description -n libquadmath-%{_cross_target}
%{summary}.

%package -n libmpx-%{_cross_target}
Summary: MPX libraries for %{_cross_target}
Version: %{gccver}
BuildArch: noarch
Requires: gcc-%{_cross_target} = %{gccver}
License: GPLv3+

%description -n libmpx-%{_cross_target}
%{summary}.
%endif

%prep
%setup -n buildroot-%{brver}

# apply patches
%patch1 -p1
%patch2 -p1
%patch3 -p1
%patch4 -p1
%patch5 -p1

# move sources into place
mkdir -p dl/{binutils,bison,gawk,gcc,glibc,gmp,isl,linux,lzip,m4,mpc,mpfr,tar}
cp -a %{SOURCE1} dl/binutils
cp -a %{SOURCE2} dl/bison
cp -a %{SOURCE3} dl/gawk
cp -a %{SOURCE4} dl/gcc
cp -a %{SOURCE5} dl/glibc
cp -a %{SOURCE6} dl/gmp
cp -a %{SOURCE7} dl/isl
cp -a %{SOURCE8} dl/linux
cp -a %{SOURCE9} dl/lzip
cp -a %{SOURCE10} dl/m4
cp -a %{SOURCE11} dl/mpc
cp -a %{SOURCE12} dl/mpfr
cp -a %{SOURCE13} dl/tar

# move configurations into place
test -d configs || exit 1
cp -a %{SOURCE100} configs

%build
mkdir output
output="output/%{_cross_arch}"
config="configs/sdk-%{_cross_arch}-defconfig"
make O=${output} defconfig BR2_DEFCONFIG=${config}
make O=${output} toolchain

%install
rsync -av output/%{_cross_arch}/toolchain/ %{buildroot}

%files

%files -n binutils-%{_cross_target}
%{_bindir}/%{_cross_target}-addr2line
%{_bindir}/%{_cross_target}-ar
%{_bindir}/%{_cross_target}-as
%{_bindir}/%{_cross_target}-c++filt
%{_bindir}/%{_cross_target}-elfedit
%{_bindir}/%{_cross_target}-gprof
%{_bindir}/%{_cross_target}-ld
%{_bindir}/%{_cross_target}-ld.bfd
%{_bindir}/%{_cross_target}-nm
%{_bindir}/%{_cross_target}-objcopy
%{_bindir}/%{_cross_target}-objdump
%{_bindir}/%{_cross_target}-ranlib
%{_bindir}/%{_cross_target}-readelf
%{_bindir}/%{_cross_target}-size
%{_bindir}/%{_cross_target}-strings
%{_bindir}/%{_cross_target}-strip
%dir %{_prefix}/%{_cross_target}/bin
%{_prefix}/%{_cross_target}/bin/ld
%{_prefix}/%{_cross_target}/bin/ld.bfd
%{_prefix}/%{_cross_target}/bin/strip
%{_prefix}/%{_cross_target}/bin/objdump
%{_prefix}/%{_cross_target}/bin/objcopy
%{_prefix}/%{_cross_target}/bin/as
%{_prefix}/%{_cross_target}/bin/ranlib
%{_prefix}/%{_cross_target}/bin/readelf
%{_prefix}/%{_cross_target}/bin/nm
%{_prefix}/%{_cross_target}/bin/ar
%dir %{_prefix}/%{_cross_target}/include
%dir %{_prefix}/%{_cross_target}/lib
%dir %{_prefix}/%{_cross_target}/sys-root
%dir %{_cross_prefix}
%dir %{_cross_prefix}/lib

%files -n gcc-%{_cross_target}
%{_bindir}/%{_cross_target}-cc
%{_bindir}/%{_cross_target}-cpp
%{_bindir}/%{_cross_target}-gcc
%{_bindir}/%{_cross_target}-gcc-8
%{_bindir}/%{_cross_target}-gcc-ar
%{_bindir}/%{_cross_target}-gcc-nm
%{_bindir}/%{_cross_target}-gcc-ranlib
%{_bindir}/%{_cross_target}-gcov
%{_bindir}/%{_cross_target}-gcov-dump
%{_bindir}/%{_cross_target}-gcov-tool
%dir %{_prefix}/lib/gcc/%{_cross_target}
%dir %{_prefix}/lib/gcc/%{_cross_target}/%{gccmaj}
%{_prefix}/lib/gcc/%{_cross_target}/%{gccmaj}/crtbegin.o
%{_prefix}/lib/gcc/%{_cross_target}/%{gccmaj}/crtbeginS.o
%{_prefix}/lib/gcc/%{_cross_target}/%{gccmaj}/crtbeginT.o
%{_prefix}/lib/gcc/%{_cross_target}/%{gccmaj}/crtend.o
%{_prefix}/lib/gcc/%{_cross_target}/%{gccmaj}/crtendS.o
%{_prefix}/lib/gcc/%{_cross_target}/%{gccmaj}/crtfastmath.o
%if "%{_cross_arch}" == "x86_64"
%{_prefix}/lib/gcc/%{_cross_target}/%{gccmaj}/crtprec32.o
%{_prefix}/lib/gcc/%{_cross_target}/%{gccmaj}/crtprec64.o
%{_prefix}/lib/gcc/%{_cross_target}/%{gccmaj}/crtprec80.o
%endif
%{_prefix}/lib/gcc/%{_cross_target}/%{gccmaj}/libgcov.a
%{_prefix}/lib/gcc/%{_cross_target}/%{gccmaj}/libgcc.a
%{_prefix}/lib/gcc/%{_cross_target}/%{gccmaj}/libgcc_eh.a
%dir %{_prefix}/lib/gcc/%{_cross_target}/%{gccmaj}/include
%{_prefix}/lib/gcc/%{_cross_target}/%{gccmaj}/include/*
%dir %{_prefix}/lib/gcc/%{_cross_target}/%{gccmaj}/include-fixed
%{_prefix}/lib/gcc/%{_cross_target}/%{gccmaj}/include-fixed/README
%exclude %{_prefix}/lib/gcc/%{_cross_target}/%{gccmaj}/include-fixed/syslimits.h
%exclude %{_prefix}/lib/gcc/%{_cross_target}/%{gccmaj}/include-fixed/limits.h
%dir %{_prefix}/lib/gcc/%{_cross_target}/%{gccmaj}/install-tools
%{_prefix}/lib/gcc/%{_cross_target}/%{gccmaj}/install-tools/fixinc_list
%{_prefix}/lib/gcc/%{_cross_target}/%{gccmaj}/install-tools/gsyslimits.h
%{_prefix}/lib/gcc/%{_cross_target}/%{gccmaj}/install-tools/macro_list
%{_prefix}/lib/gcc/%{_cross_target}/%{gccmaj}/install-tools/mkheaders.conf
%dir %{_prefix}/lib/gcc/%{_cross_target}/%{gccmaj}/install-tools/include
%{_prefix}/lib/gcc/%{_cross_target}/%{gccmaj}/install-tools/include/README
%{_prefix}/lib/gcc/%{_cross_target}/%{gccmaj}/install-tools/include/limits.h
%dir %{_prefix}/lib/gcc/%{_cross_target}/%{gccmaj}/plugin
%{_prefix}/lib/gcc/%{_cross_target}/%{gccmaj}/plugin/*
%dir %{_libexecdir}/gcc/%{_cross_target}
%dir %{_libexecdir}/gcc/%{_cross_target}/%{gccmaj}
%{_libexecdir}/gcc/%{_cross_target}/%{gccmaj}/cc1
%{_libexecdir}/gcc/%{_cross_target}/%{gccmaj}/cc1plus
%{_libexecdir}/gcc/%{_cross_target}/%{gccmaj}/collect2
%{_libexecdir}/gcc/%{_cross_target}/%{gccmaj}/liblto_plugin.so
%{_libexecdir}/gcc/%{_cross_target}/%{gccmaj}/liblto_plugin.so.0
%{_libexecdir}/gcc/%{_cross_target}/%{gccmaj}/liblto_plugin.so.0.0.0
%{_libexecdir}/gcc/%{_cross_target}/%{gccmaj}/lto1
%{_libexecdir}/gcc/%{_cross_target}/%{gccmaj}/lto-wrapper
%dir %{_libexecdir}/gcc/%{_cross_target}/%{gccmaj}/install-tools
%{_libexecdir}/gcc/%{_cross_target}/%{gccmaj}/install-tools/mkinstalldirs
%{_libexecdir}/gcc/%{_cross_target}/%{gccmaj}/install-tools/fixincl
%{_libexecdir}/gcc/%{_cross_target}/%{gccmaj}/install-tools/mkheaders
%{_libexecdir}/gcc/%{_cross_target}/%{gccmaj}/install-tools/fixinc.sh
%dir %{_libexecdir}/gcc/%{_cross_target}/%{gccmaj}/plugin
%{_libexecdir}/gcc/%{_cross_target}/%{gccmaj}/plugin/gengtype
%{_cross_prefix}/lib/libgcc_s.so
%{_cross_prefix}/lib/libgcc_s.so.1

%files -n gcc-c++-%{_cross_target}
%{_bindir}/%{_cross_target}-c++
%{_bindir}/%{_cross_target}-g++
%dir %{_prefix}/%{_cross_target}/include/c++
%dir %{_prefix}/%{_cross_target}/include/c++/%{gccmaj}
%{_prefix}/%{_cross_target}/include/c++/%{gccmaj}/*

%files -n libstdc++-%{_cross_target}
%{_cross_prefix}/lib/libstdc++.a
%{_cross_prefix}/lib/libstdc++fs.a
%{_cross_prefix}/lib/libsupc++.a

%files -n libatomic-%{_cross_target}
%{_cross_prefix}/lib/libatomic.a

%files -n libitm-%{_cross_target}
%{_cross_prefix}/lib/libitm.a
%{_cross_prefix}/lib/libitm.spec

%files -n libsanitizer-%{_cross_target}
%{_cross_prefix}/lib/libasan.a
%{_cross_prefix}/lib/libasan_preinit.o
%{_cross_prefix}/lib/liblsan.a
%{_cross_prefix}/lib/liblsan_preinit.o
%{_cross_prefix}/lib/libsanitizer.spec
%{_cross_prefix}/lib/libtsan.a
%{_cross_prefix}/lib/libtsan_preinit.o
%{_cross_prefix}/lib/libubsan.a

%if "%{_cross_arch}" == "x86_64"
%files -n libquadmath-%{_cross_target}
%{_cross_prefix}/lib/libquadmath.a

%files -n libmpx-%{_cross_target}
%{_cross_prefix}/lib/libmpx.a
%{_cross_prefix}/lib/libmpx.spec
%{_cross_prefix}/lib/libmpxwrappers.a
%endif

%changelog
