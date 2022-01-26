%global nvidia_tesla_470_version 470.82.01
%global spdx_id %(bottlerocket-license-tool -l %{_builddir}/Licenses.toml spdx-id nvidia)
%global license_file %(bottlerocket-license-tool -l %{_builddir}/Licenses.toml path nvidia -p ./licenses)

Name: %{_cross_os}kmod-5.10-nvidia
Version: 1.0.0
Release: 1%{?dist}
Summary: NVIDIA drivers for the 5.10 kernel
# We use these licences because we only ship our own software in the main package,
# each subpackage includes the LICENSE file provided by the Licenses.toml file
License: Apache-2.0 OR MIT
URL: http://www.nvidia.com/

# NVIDIA .run scripts from 0 to 199
Source0: https://us.download.nvidia.com/tesla/%{nvidia_tesla_470_version}/NVIDIA-Linux-x86_64-%{nvidia_tesla_470_version}.run
Source1: https://us.download.nvidia.com/tesla/%{nvidia_tesla_470_version}/NVIDIA-Linux-aarch64-%{nvidia_tesla_470_version}.run

# Common NVIDIA conf files from 200 to 299
Source200: nvidia-tmpfiles.conf.in
Source202: nvidia-dependencies-modules-load.conf

# NVIDIA tesla conf files from 300 to 399
Source300: nvidia-tesla-tmpfiles.conf.in
Source301: nvidia-tesla-build-config.toml.in
Source302: nvidia-tesla-path.env.in
Source303: nvidia-ld.so.conf.in

BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}kernel-5.10-archive

%description
%{summary}.

%package tesla-470
Summary: NVIDIA 470 Tesla driver
Version: %{nvidia_tesla_470_version}
License: %{spdx_id}
Requires: %{name}

%description tesla-470
%{summary}

%prep
# Extract nvidia sources with `-x`, otherwise the script will try to install
# the driver in the current run
sh %{_sourcedir}/NVIDIA-Linux-%{_cross_arch}-%{nvidia_tesla_470_version}.run -x

%global kernel_sources %{_builddir}/kernel-devel
tar -xf %{_cross_datadir}/bottlerocket/kernel-devel.tar.xz

%build
pushd NVIDIA-Linux-%{_cross_arch}-%{nvidia_tesla_470_version}/kernel

# This recipe was based in the NVIDIA yum/dnf specs:
# https://github.com/NVIDIA/yum-packaging-precompiled-kmod

# We set IGNORE_CC_MISMATCH even though we are using the same compiler used to compile the kernel, if
# we don't set this flag the compilation fails
make %{?_smp_mflags} ARCH=%{_cross_karch} IGNORE_CC_MISMATCH=1 SYSSRC=%{kernel_sources} CC=%{_cross_target}-gcc LD=%{_cross_target}-ld

%{_cross_target}-strip -g --strip-unneeded nvidia/nv-interface.o
%{_cross_target}-strip -g --strip-unneeded nvidia-uvm.o
%{_cross_target}-strip -g --strip-unneeded nvidia-drm.o
%{_cross_target}-strip -g --strip-unneeded nvidia-peermem/nvidia-peermem.o
%{_cross_target}-strip -g --strip-unneeded nvidia-modeset/nv-modeset-interface.o

# We delete these files since we just stripped the input .o files above, and
# will be build at runtime in the host
rm nvidia{,-modeset,-peermem}.o

# Delete the .ko files created in make command, just to be safe that we
# don't include any linked module in the base image
rm nvidia{,-modeset,-peermem,-drm}.ko

popd

%install
install -d %{buildroot}%{_cross_libexecdir}
install -d %{buildroot}%{_cross_libdir}
install -d %{buildroot}%{_cross_tmpfilesdir}
install -d %{buildroot}%{_cross_unitdir}
install -d %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}/{drivers,ld.so.conf.d}

KERNEL_VERSION=$(cat %{kernel_sources}/include/config/kernel.release)
sed -e "s|__KERNEL_VERSION__|${KERNEL_VERSION}|" %{S:200} > nvidia.conf
install -p -m 0644 nvidia.conf %{buildroot}%{_cross_tmpfilesdir}

# Install modules-load.d drop-in to autoload required kernel modules
install -d %{buildroot}%{_cross_libdir}/modules-load.d
install -p -m 0644 %{S:202} %{buildroot}%{_cross_libdir}/modules-load.d/nvidia-dependencies.conf

# Begin NVIDIA tesla 470
pushd NVIDIA-Linux-%{_cross_arch}-%{nvidia_tesla_470_version}
# We install bins and libs in a versioned directory to prevent collisions with future drivers versions
install -d %{buildroot}%{_cross_libexecdir}/nvidia/tesla/bin/%{nvidia_tesla_470_version}
install -d %{buildroot}%{_cross_libdir}/nvidia/tesla/%{nvidia_tesla_470_version}/
install -d %{buildroot}%{_cross_datadir}/nvidia/tesla/%{nvidia_tesla_470_version}/module-objects.d
install -d %{buildroot}%{_cross_factorydir}/nvidia/tesla/%{nvidia_tesla_470_version}

