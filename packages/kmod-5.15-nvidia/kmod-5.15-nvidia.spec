%global tesla_515 515.86.01
%global tesla_515_libdir %{_cross_libdir}/nvidia/tesla/%{tesla_515}
%global tesla_515_bindir %{_cross_libexecdir}/nvidia/tesla/bin/%{tesla_515}
%global tesla_515_firmwaredir %{_cross_libdir}/firmware/nvidia/%{tesla_515}
%global spdx_id %(bottlerocket-license-tool -l %{_builddir}/Licenses.toml spdx-id nvidia)
%global license_file %(bottlerocket-license-tool -l %{_builddir}/Licenses.toml path nvidia -p ./licenses)

Name: %{_cross_os}kmod-5.15-nvidia
Version: 1.0.0
Release: 1%{?dist}
Summary: NVIDIA drivers for the 5.15 kernel
# We use these licences because we only ship our own software in the main package,
# each subpackage includes the LICENSE file provided by the Licenses.toml file
License: Apache-2.0 OR MIT
URL: http://www.nvidia.com/

# NVIDIA .run scripts from 0 to 199
Source0: https://us.download.nvidia.com/tesla/%{tesla_515}/NVIDIA-Linux-x86_64-%{tesla_515}.run
Source1: https://us.download.nvidia.com/tesla/%{tesla_515}/NVIDIA-Linux-aarch64-%{tesla_515}.run

# Common NVIDIA conf files from 200 to 299
Source200: nvidia-tmpfiles.conf.in
Source202: nvidia-dependencies-modules-load.conf

# NVIDIA tesla conf files from 300 to 399
Source300: nvidia-tesla-tmpfiles.conf.in
Source301: nvidia-tesla-build-config.toml.in
Source302: nvidia-tesla-path.env.in
Source303: nvidia-ld.so.conf.in

BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}kernel-5.15-archive

%description
%{summary}.

%package tesla-515
Summary: NVIDIA 515 Tesla driver
Version: %{tesla_515}
License: %{spdx_id}
Requires: %{name}

%description tesla-515
%{summary}

%prep
# Extract nvidia sources with `-x`, otherwise the script will try to install
# the driver in the current run
sh %{_sourcedir}/NVIDIA-Linux-%{_cross_arch}-%{tesla_515}.run -x

%global kernel_sources %{_builddir}/kernel-devel
tar -xf %{_cross_datadir}/bottlerocket/kernel-devel.tar.xz

%build
pushd NVIDIA-Linux-%{_cross_arch}-%{tesla_515}/kernel

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
sed \
  -e "s|__KERNEL_VERSION__|${KERNEL_VERSION}|" \
  -e "s|__PREFIX__|%{_cross_prefix}|" %{S:200} > nvidia.conf
install -p -m 0644 nvidia.conf %{buildroot}%{_cross_tmpfilesdir}

# Install modules-load.d drop-in to autoload required kernel modules
install -d %{buildroot}%{_cross_libdir}/modules-load.d
install -p -m 0644 %{S:202} %{buildroot}%{_cross_libdir}/modules-load.d/nvidia-dependencies.conf

# Begin NVIDIA tesla 515
pushd NVIDIA-Linux-%{_cross_arch}-%{tesla_515}
# We install bins and libs in a versioned directory to prevent collisions with future drivers versions
install -d %{buildroot}%{_cross_libexecdir}/nvidia/tesla/bin/%{tesla_515}
install -d %{buildroot}%{tesla_515_libdir}
install -d %{buildroot}%{_cross_datadir}/nvidia/tesla/%{tesla_515}/module-objects.d
install -d %{buildroot}%{_cross_factorydir}/nvidia/tesla/%{tesla_515}

sed -e 's|__NVIDIA_VERSION__|%{tesla_515}|' %{S:300} > nvidia-tesla-%{tesla_515}.conf
install -m 0644 nvidia-tesla-%{tesla_515}.conf %{buildroot}%{_cross_tmpfilesdir}/
sed -e 's|__NVIDIA_MODULES__|%{_cross_datadir}/nvidia/tesla/%{tesla_515}/module-objects.d/|' %{S:301} > \
  nvidia-tesla-%{tesla_515}.toml
