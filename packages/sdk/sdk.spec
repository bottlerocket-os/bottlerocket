%global product thar
%global cross %{product}-
%global dist .%{product}1

# Disable automatic debug symbol extraction.
%global debug_package %{nil}
%global __debug_install_post /bin/true
%global __os_install_post %{nil}
%undefine _enable_debug_packages

# Prevent RPM from detecting provides or requires in any directories.
# These are not suitable dependencies for most other packages.
%global __requires_exclude_from ^/%{_prefix}/.*$
%global __provides_exclude_from ^/%{_prefix}/.*$

%global brver 2018.11.1
%global binver 2.31.1
%global gccver 8.2.0
%global gccmaj 8

%bcond_without x86_64 # with
%bcond_without aarch64 # with

Name:    %{?cross}sdk
Version: %{brver}
Release: 1%{?dist}
Summary: Thar SDK
Group:   Development/Tools
License: GPLv2+ and GPLv3+ and LGPLv3+ and GFDL and MIT
URL:     https://github.com/buildroot/buildroot
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
Source100: thar_sdk_aarch64_defconfig
Source101: thar_sdk_x86_64_defconfig
Patch1: 0001-disable-shared-for-host-builds-of-gmp-isl-mpc-mpfr.patch
Patch2: 0002-allow-unknown-vendor-name-for-toolchain.patch
Patch3: 0003-add-TOOLS_DIR-and-SYSROOT_DIR-to-control-output.patch
Patch4: 0004-build-binutils-with-TOOLS_DIR-and-SYSROOT_DIR.patch
Patch5: 0005-build-gcc-with-TOOLS_DIR-and-SYSROOT_DIR.patch
BuildRoot: %{_tmppath}/%{name}-%{version}-%{release}-root-%(%{__id_u} -n)
BuildRequires: bc
BuildRequires: perl-ExtUtils-MakeMaker
BuildRequires: python
BuildRequires: rsync
BuildRequires: wget

%description
The Thar SDK.

# Packages containing binaries meant to execute on the host system
# are kept as architecture-specific, since we will install and run
# them on systems of that type. Packages containing libraries for the
# target system are marked as "noarch", since although they can be
# installed, they are not native, and the resulting binaries must be
# executed elsewhere.

%if %{with aarch64}
%package -n binutils-aarch64-unknown-linux-gnu
Summary: Binary utilities for aarch64-unknown-linux-gnu
Version: %{binver}
License: GPLv3+

%description -n binutils-aarch64-unknown-linux-gnu
%{summary}.

%package -n gcc-aarch64-unknown-linux-gnu
Summary: GNU C compiler for aarch64-unknown-linux-gnu
Version: %{gccver}
Requires: binutils-aarch64-unknown-linux-gnu%{?_isa} = %{binver}
License: GPLv3+

%description -n gcc-aarch64-unknown-linux-gnu
%{summary}.

%package -n gcc-c++-aarch64-unknown-linux-gnu
Summary: GNU C++ compiler for aarch64-unknown-linux-gnu
Version: %{gccver}
Requires: gcc-aarch64-unknown-linux-gnu%{?_isa} = %{gccver}
Requires: libatomic-aarch64-unknown-linux-gnu = %{gccver}
Requires: libitm-aarch64-unknown-linux-gnu = %{gccver}
Requires: libstdc++-aarch64-unknown-linux-gnu = %{gccver}
License: GPLv3+

%description -n gcc-c++-aarch64-unknown-linux-gnu
%{summary}.

%package -n libstdc++-aarch64-unknown-linux-gnu
Summary: GNU Standard C++ library for aarch64-unknown-linux-gnu
Version: %{gccver}
BuildArch: noarch
Requires: gcc-aarch64-unknown-linux-gnu = %{gccver}
License: GPLv3+

%description -n libstdc++-aarch64-unknown-linux-gnu
%{summary}.

%package -n libatomic-aarch64-unknown-linux-gnu
Summary: GNU Atomic library for aarch64-unknown-linux-gnu
Version: %{gccver}
BuildArch: noarch
Requires: gcc-aarch64-unknown-linux-gnu = %{gccver}
License: GPLv3+