sed -e 's|__NVIDIA_VERSION__|%{nvidia_tesla_470_version}|' %{S:300} > nvidia-tesla-%{nvidia_tesla_470_version}.conf
install -m 0644 nvidia-tesla-%{nvidia_tesla_470_version}.conf %{buildroot}%{_cross_tmpfilesdir}/
sed -e 's|__NVIDIA_MODULES__|%{_cross_datadir}/nvidia/tesla/%{nvidia_tesla_470_version}/module-objects.d/|' %{S:301} > \
  nvidia-tesla-%{nvidia_tesla_470_version}.toml
install -m 0644 nvidia-tesla-%{nvidia_tesla_470_version}.toml %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}/drivers
# Install nvidia-path environment file, will be used as a drop-in for containerd.service since
# libnvidia-container locates and mounts helper binaries into the containers from either
# `PATH` or `NVIDIA_PATH`
sed -e 's|__NVIDIA_BINDIR__|%{_cross_libexecdir}/nvidia/tesla/bin/%{nvidia_tesla_470_version}|' %{S:302} > nvidia-path.env
install -m 0644 nvidia-path.env %{buildroot}%{_cross_factorydir}/nvidia/tesla/%{nvidia_tesla_470_version}
# We need to add `_cross_libdir/nvidia_tesla_470_version` to the paths loaded by the ldconfig service
# because libnvidia-container uses the `ldcache` file created by the service, to locate and mount the
# libraries into the containers
sed -e 's|__LIBDIR__|%{_cross_libdir}|' %{S:303} | sed -e 's|__NVIDIA_VERSION__|%{nvidia_tesla_470_version}|' \
  > nvidia-tesla-%{nvidia_tesla_470_version}.conf
install -m 0644 nvidia-tesla-%{nvidia_tesla_470_version}.conf %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}/ld.so.conf.d/

# driver
install kernel/nvidia.mod.o %{buildroot}%{_cross_datadir}/nvidia/tesla/%{nvidia_tesla_470_version}/module-objects.d
install kernel/nvidia/nv-interface.o %{buildroot}%{_cross_datadir}/nvidia/tesla/%{nvidia_tesla_470_version}/module-objects.d
install kernel/nvidia/nv-kernel.o_binary %{buildroot}%{_cross_datadir}/nvidia/tesla/%{nvidia_tesla_470_version}/module-objects.d/nv-kernel.o

# uvm
install kernel/nvidia-uvm.mod.o %{buildroot}%{_cross_datadir}/nvidia/tesla/%{nvidia_tesla_470_version}/module-objects.d
install kernel/nvidia-uvm.o %{buildroot}%{_cross_datadir}/nvidia/tesla/%{nvidia_tesla_470_version}/module-objects.d

# modeset
install kernel/nvidia-modeset.mod.o %{buildroot}%{_cross_datadir}/nvidia/tesla/%{nvidia_tesla_470_version}/module-objects.d
install kernel/nvidia-modeset/nv-modeset-interface.o %{buildroot}%{_cross_datadir}/nvidia/tesla/%{nvidia_tesla_470_version}/module-objects.d
install kernel/nvidia-modeset/nv-modeset-kernel.o %{buildroot}%{_cross_datadir}/nvidia/tesla/%{nvidia_tesla_470_version}/module-objects.d

# peermem
install kernel/nvidia-peermem.mod.o %{buildroot}%{_cross_datadir}/nvidia/tesla/%{nvidia_tesla_470_version}/module-objects.d
install kernel/nvidia-peermem/nvidia-peermem.o %{buildroot}%{_cross_datadir}/nvidia/tesla/%{nvidia_tesla_470_version}/module-objects.d

# drm
install kernel/nvidia-drm.mod.o %{buildroot}/%{_cross_datadir}/nvidia/tesla/%{nvidia_tesla_470_version}/module-objects.d
install kernel/nvidia-drm.o %{buildroot}/%{_cross_datadir}/nvidia/tesla/%{nvidia_tesla_470_version}/module-objects.d

install -m 755 nvidia-smi %{buildroot}%{_cross_libexecdir}/nvidia/tesla/bin/%{nvidia_tesla_470_version}
install -m 755 nvidia-debugdump %{buildroot}%{_cross_libexecdir}/nvidia/tesla/bin/%{nvidia_tesla_470_version}
install -m 755 nvidia-cuda-mps-control %{buildroot}%{_cross_libexecdir}/nvidia/tesla/bin/%{nvidia_tesla_470_version}
install -m 755 nvidia-cuda-mps-server %{buildroot}%{_cross_libexecdir}/nvidia/tesla/bin/%{nvidia_tesla_470_version}
%if "%{_cross_arch}" == "x86_64"
install -m 755 nvidia-ngx-updater %{buildroot}%{_cross_libexecdir}/nvidia/tesla/bin/%{nvidia_tesla_470_version}
%endif

