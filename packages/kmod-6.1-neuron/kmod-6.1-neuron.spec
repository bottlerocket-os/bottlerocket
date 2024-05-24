Name: %{_cross_os}kmod-6.1-neuron
Version: 2.16.7.0
Release: 1%{?dist}
Summary: Neuron drivers for the 6.1 kernel
License: GPL-2.0-only
URL: https://awsdocs-neuron.readthedocs-hosted.com/en/latest/

Source0: https://yum.repos.neuron.amazonaws.com/aws-neuronx-dkms-%{version}.noarch.rpm
Source1: neuron-modules-load.conf
Source2: neuron-systemd-modules-load.drop-in.conf

BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}kernel-6.1-archive

%description
%{summary}.

%prep
rpm2cpio %{SOURCE0} | cpio -idmv
tar -xf %{_cross_datadir}/bottlerocket/kernel-devel.tar.xz

%global neuron_sources usr/src/aws-neuronx-%{version}
%global kernel_sources %{_builddir}/kernel-devel

%build
pushd %{_builddir}/%{neuron_sources}
%make_build \
  -C %{kernel_sources} \
  M=${PWD} \
  ARCH=%{_cross_karch} \
  CROSS_COMPILE=%{_cross_target}- \
  INSTALL_MOD_STRIP=1 \
  %{nil}
gzip -9 neuron.ko
popd

%install
pushd %{_builddir}/%{neuron_sources}
export KVER="$(cat %{kernel_sources}/include/config/kernel.release)"
export KMODDIR="%{_cross_libdir}/modules/${KVER}/extra"
install -d "%{buildroot}${KMODDIR}"
install -p -m 0644 neuron.ko.gz "%{buildroot}${KMODDIR}"
popd

# Install modules-load.d drop-in to autoload required kernel modules
install -d %{buildroot}%{_cross_libdir}/modules-load.d
install -p -m 0644 %{S:1} %{buildroot}%{_cross_libdir}/modules-load.d/neuron.conf

# Install systemd-modules-load drop-in to ensure that depmod runs.
install -d %{buildroot}%{_cross_unitdir}/systemd-modules-load.service.d
install -p -m 0644 %{S:2} %{buildroot}%{_cross_unitdir}/systemd-modules-load.service.d/neuron.conf

%files
%license %{neuron_sources}/LICENSE
%{_cross_attribution_file}
%{_cross_libdir}/modules/*/extra/neuron.ko.gz
%{_cross_libdir}/modules-load.d/neuron.conf
%{_cross_unitdir}/systemd-modules-load.service.d/neuron.conf