install -m 0644 nvidia-tesla-%{tesla_515}.toml %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}/drivers
# Install nvidia-path environment file, will be used as a drop-in for containerd.service since
# libnvidia-container locates and mounts helper binaries into the containers from either
# `PATH` or `NVIDIA_PATH`
sed -e 's|__NVIDIA_BINDIR__|%{_cross_libexecdir}/nvidia/tesla/bin/%{tesla_515}|' %{S:302} > nvidia-path.env
install -m 0644 nvidia-path.env %{buildroot}%{_cross_factorydir}/nvidia/tesla/%{tesla_515}
# We need to add `_cross_libdir/tesla_515` to the paths loaded by the ldconfig service
# because libnvidia-container uses the `ldcache` file created by the service, to locate and mount the
# libraries into the containers
sed -e 's|__LIBDIR__|%{_cross_libdir}|' %{S:303} | sed -e 's|__NVIDIA_VERSION__|%{tesla_515}|' \
  > nvidia-tesla-%{tesla_515}.conf
install -m 0644 nvidia-tesla-%{tesla_515}.conf %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}/ld.so.conf.d/

# driver
install kernel/nvidia.mod.o %{buildroot}%{_cross_datadir}/nvidia/tesla/%{tesla_515}/module-objects.d
install kernel/nvidia/nv-interface.o %{buildroot}%{_cross_datadir}/nvidia/tesla/%{tesla_515}/module-objects.d
install kernel/nvidia/nv-kernel.o_binary %{buildroot}%{_cross_datadir}/nvidia/tesla/%{tesla_515}/module-objects.d/nv-kernel.o

# uvm
install kernel/nvidia-uvm.mod.o %{buildroot}%{_cross_datadir}/nvidia/tesla/%{tesla_515}/module-objects.d
install kernel/nvidia-uvm.o %{buildroot}%{_cross_datadir}/nvidia/tesla/%{tesla_515}/module-objects.d

# modeset
install kernel/nvidia-modeset.mod.o %{buildroot}%{_cross_datadir}/nvidia/tesla/%{tesla_515}/module-objects.d
install kernel/nvidia-modeset/nv-modeset-interface.o %{buildroot}%{_cross_datadir}/nvidia/tesla/%{tesla_515}/module-objects.d
install kernel/nvidia-modeset/nv-modeset-kernel.o %{buildroot}%{_cross_datadir}/nvidia/tesla/%{tesla_515}/module-objects.d

# peermem
install kernel/nvidia-peermem.mod.o %{buildroot}%{_cross_datadir}/nvidia/tesla/%{tesla_515}/module-objects.d
install kernel/nvidia-peermem/nvidia-peermem.o %{buildroot}%{_cross_datadir}/nvidia/tesla/%{tesla_515}/module-objects.d

# drm
install kernel/nvidia-drm.mod.o %{buildroot}/%{_cross_datadir}/nvidia/tesla/%{tesla_515}/module-objects.d
install kernel/nvidia-drm.o %{buildroot}/%{_cross_datadir}/nvidia/tesla/%{tesla_515}/module-objects.d

# Binaries
install -m 755 nvidia-smi %{buildroot}%{_cross_libexecdir}/nvidia/tesla/bin/%{tesla_515}
install -m 755 nvidia-debugdump %{buildroot}%{_cross_libexecdir}/nvidia/tesla/bin/%{tesla_515}
install -m 755 nvidia-cuda-mps-control %{buildroot}%{_cross_libexecdir}/nvidia/tesla/bin/%{tesla_515}
install -m 755 nvidia-cuda-mps-server %{buildroot}%{_cross_libexecdir}/nvidia/tesla/bin/%{tesla_515}
%if "%{_cross_arch}" == "x86_64"
install -m 755 nvidia-ngx-updater %{buildroot}%{_cross_libexecdir}/nvidia/tesla/bin/%{tesla_515}
%endif

# We install all the libraries, and filter them out in the 'files' section, so we can catch
# when new libraries are added
install -m 755 *.so* %{buildroot}/%{tesla_515_libdir}/

# This library has the same SONAME as libEGL.so.1.1.0, this will cause collisions while
# the symlinks are created. For now, we only symlink libEGL.so.1.1.0.
EXCLUDED_LIBS="libEGL.so.%{tesla_515}"

for lib in $(find . -maxdepth 1 -type f -name 'lib*.so.*' -printf '%%P\n'); do
  [[ "${EXCLUDED_LIBS}" =~ "${lib}" ]] && continue
  soname="$(%{_cross_target}-readelf -d "${lib}" | awk '/SONAME/{print $5}' | tr -d '[]')"
  [ -n "${soname}" ] || continue
  [ "${lib}" == "${soname}" ] && continue
  ln -s "${lib}" %{buildroot}/%{tesla_515_libdir}/"${soname}"
done

# Include the firmware file for GSP support
install -d %{buildroot}%{tesla_515_firmwaredir}
install -p -m 0644 firmware/gsp.bin %{buildroot}%{tesla_515_firmwaredir}

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