# TODO: add remaining libraries
# misc
# Add libnvidia-ml.so for testing purposes
install -m755 libnvidia-ml.so.%{nvidia_tesla_470_version} %{buildroot}%{_cross_libdir}/nvidia/tesla/%{nvidia_tesla_470_version}

ln -s libnvidia-ml.so.%{nvidia_tesla_470_version} %{buildroot}%{_cross_libdir}/nvidia/tesla/%{nvidia_tesla_470_version}/libnvidia-ml.so.1

popd

%files
%{_cross_attribution_file}
%dir %{_cross_libexecdir}/nvidia
%dir %{_cross_libdir}/nvidia
%dir %{_cross_datadir}/nvidia
%dir %{_cross_libdir}/modules-load.d
%dir %{_cross_factorydir}%{_cross_sysconfdir}/drivers
%{_cross_tmpfilesdir}/nvidia.conf
%{_cross_libdir}/systemd/system/
%{_cross_libdir}/modules-load.d/nvidia-dependencies.conf

%files tesla-470
%license %{license_file}
%dir %{_cross_datadir}/nvidia/tesla/%{nvidia_tesla_470_version}
%dir %{_cross_libexecdir}/nvidia/tesla/bin/%{nvidia_tesla_470_version}
%dir %{_cross_libdir}/nvidia/tesla/%{nvidia_tesla_470_version}
%dir %{_cross_datadir}/nvidia/tesla/%{nvidia_tesla_470_version}/module-objects.d
%dir %{_cross_factorydir}/nvidia/tesla/%{nvidia_tesla_470_version}

# Binaries
%{_cross_libexecdir}/nvidia/tesla/bin/%{nvidia_tesla_470_version}/nvidia-debugdump
%{_cross_libexecdir}/nvidia/tesla/bin/%{nvidia_tesla_470_version}/nvidia-smi

# Libraries
%{_cross_libdir}/nvidia/tesla/%{nvidia_tesla_470_version}/libnvidia-ml.so.1
%{_cross_libdir}/nvidia/tesla/%{nvidia_tesla_470_version}/libnvidia-ml.so.%{nvidia_tesla_470_version}

# Configuration files
%{_cross_factorydir}%{_cross_sysconfdir}/drivers/nvidia-tesla-%{nvidia_tesla_470_version}.toml
%{_cross_factorydir}%{_cross_sysconfdir}/ld.so.conf.d/nvidia-tesla-%{nvidia_tesla_470_version}.conf
%{_cross_factorydir}/nvidia/tesla/%{nvidia_tesla_470_version}/nvidia-path.env

# driver
%{_cross_datadir}/nvidia/tesla/%{nvidia_tesla_470_version}/module-objects.d/nvidia.mod.o
%{_cross_datadir}/nvidia/tesla/%{nvidia_tesla_470_version}/module-objects.d/nv-interface.o
%{_cross_datadir}/nvidia/tesla/%{nvidia_tesla_470_version}/module-objects.d/nv-kernel.o

# uvm
%{_cross_datadir}/nvidia/tesla/%{nvidia_tesla_470_version}/module-objects.d/nvidia-uvm.mod.o
%{_cross_datadir}/nvidia/tesla/%{nvidia_tesla_470_version}/module-objects.d/nvidia-uvm.o

# modeset
%{_cross_datadir}/nvidia/tesla/%{nvidia_tesla_470_version}/module-objects.d/nv-modeset-interface.o
%{_cross_datadir}/nvidia/tesla/%{nvidia_tesla_470_version}/module-objects.d/nv-modeset-kernel.o
%{_cross_datadir}/nvidia/tesla/%{nvidia_tesla_470_version}/module-objects.d/nvidia-modeset.mod.o

# tmpfiles
%{_cross_tmpfilesdir}/nvidia-tesla-%{nvidia_tesla_470_version}.conf

# Neither nvidia-peermem nor nvidia-drm are included in driver container images, we exclude them
# for now, and we will add them if requested
%exclude %{_cross_datadir}/nvidia/tesla/%{nvidia_tesla_470_version}/module-objects.d/nvidia-peermem.mod.o
%exclude %{_cross_datadir}/nvidia/tesla/%{nvidia_tesla_470_version}/module-objects.d/nvidia-peermem.o
%exclude %{_cross_datadir}/nvidia/tesla/%{nvidia_tesla_470_version}/module-objects.d/nvidia-drm.mod.o
%exclude %{_cross_datadir}/nvidia/tesla/%{nvidia_tesla_470_version}/module-objects.d/nvidia-drm.o
%exclude %{_cross_libexecdir}/nvidia/tesla/bin/%{nvidia_tesla_470_version}/nvidia-cuda-mps-control
%exclude %{_cross_libexecdir}/nvidia/tesla/bin/%{nvidia_tesla_470_version}/nvidia-cuda-mps-server
%if "%{_cross_arch}" == "x86_64"
%exclude %{_cross_libexecdir}/nvidia/tesla/bin/%{nvidia_tesla_470_version}/nvidia-ngx-updater
%endif