%description -n libatomic-aarch64-unknown-linux-gnu
%{summary}.

%package -n libitm-aarch64-unknown-linux-gnu
Summary: GNU Transactional Memory library for aarch64-unknown-linux-gnu
Version: %{gccver}
BuildArch: noarch
Requires: gcc-aarch64-unknown-linux-gnu = %{gccver}
License: GPLv3+

%description -n libitm-aarch64-unknown-linux-gnu
%{summary}.

%package -n libsanitizer-aarch64-unknown-linux-gnu
Summary: Sanitizer libraries for aarch64-unknown-linux-gnu
Version: %{gccver}
BuildArch: noarch
Requires: gcc-aarch64-unknown-linux-gnu = %{gccver}
License: GPLv3+

%description -n libsanitizer-aarch64-unknown-linux-gnu
%{summary}.
%endif

%if %{with x86_64}
%package -n binutils-x86_64-unknown-linux-gnu
Summary: Binary utilities for x86_64-unknown-linux-gnu
Version: %{binver}
License: GPLv3+

%description -n binutils-x86_64-unknown-linux-gnu
%{summary}.

%package -n gcc-x86_64-unknown-linux-gnu
Summary: GNU C compiler for x86_64-unknown-linux-gnu
Version: %{gccver}
Requires: binutils-x86_64-unknown-linux-gnu%{?_isa} = %{binver}
License: GPLv3+

%description -n gcc-x86_64-unknown-linux-gnu
%{summary}.

%package -n gcc-c++-x86_64-unknown-linux-gnu
Summary: GNU C++ compiler for x86_64-unknown-linux-gnu
Version: %{gccver}
Requires: gcc-x86_64-unknown-linux-gnu%{?_isa} = %{gccver}
Requires: libatomic-x86_64-unknown-linux-gnu = %{gccver}
Requires: libitm-x86_64-unknown-linux-gnu = %{gccver}
Requires: libquadmath-x86_64-unknown-linux-gnu = %{gccver}
Requires: libstdc++-x86_64-unknown-linux-gnu = %{gccver}
License: GPLv3+

%description -n gcc-c++-x86_64-unknown-linux-gnu
%{summary}.

%package -n libstdc++-x86_64-unknown-linux-gnu
Summary: GNU Standard C++ library for x86_64-unknown-linux-gnu
Version: %{gccver}
BuildArch: noarch
Requires: gcc-x86_64-unknown-linux-gnu = %{gccver}
License: GPLv3+

%description -n libstdc++-x86_64-unknown-linux-gnu
%{summary}.

%package -n libatomic-x86_64-unknown-linux-gnu
Summary: GNU Atomic library for x86_64-unknown-linux-gnu
Version: %{gccver}
BuildArch: noarch
Requires: gcc-x86_64-unknown-linux-gnu = %{gccver}
License: GPLv3+

%description -n libatomic-x86_64-unknown-linux-gnu
%{summary}.

%package -n libitm-x86_64-unknown-linux-gnu
Summary: GNU Transactional Memory library for x86_64-unknown-linux-gnu
Version: %{gccver}
BuildArch: noarch
Requires: gcc-x86_64-unknown-linux-gnu = %{gccver}
License: GPLv3+

%description -n libitm-x86_64-unknown-linux-gnu
%{summary}.

%package -n libquadmath-x86_64-unknown-linux-gnu
Summary: GNU Quad-Precision Math library for x86_64-unknown-linux-gnu
Version: %{gccver}
BuildArch: noarch
Requires: gcc-x86_64-unknown-linux-gnu = %{gccver}
License: GPLv3+

%description -n libquadmath-x86_64-unknown-linux-gnu
%{summary}.

%package -n libmpx-x86_64-unknown-linux-gnu
Summary: MPX libraries for x86_64-unknown-linux-gnu
Version: %{gccver}
BuildArch: noarch
Requires: gcc-x86_64-unknown-linux-gnu = %{gccver}
License: GPLv3+

