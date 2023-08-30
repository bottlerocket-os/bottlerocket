%global nvidia_modprobe_version 495.44

Name: %{_cross_os}libnvidia-container
Version: 1.13.5
Release: 1%{?dist}
Summary: NVIDIA container runtime library
# The COPYING and COPYING.LESSER files in the sources don't apply to libnvidia-container
# they are there because they apply to libelf in elfutils
License: Apache-2.0
URL: https://github.com/NVIDIA/libnvidia-container
Source0: https://github.com/NVIDIA/libnvidia-container/archive/v%{version}/libnvidia-container-%{version}.tar.gz
Source1: https://github.com/NVIDIA/nvidia-modprobe/archive/%{nvidia_modprobe_version}/nvidia-modprobe-%{nvidia_modprobe_version}.tar.gz
Source2: libnvidia-container-sysctl.conf

# First party patches from 1 to 1000
Patch0001: 0001-use-shared-libtirpc.patch
Patch0002: 0002-use-prefix-from-environment.patch
Patch0003: 0003-keep-debug-symbols.patch
Patch0004: 0004-Use-NVIDIA_PATH-to-look-up-binaries.patch
Patch0005: 0005-makefile-avoid-ldconfig-when-cross-compiling.patch

BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}libelf-devel
BuildRequires: %{_cross_os}libcap-devel
BuildRequires: %{_cross_os}libseccomp-devel
BuildRequires: %{_cross_os}libtirpc-devel
Requires: %{_cross_os}libelf
Requires: %{_cross_os}libcap
Requires: %{_cross_os}libseccomp
Requires: %{_cross_os}libtirpc

%description
%{summary}.

%package devel
Summary: Files for development using the NVIDIA container runtime library
Requires: %{_cross_os}libnvidia-container

%description devel
%{summary}.

%prep
%autosetup -Sgit -n libnvidia-container-%{version} -p1
mkdir -p deps/src/nvidia-modprobe-%{nvidia_modprobe_version}
tar -C deps/src/nvidia-modprobe-%{nvidia_modprobe_version} --strip-components=1 \
  -xzf %{SOURCE1} nvidia-modprobe-%{nvidia_modprobe_version}/{modprobe-utils,COPYING}
patch -d deps/src/nvidia-modprobe-%{nvidia_modprobe_version} -p1 < mk/nvidia-modprobe.patch
touch deps/src/nvidia-modprobe-%{nvidia_modprobe_version}/.download_stamp

%global set_env \
%set_cross_build_flags \\\
%set_cross_go_flags \\\
export CC=%{_cross_target}-gcc \\\
export LD=%{_cross_target}-ld \\\
export CFLAGS="${CFLAGS} -I%{_cross_includedir}/tirpc" \\\
export WITH_LIBELF=yes \\\
export WITH_SECCOMP=yes \\\
export WITH_TIRPC=yes \\\
export WITH_NVCGO=yes \\\
export prefix=%{_cross_prefix} \\\
export DESTDIR=%{buildroot} \\\
%{nil}

%build
%set_env
%make_build

%install
%set_env
%make_install
install -d %{buildroot}%{_cross_sysctldir}
install -p -m 0644 %{S:2} %{buildroot}%{_cross_sysctldir}/90-libnvidia-container.conf

%files
%license NOTICE LICENSE
%{_cross_attribution_file}
%{_cross_libdir}/libnvidia-container.so
%{_cross_libdir}/libnvidia-container.so.*
%{_cross_libdir}/libnvidia-container-go.so
%{_cross_libdir}/libnvidia-container-go.so.*
%{_cross_bindir}/nvidia-container-cli
%{_cross_sysctldir}/90-libnvidia-container.conf
%exclude %{_cross_docdir}

%files devel
%{_cross_includedir}/*.h
%{_cross_libdir}/*.so
%{_cross_libdir}/*.a
%{_cross_libdir}/pkgconfig/*.pc