%files tesla-515
%license %{license_file}
%dir %{_cross_datadir}/nvidia/tesla/%{tesla_515}
%dir %{_cross_libexecdir}/nvidia/tesla/bin/%{tesla_515}
%dir %{tesla_515_libdir}
%dir %{tesla_515_firmwaredir}
%dir %{_cross_datadir}/nvidia/tesla/%{tesla_515}/module-objects.d
%dir %{_cross_factorydir}/nvidia/tesla/%{tesla_515}

# Binaries
%{_cross_libexecdir}/nvidia/tesla/bin/%{tesla_515}/nvidia-debugdump
%{_cross_libexecdir}/nvidia/tesla/bin/%{tesla_515}/nvidia-smi

# Configuration files
%{_cross_factorydir}%{_cross_sysconfdir}/drivers/nvidia-tesla-%{tesla_515}.toml
%{_cross_factorydir}%{_cross_sysconfdir}/ld.so.conf.d/nvidia-tesla-%{tesla_515}.conf
%{_cross_factorydir}/nvidia/tesla/%{tesla_515}/nvidia-path.env

# driver
%{_cross_datadir}/nvidia/tesla/%{tesla_515}/module-objects.d/nvidia.mod.o
%{_cross_datadir}/nvidia/tesla/%{tesla_515}/module-objects.d/nv-interface.o
%{_cross_datadir}/nvidia/tesla/%{tesla_515}/module-objects.d/nv-kernel.o

# uvm
%{_cross_datadir}/nvidia/tesla/%{tesla_515}/module-objects.d/nvidia-uvm.mod.o
%{_cross_datadir}/nvidia/tesla/%{tesla_515}/module-objects.d/nvidia-uvm.o

# modeset
%{_cross_datadir}/nvidia/tesla/%{tesla_515}/module-objects.d/nv-modeset-interface.o
%{_cross_datadir}/nvidia/tesla/%{tesla_515}/module-objects.d/nv-modeset-kernel.o
%{_cross_datadir}/nvidia/tesla/%{tesla_515}/module-objects.d/nvidia-modeset.mod.o

# tmpfiles
%{_cross_tmpfilesdir}/nvidia-tesla-%{tesla_515}.conf

# We only install the libraries required by all the DRIVER_CAPABILITIES, described here:
# https://docs.nvidia.com/datacenter/cloud-native/container-toolkit/user-guide.html#driver-capabilities

# Utility libs
%{tesla_515_libdir}/libnvidia-ml.so.%{tesla_515}
%{tesla_515_libdir}/libnvidia-ml.so.1
%{tesla_515_libdir}/libnvidia-cfg.so.%{tesla_515}
%{tesla_515_libdir}/libnvidia-cfg.so.1
%{tesla_515_libdir}/libnvidia-nvvm.so.4
%{tesla_515_libdir}/libnvidia-nvvm.so.%{tesla_515}

# Compute libs
%{tesla_515_libdir}/libcuda.so.%{tesla_515}
%{tesla_515_libdir}/libcuda.so.1
%{tesla_515_libdir}/libnvidia-opencl.so.%{tesla_515}
%{tesla_515_libdir}/libnvidia-opencl.so.1
%{tesla_515_libdir}/libnvidia-ptxjitcompiler.so.%{tesla_515}
%{tesla_515_libdir}/libnvidia-ptxjitcompiler.so.1
%{tesla_515_libdir}/libnvidia-allocator.so.%{tesla_515}
%{tesla_515_libdir}/libnvidia-allocator.so.1
%{tesla_515_libdir}/libOpenCL.so.1.0.0
%{tesla_515_libdir}/libOpenCL.so.1
%if "%{_cross_arch}" == "x86_64"
%{tesla_515_libdir}/libnvidia-compiler.so.%{tesla_515}
%endif

# Video libs
%{tesla_515_libdir}/libvdpau_nvidia.so.%{tesla_515}
%{tesla_515_libdir}/libvdpau_nvidia.so.1
%{tesla_515_libdir}/libnvidia-encode.so.%{tesla_515}
%{tesla_515_libdir}/libnvidia-encode.so.1
%{tesla_515_libdir}/libnvidia-opticalflow.so.%{tesla_515}
%{tesla_515_libdir}/libnvidia-opticalflow.so.1
%{tesla_515_libdir}/libnvcuvid.so.%{tesla_515}
%{tesla_515_libdir}/libnvcuvid.so.1

