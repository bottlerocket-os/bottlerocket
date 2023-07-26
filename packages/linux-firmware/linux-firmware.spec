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
License: GPL+ and GPLv2+ and MIT and Redistributable, no modification permitted
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

# Here we have potential to shave off some extra space by using `install-xz` of
# `install-zst` to compress firmware images on disk. However, that functionality
# relies on kernels being configured with `CONFIG_FW_LOADER_COMPRESS_[ZSTD|XZ]`
# which we currently do not have.
make DESTDIR=%{buildroot}/ FIRMWAREDIR=%{fwdir} install


# Remove executable bits from random firmware
pushd %{buildroot}/%{fwdir}
find . -type f -executable -exec chmod -x {} \;
popd

%files
%dir %{fwdir}
%{fwdir}/*
%license LICENCE.* LICENSE.* GPL* WHENCE
%{_cross_attribution_file}
