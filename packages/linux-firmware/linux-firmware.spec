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