# Graphics libs
%{tesla_515_libdir}/libnvidia-eglcore.so.%{tesla_515}
%{tesla_515_libdir}/libnvidia-glcore.so.%{tesla_515}
%{tesla_515_libdir}/libnvidia-tls.so.%{tesla_515}
%{tesla_515_libdir}/libnvidia-glsi.so.%{tesla_515}
%{tesla_515_libdir}/libnvidia-rtcore.so.%{tesla_515}
%{tesla_515_libdir}/libnvidia-fbc.so.%{tesla_515}
%{tesla_515_libdir}/libnvidia-fbc.so.1
%{tesla_515_libdir}/libnvoptix.so.%{tesla_515}
%{tesla_515_libdir}/libnvoptix.so.1
%{tesla_515_libdir}/libnvidia-vulkan-producer.so.%{tesla_515}

# Graphics GLVND libs
%{tesla_515_libdir}/libnvidia-glvkspirv.so.%{tesla_515}
%{tesla_515_libdir}/libGLX_nvidia.so.%{tesla_515}
%{tesla_515_libdir}/libGLX_nvidia.so.0
%{tesla_515_libdir}/libEGL_nvidia.so.%{tesla_515}
%{tesla_515_libdir}/libEGL_nvidia.so.0
%{tesla_515_libdir}/libGLESv2_nvidia.so.%{tesla_515}
%{tesla_515_libdir}/libGLESv2_nvidia.so.2
%{tesla_515_libdir}/libGLESv1_CM_nvidia.so.%{tesla_515}
%{tesla_515_libdir}/libGLESv1_CM_nvidia.so.1

# Graphics compat
%{tesla_515_libdir}/libEGL.so.1.1.0
%{tesla_515_libdir}/libEGL.so.1
%{tesla_515_libdir}/libEGL.so.%{tesla_515}
%{tesla_515_libdir}/libGL.so.1.7.0
%{tesla_515_libdir}/libGL.so.1
%{tesla_515_libdir}/libGLESv1_CM.so.1.2.0
%{tesla_515_libdir}/libGLESv1_CM.so.1
%{tesla_515_libdir}/libGLESv2.so.2.1.0
%{tesla_515_libdir}/libGLESv2.so.2

# NGX
%{tesla_515_libdir}/libnvidia-ngx.so.%{tesla_515}
%{tesla_515_libdir}/libnvidia-ngx.so.1

# Firmware
%{tesla_515_firmwaredir}/gsp.bin

# Neither nvidia-peermem nor nvidia-drm are included in driver container images, we exclude them
# for now, and we will add them if requested
%exclude %{_cross_datadir}/nvidia/tesla/%{tesla_515}/module-objects.d/nvidia-peermem.mod.o
%exclude %{_cross_datadir}/nvidia/tesla/%{tesla_515}/module-objects.d/nvidia-peermem.o
%exclude %{_cross_datadir}/nvidia/tesla/%{tesla_515}/module-objects.d/nvidia-drm.mod.o
%exclude %{_cross_datadir}/nvidia/tesla/%{tesla_515}/module-objects.d/nvidia-drm.o
%exclude %{_cross_libexecdir}/nvidia/tesla/bin/%{tesla_515}/nvidia-cuda-mps-control
%exclude %{_cross_libexecdir}/nvidia/tesla/bin/%{tesla_515}/nvidia-cuda-mps-server
%if "%{_cross_arch}" == "x86_64"
%exclude %{_cross_libexecdir}/nvidia/tesla/bin/%{tesla_515}/nvidia-ngx-updater
%endif

# None of these libraries are required by libnvidia-container, so they
# won't be used by a containerized workload
%exclude %{tesla_515_libdir}/libGLX.so.0
%exclude %{tesla_515_libdir}/libGLdispatch.so.0
%exclude %{tesla_515_libdir}/libOpenGL.so.0
%exclude %{tesla_515_libdir}/libglxserver_nvidia.so.%{tesla_515}
%exclude %{tesla_515_libdir}/libnvidia-gtk2.so.%{tesla_515}
%exclude %{tesla_515_libdir}/libnvidia-gtk3.so.%{tesla_515}
%exclude %{tesla_515_libdir}/nvidia_drv.so
%exclude %{tesla_515_libdir}/libnvidia-egl-wayland.so.1
%exclude %{tesla_515_libdir}/libnvidia-egl-gbm.so.1
%exclude %{tesla_515_libdir}/libnvidia-egl-gbm.so.1.1.0
%exclude %{tesla_515_libdir}/libnvidia-egl-wayland.so.1.1.9
%exclude %{tesla_515_libdir}/libnvidia-wayland-client.so.%{tesla_515}