%description -n libmpx-x86_64-unknown-linux-gnu
%{summary}.

%package -n libsanitizer-x86_64-unknown-linux-gnu
Summary: Sanitizer libraries for x86_64-unknown-linux-gnu
Version: %{gccver}
BuildArch: noarch
Requires: gcc-x86_64-unknown-linux-gnu = %{gccver}
License: GPLv3+

%description -n libsanitizer-x86_64-unknown-linux-gnu
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
cp -a %{SOURCE101} configs

%build
mkdir output
%if %{with aarch64}
output="output/aarch64"
config="configs/thar_sdk_aarch64_defconfig"
make O=${output} defconfig BR2_DEFCONFIG=${config}
make O=${output} toolchain
%endif

%if %{with x86_64}
output="output/x86_64"
config="configs/thar_sdk_x86_64_defconfig"
make O=${output} defconfig BR2_DEFCONFIG=${config}
make O=${output} toolchain
%endif

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}

%if %{with aarch64}
rsync -av output/aarch64/toolchain/ %{buildroot}
%endif

%if %{with x86_64}
rsync -av output/x86_64/toolchain/ %{buildroot}
%endif

%check

%clean
rm -rf %{buildroot}

%files

%if %{with aarch64}
%files -n binutils-aarch64-unknown-linux-gnu
%{_bindir}/aarch64-unknown-linux-gnu-addr2line
%{_bindir}/aarch64-unknown-linux-gnu-ar
%{_bindir}/aarch64-unknown-linux-gnu-as
%{_bindir}/aarch64-unknown-linux-gnu-c++filt
%{_bindir}/aarch64-unknown-linux-gnu-elfedit
%{_bindir}/aarch64-unknown-linux-gnu-gprof
%{_bindir}/aarch64-unknown-linux-gnu-ld
%{_bindir}/aarch64-unknown-linux-gnu-ld.bfd
%{_bindir}/aarch64-unknown-linux-gnu-nm
%{_bindir}/aarch64-unknown-linux-gnu-objcopy
%{_bindir}/aarch64-unknown-linux-gnu-objdump
%{_bindir}/aarch64-unknown-linux-gnu-ranlib
%{_bindir}/aarch64-unknown-linux-gnu-readelf
%{_bindir}/aarch64-unknown-linux-gnu-size
%{_bindir}/aarch64-unknown-linux-gnu-strings
%{_bindir}/aarch64-unknown-linux-gnu-strip
%dir %{_prefix}/aarch64-unknown-linux-gnu/bin
%{_prefix}/aarch64-unknown-linux-gnu/bin/ld
%{_prefix}/aarch64-unknown-linux-gnu/bin/ld.bfd
%{_prefix}/aarch64-unknown-linux-gnu/bin/strip
%{_prefix}/aarch64-unknown-linux-gnu/bin/objdump
%{_prefix}/aarch64-unknown-linux-gnu/bin/objcopy
%{_prefix}/aarch64-unknown-linux-gnu/bin/as
%{_prefix}/aarch64-unknown-linux-gnu/bin/ranlib
%{_prefix}/aarch64-unknown-linux-gnu/bin/readelf
%{_prefix}/aarch64-unknown-linux-gnu/bin/nm
%{_prefix}/aarch64-unknown-linux-gnu/bin/ar
%dir %{_prefix}/aarch64-unknown-linux-gnu/include
%dir %{_prefix}/aarch64-unknown-linux-gnu/lib
%dir %{_prefix}/aarch64-unknown-linux-gnu/sys-root
%dir %{_prefix}/aarch64-unknown-linux-gnu/sys-root/usr
%dir %{_prefix}/aarch64-unknown-linux-gnu/sys-root/usr/lib

