%global debug_package %{nil}

%global fwdir %{_cross_libdir}/firmware

# Many of the firmware files have specialized binary formats that are not supported
# by the strip binary used in __spec_install_post macro. Work around build failures
# by skipping striping.
%global __strip /usr/bin/true

Name: %{_cross_os}linux-firmware
Version: 20230625
Release: 1%{?dist}
Summary: Firmware files used by the Linux kernel
# The following list of SPDX identifiers was constructed with help of scancode
# tooling and has turned up the following licenses for different drivers by
# checking the different LICENCE/LICENSE files and the licenses in WHENCE:
# * BSD-Source-Code - myri10ge
# * LicenseRef-scancode-chelsio-linux-firmware - cxgb4
# * LicenseRef-scancode-qlogic-firmware - netxen_nic
# * LicenseRef-scancode-intel - i915, ice
# * LicenseRef-scancode-proprietary-license - bnx2x, qed
# * LicenseRef-scancode-free-unknown - tg3
License: GPL-1.0-or-later AND GPL-2.0-or-later AND BSD-Source-Code AND LicenseRef-scancode-chelsio-linux-firmware AND LicenseRef-scancode-qlogic-firmware AND LicenseRef-scancode-intel AND LicenseRef-scancode-proprietary-license AND LicenseRef-scancode-free-unknown
URL: https://www.kernel.org/

Source0: https://www.kernel.org/pub/linux/kernel/firmware/linux-firmware-%{version}.tar.xz

Patch0001: 0001-linux-firmware-snd-remove-firmware-for-snd-audio-dev.patch
Patch0002: 0002-linux-firmware-video-Remove-firmware-for-video-broad.patch
Patch0003: 0003-linux-firmware-bt-wifi-Remove-firmware-for-Bluetooth.patch
Patch0004: 0004-linux-firmware-scsi-Remove-firmware-for-SCSI-devices.patch
Patch0005: 0005-linux-firmware-usb-remove-firmware-for-USB-Serial-PC.patch
Patch0006: 0006-linux-firmware-ethernet-Remove-firmware-for-ethernet.patch
Patch0007: 0007-linux-firmware-Remove-firmware-for-Accelarator-devic.patch
Patch0008: 0008-linux-firmware-gpu-Remove-firmware-for-GPU-devices.patch
Patch0009: 0009-linux-firmware-various-Remove-firmware-for-various-d.patch
Patch0010: 0010-linux-firmware-amd-ucode-Remove-amd-microcode.patch

%description
%{summary}.

%prep
%autosetup -n linux-firmware-%{version} -p1

%build

%install
mkdir -p %{buildroot}/%{fwdir}
mkdir -p %{buildroot}/%{fwdir}/updates

# Use xz compression for firmware files to reduce size on disk. This relies on
# kernel support through FW_LOADER_COMPRESS (and FW_LOADER_COMPRESS_XZ for kernels >=5.19)
make DESTDIR=%{buildroot}/ FIRMWAREDIR=%{fwdir} install-xz

%files
%dir %{fwdir}
%{fwdir}/*
%license LICENCE.* LICENSE.* GPL* WHENCE
%{_cross_attribution_file}