%files -n gcc-aarch64-unknown-linux-gnu
%{_bindir}/aarch64-unknown-linux-gnu-cc
%{_bindir}/aarch64-unknown-linux-gnu-cpp
%{_bindir}/aarch64-unknown-linux-gnu-gcc
%{_bindir}/aarch64-unknown-linux-gnu-gcc-8
%{_bindir}/aarch64-unknown-linux-gnu-gcc-ar
%{_bindir}/aarch64-unknown-linux-gnu-gcc-nm
%{_bindir}/aarch64-unknown-linux-gnu-gcc-ranlib
%{_bindir}/aarch64-unknown-linux-gnu-gcov
%{_bindir}/aarch64-unknown-linux-gnu-gcov-dump
%{_bindir}/aarch64-unknown-linux-gnu-gcov-tool
%dir %{_prefix}/lib/gcc/aarch64-unknown-linux-gnu
%dir %{_prefix}/lib/gcc/aarch64-unknown-linux-gnu/%{gccmaj}
%{_prefix}/lib/gcc/aarch64-unknown-linux-gnu/%{gccmaj}/crtbegin.o
%{_prefix}/lib/gcc/aarch64-unknown-linux-gnu/%{gccmaj}/crtbeginS.o
%{_prefix}/lib/gcc/aarch64-unknown-linux-gnu/%{gccmaj}/crtbeginT.o
%{_prefix}/lib/gcc/aarch64-unknown-linux-gnu/%{gccmaj}/crtend.o
%{_prefix}/lib/gcc/aarch64-unknown-linux-gnu/%{gccmaj}/crtendS.o
%{_prefix}/lib/gcc/aarch64-unknown-linux-gnu/%{gccmaj}/crtfastmath.o
%{_prefix}/lib/gcc/aarch64-unknown-linux-gnu/%{gccmaj}/libgcov.a
%{_prefix}/lib/gcc/aarch64-unknown-linux-gnu/%{gccmaj}/libgcc.a
%{_prefix}/lib/gcc/aarch64-unknown-linux-gnu/%{gccmaj}/libgcc_eh.a
%dir %{_prefix}/lib/gcc/aarch64-unknown-linux-gnu/%{gccmaj}/include
%{_prefix}/lib/gcc/aarch64-unknown-linux-gnu/%{gccmaj}/include/*
%dir %{_prefix}/lib/gcc/aarch64-unknown-linux-gnu/%{gccmaj}/include-fixed
%{_prefix}/lib/gcc/aarch64-unknown-linux-gnu/%{gccmaj}/include-fixed/README
%exclude %{_prefix}/lib/gcc/aarch64-unknown-linux-gnu/%{gccmaj}/include-fixed/syslimits.h
%exclude %{_prefix}/lib/gcc/aarch64-unknown-linux-gnu/%{gccmaj}/include-fixed/limits.h
%dir %{_prefix}/lib/gcc/aarch64-unknown-linux-gnu/%{gccmaj}/install-tools
%{_prefix}/lib/gcc/aarch64-unknown-linux-gnu/%{gccmaj}/install-tools/fixinc_list
%{_prefix}/lib/gcc/aarch64-unknown-linux-gnu/%{gccmaj}/install-tools/gsyslimits.h
%{_prefix}/lib/gcc/aarch64-unknown-linux-gnu/%{gccmaj}/install-tools/macro_list
%{_prefix}/lib/gcc/aarch64-unknown-linux-gnu/%{gccmaj}/install-tools/mkheaders.conf
%dir %{_prefix}/lib/gcc/aarch64-unknown-linux-gnu/%{gccmaj}/install-tools/include
%{_prefix}/lib/gcc/aarch64-unknown-linux-gnu/%{gccmaj}/install-tools/include/README
%{_prefix}/lib/gcc/aarch64-unknown-linux-gnu/%{gccmaj}/install-tools/include/limits.h
%dir %{_prefix}/lib/gcc/aarch64-unknown-linux-gnu/%{gccmaj}/plugin
%{_prefix}/lib/gcc/aarch64-unknown-linux-gnu/%{gccmaj}/plugin/*
%dir %{_libexecdir}/gcc/aarch64-unknown-linux-gnu
%dir %{_libexecdir}/gcc/aarch64-unknown-linux-gnu/%{gccmaj}
%{_libexecdir}/gcc/aarch64-unknown-linux-gnu/%{gccmaj}/cc1
%{_libexecdir}/gcc/aarch64-unknown-linux-gnu/%{gccmaj}/cc1plus
%{_libexecdir}/gcc/aarch64-unknown-linux-gnu/%{gccmaj}/collect2
%{_libexecdir}/gcc/aarch64-unknown-linux-gnu/%{gccmaj}/liblto_plugin.so
%{_libexecdir}/gcc/aarch64-unknown-linux-gnu/%{gccmaj}/liblto_plugin.so.0
%{_libexecdir}/gcc/aarch64-unknown-linux-gnu/%{gccmaj}/liblto_plugin.so.0.0.0
%{_libexecdir}/gcc/aarch64-unknown-linux-gnu/%{gccmaj}/lto1
%{_libexecdir}/gcc/aarch64-unknown-linux-gnu/%{gccmaj}/lto-wrapper
%dir %{_libexecdir}/gcc/aarch64-unknown-linux-gnu/%{gccmaj}/install-tools
%{_libexecdir}/gcc/aarch64-unknown-linux-gnu/%{gccmaj}/install-tools/mkinstalldirs
%{_libexecdir}/gcc/aarch64-unknown-linux-gnu/%{gccmaj}/install-tools/fixincl
%{_libexecdir}/gcc/aarch64-unknown-linux-gnu/%{gccmaj}/install-tools/mkheaders
%{_libexecdir}/gcc/aarch64-unknown-linux-gnu/%{gccmaj}/install-tools/fixinc.sh
%dir %{_libexecdir}/gcc/aarch64-unknown-linux-gnu/%{gccmaj}/plugin
%{_libexecdir}/gcc/aarch64-unknown-linux-gnu/%{gccmaj}/plugin/gengtype
%{_prefix}/aarch64-unknown-linux-gnu/sys-root/usr/lib/libgcc_s.so
%{_prefix}/aarch64-unknown-linux-gnu/sys-root/usr/lib/libgcc_s.so.1

%files -n gcc-c++-aarch64-unknown-linux-gnu
%{_bindir}/aarch64-unknown-linux-gnu-c++
%{_bindir}/aarch64-unknown-linux-gnu-g++
%dir %{_prefix}/aarch64-unknown-linux-gnu/include/c++
%dir %{_prefix}/aarch64-unknown-linux-gnu/include/c++/%{gccmaj}
%{_prefix}/aarch64-unknown-linux-gnu/include/c++/%{gccmaj}/*

%files -n libstdc++-aarch64-unknown-linux-gnu
%{_prefix}/aarch64-unknown-linux-gnu/sys-root/usr/lib/libstdc++.a
%{_prefix}/aarch64-unknown-linux-gnu/sys-root/usr/lib/libstdc++fs.a
%{_prefix}/aarch64-unknown-linux-gnu/sys-root/usr/lib/libsupc++.a

%files -n libatomic-aarch64-unknown-linux-gnu
%{_prefix}/aarch64-unknown-linux-gnu/sys-root/usr/lib/libatomic.a

%files -n libitm-aarch64-unknown-linux-gnu
%{_prefix}/aarch64-unknown-linux-gnu/sys-root/usr/lib/libitm.a
%{_prefix}/aarch64-unknown-linux-gnu/sys-root/usr/lib/libitm.spec

%files -n libsanitizer-aarch64-unknown-linux-gnu
%{_prefix}/aarch64-unknown-linux-gnu/sys-root/usr/lib/libasan.a
%{_prefix}/aarch64-unknown-linux-gnu/sys-root/usr/lib/libasan_preinit.o
%{_prefix}/aarch64-unknown-linux-gnu/sys-root/usr/lib/liblsan.a
%{_prefix}/aarch64-unknown-linux-gnu/sys-root/usr/lib/liblsan_preinit.o
%{_prefix}/aarch64-unknown-linux-gnu/sys-root/usr/lib/libsanitizer.spec
%{_prefix}/aarch64-unknown-linux-gnu/sys-root/usr/lib/libtsan.a
%{_prefix}/aarch64-unknown-linux-gnu/sys-root/usr/lib/libtsan_preinit.o
%{_prefix}/aarch64-unknown-linux-gnu/sys-root/usr/lib/libubsan.a
%endif

%if %{with x86_64}
%files -n binutils-x86_64-unknown-linux-gnu
%{_bindir}/x86_64-unknown-linux-gnu-addr2line
%{_bindir}/x86_64-unknown-linux-gnu-ar
%{_bindir}/x86_64-unknown-linux-gnu-as
%{_bindir}/x86_64-unknown-linux-gnu-c++filt
%{_bindir}/x86_64-unknown-linux-gnu-elfedit
%{_bindir}/x86_64-unknown-linux-gnu-gprof
%{_bindir}/x86_64-unknown-linux-gnu-ld
%{_bindir}/x86_64-unknown-linux-gnu-ld.bfd
%{_bindir}/x86_64-unknown-linux-gnu-nm
%{_bindir}/x86_64-unknown-linux-gnu-objcopy
%{_bindir}/x86_64-unknown-linux-gnu-objdump
%{_bindir}/x86_64-unknown-linux-gnu-ranlib
%{_bindir}/x86_64-unknown-linux-gnu-readelf
%{_bindir}/x86_64-unknown-linux-gnu-size
%{_bindir}/x86_64-unknown-linux-gnu-strings
%{_bindir}/x86_64-unknown-linux-gnu-strip
%dir %{_prefix}/x86_64-unknown-linux-gnu/bin
%{_prefix}/x86_64-unknown-linux-gnu/bin/ld
%{_prefix}/x86_64-unknown-linux-gnu/bin/ld.bfd
%{_prefix}/x86_64-unknown-linux-gnu/bin/strip
%{_prefix}/x86_64-unknown-linux-gnu/bin/objdump
%{_prefix}/x86_64-unknown-linux-gnu/bin/objcopy
%{_prefix}/x86_64-unknown-linux-gnu/bin/as
%{_prefix}/x86_64-unknown-linux-gnu/bin/ranlib
%{_prefix}/x86_64-unknown-linux-gnu/bin/readelf
%{_prefix}/x86_64-unknown-linux-gnu/bin/nm
%{_prefix}/x86_64-unknown-linux-gnu/bin/ar
%dir %{_prefix}/x86_64-unknown-linux-gnu/include
%dir %{_prefix}/x86_64-unknown-linux-gnu/lib
%dir %{_prefix}/x86_64-unknown-linux-gnu/sys-root
%dir %{_prefix}/x86_64-unknown-linux-gnu/sys-root/usr
%dir %{_prefix}/x86_64-unknown-linux-gnu/sys-root/usr/lib

%files -n gcc-x86_64-unknown-linux-gnu
%{_bindir}/x86_64-unknown-linux-gnu-cc
%{_bindir}/x86_64-unknown-linux-gnu-cpp
%{_bindir}/x86_64-unknown-linux-gnu-gcc
%{_bindir}/x86_64-unknown-linux-gnu-gcc-8
%{_bindir}/x86_64-unknown-linux-gnu-gcc-ar
%{_bindir}/x86_64-unknown-linux-gnu-gcc-nm
%{_bindir}/x86_64-unknown-linux-gnu-gcc-ranlib
%{_bindir}/x86_64-unknown-linux-gnu-gcov
%{_bindir}/x86_64-unknown-linux-gnu-gcov-dump
%{_bindir}/x86_64-unknown-linux-gnu-gcov-tool
%dir %{_prefix}/lib/gcc/x86_64-unknown-linux-gnu
%dir %{_prefix}/lib/gcc/x86_64-unknown-linux-gnu/%{gccmaj}
%{_prefix}/lib/gcc/x86_64-unknown-linux-gnu/%{gccmaj}/crtbegin.o
%{_prefix}/lib/gcc/x86_64-unknown-linux-gnu/%{gccmaj}/crtbeginS.o
%{_prefix}/lib/gcc/x86_64-unknown-linux-gnu/%{gccmaj}/crtbeginT.o
%{_prefix}/lib/gcc/x86_64-unknown-linux-gnu/%{gccmaj}/crtend.o
%{_prefix}/lib/gcc/x86_64-unknown-linux-gnu/%{gccmaj}/crtendS.o
%{_prefix}/lib/gcc/x86_64-unknown-linux-gnu/%{gccmaj}/crtfastmath.o
%{_prefix}/lib/gcc/x86_64-unknown-linux-gnu/%{gccmaj}/crtprec32.o
%{_prefix}/lib/gcc/x86_64-unknown-linux-gnu/%{gccmaj}/crtprec64.o
%{_prefix}/lib/gcc/x86_64-unknown-linux-gnu/%{gccmaj}/crtprec80.o
%{_prefix}/lib/gcc/x86_64-unknown-linux-gnu/%{gccmaj}/libgcov.a
%{_prefix}/lib/gcc/x86_64-unknown-linux-gnu/%{gccmaj}/libgcc.a
%{_prefix}/lib/gcc/x86_64-unknown-linux-gnu/%{gccmaj}/libgcc_eh.a
%dir %{_prefix}/lib/gcc/x86_64-unknown-linux-gnu/%{gccmaj}/include
%{_prefix}/lib/gcc/x86_64-unknown-linux-gnu/%{gccmaj}/include/*
%dir %{_prefix}/lib/gcc/x86_64-unknown-linux-gnu/%{gccmaj}/include-fixed
%{_prefix}/lib/gcc/x86_64-unknown-linux-gnu/%{gccmaj}/include-fixed/README
%exclude %{_prefix}/lib/gcc/x86_64-unknown-linux-gnu/%{gccmaj}/include-fixed/syslimits.h
%exclude %{_prefix}/lib/gcc/x86_64-unknown-linux-gnu/%{gccmaj}/include-fixed/limits.h
%dir %{_prefix}/lib/gcc/x86_64-unknown-linux-gnu/%{gccmaj}/install-tools
%{_prefix}/lib/gcc/x86_64-unknown-linux-gnu/%{gccmaj}/install-tools/fixinc_list
%{_prefix}/lib/gcc/x86_64-unknown-linux-gnu/%{gccmaj}/install-tools/gsyslimits.h
%{_prefix}/lib/gcc/x86_64-unknown-linux-gnu/%{gccmaj}/install-tools/macro_list
%{_prefix}/lib/gcc/x86_64-unknown-linux-gnu/%{gccmaj}/install-tools/mkheaders.conf
%dir %{_prefix}/lib/gcc/x86_64-unknown-linux-gnu/%{gccmaj}/install-tools/include
%{_prefix}/lib/gcc/x86_64-unknown-linux-gnu/%{gccmaj}/install-tools/include/README
%{_prefix}/lib/gcc/x86_64-unknown-linux-gnu/%{gccmaj}/install-tools/include/limits.h
%dir %{_prefix}/lib/gcc/x86_64-unknown-linux-gnu/%{gccmaj}/plugin
%{_prefix}/lib/gcc/x86_64-unknown-linux-gnu/%{gccmaj}/plugin/*
%dir %{_libexecdir}/gcc/x86_64-unknown-linux-gnu
%dir %{_libexecdir}/gcc/x86_64-unknown-linux-gnu/%{gccmaj}
%{_libexecdir}/gcc/x86_64-unknown-linux-gnu/%{gccmaj}/cc1
%{_libexecdir}/gcc/x86_64-unknown-linux-gnu/%{gccmaj}/cc1plus
%{_libexecdir}/gcc/x86_64-unknown-linux-gnu/%{gccmaj}/collect2
%{_libexecdir}/gcc/x86_64-unknown-linux-gnu/%{gccmaj}/liblto_plugin.so
%{_libexecdir}/gcc/x86_64-unknown-linux-gnu/%{gccmaj}/liblto_plugin.so.0
%{_libexecdir}/gcc/x86_64-unknown-linux-gnu/%{gccmaj}/liblto_plugin.so.0.0.0
%{_libexecdir}/gcc/x86_64-unknown-linux-gnu/%{gccmaj}/lto1
%{_libexecdir}/gcc/x86_64-unknown-linux-gnu/%{gccmaj}/lto-wrapper
%dir %{_libexecdir}/gcc/x86_64-unknown-linux-gnu/%{gccmaj}/install-tools
%{_libexecdir}/gcc/x86_64-unknown-linux-gnu/%{gccmaj}/install-tools/mkinstalldirs
%{_libexecdir}/gcc/x86_64-unknown-linux-gnu/%{gccmaj}/install-tools/fixincl
%{_libexecdir}/gcc/x86_64-unknown-linux-gnu/%{gccmaj}/install-tools/mkheaders
%{_libexecdir}/gcc/x86_64-unknown-linux-gnu/%{gccmaj}/install-tools/fixinc.sh
%dir %{_libexecdir}/gcc/x86_64-unknown-linux-gnu/%{gccmaj}/plugin
%{_libexecdir}/gcc/x86_64-unknown-linux-gnu/%{gccmaj}/plugin/gengtype
%{_prefix}/x86_64-unknown-linux-gnu/sys-root/usr/lib/libgcc_s.so
%{_prefix}/x86_64-unknown-linux-gnu/sys-root/usr/lib/libgcc_s.so.1

%files -n gcc-c++-x86_64-unknown-linux-gnu
%{_bindir}/x86_64-unknown-linux-gnu-c++
%{_bindir}/x86_64-unknown-linux-gnu-g++
%dir %{_prefix}/x86_64-unknown-linux-gnu/include/c++
%dir %{_prefix}/x86_64-unknown-linux-gnu/include/c++/%{gccmaj}
%{_prefix}/x86_64-unknown-linux-gnu/include/c++/%{gccmaj}/*

%files -n libstdc++-x86_64-unknown-linux-gnu
%{_prefix}/x86_64-unknown-linux-gnu/sys-root/usr/lib/libstdc++.a
%{_prefix}/x86_64-unknown-linux-gnu/sys-root/usr/lib/libstdc++fs.a
%{_prefix}/x86_64-unknown-linux-gnu/sys-root/usr/lib/libsupc++.a

%files -n libatomic-x86_64-unknown-linux-gnu
%{_prefix}/x86_64-unknown-linux-gnu/sys-root/usr/lib/libatomic.a

%files -n libitm-x86_64-unknown-linux-gnu
%{_prefix}/x86_64-unknown-linux-gnu/sys-root/usr/lib/libitm.a
%{_prefix}/x86_64-unknown-linux-gnu/sys-root/usr/lib/libitm.spec

%files -n libquadmath-x86_64-unknown-linux-gnu
%{_prefix}/x86_64-unknown-linux-gnu/sys-root/usr/lib/libquadmath.a

%files -n libmpx-x86_64-unknown-linux-gnu
%{_prefix}/x86_64-unknown-linux-gnu/sys-root/usr/lib/libmpx.a
%{_prefix}/x86_64-unknown-linux-gnu/sys-root/usr/lib/libmpx.spec
%{_prefix}/x86_64-unknown-linux-gnu/sys-root/usr/lib/libmpxwrappers.a

%files -n libsanitizer-x86_64-unknown-linux-gnu
%{_prefix}/x86_64-unknown-linux-gnu/sys-root/usr/lib/libasan.a
%{_prefix}/x86_64-unknown-linux-gnu/sys-root/usr/lib/libasan_preinit.o
%{_prefix}/x86_64-unknown-linux-gnu/sys-root/usr/lib/liblsan.a
%{_prefix}/x86_64-unknown-linux-gnu/sys-root/usr/lib/liblsan_preinit.o
%{_prefix}/x86_64-unknown-linux-gnu/sys-root/usr/lib/libsanitizer.spec
%{_prefix}/x86_64-unknown-linux-gnu/sys-root/usr/lib/libtsan.a
%{_prefix}/x86_64-unknown-linux-gnu/sys-root/usr/lib/libtsan_preinit.o
%{_prefix}/x86_64-unknown-linux-gnu/sys-root/usr/lib/libubsan.a
%endif

%changelog
