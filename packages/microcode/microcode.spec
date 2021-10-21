# This is a wrapper package for binary-only microcode from Intel and AMD.
%global debug_package %{nil}

# These are specific to the upstream source RPM, and will likely need to be
# updated for each new version.
%global amd_ucode_archive linux-firmware-20200421.tar.gz
%global intel_ucode_archive microcode-20210608-1-amzn.tgz

Name: %{_cross_os}microcode
Version: 0.0
Release: 1%{?dist}
Summary: Microcode for AMD and Intel processors
License: LicenseRef-scancode-amd-linux-firmware-export AND LicenseRef-scancode-intel-mcu-2018

# Packaging AMD and Intel microcode together is specific to Bottlerocket, and
# RPM only allows one URL field per package, so this is about as accurate as we
# can be. The real upstream URLs for AMD and Intel microcode are given below in
# the subpackage definitions.
URL: https://github.com/bottlerocket-os/bottlerocket/tree/develop/packages/microcode

# We use Amazon Linux 2 as our upstream for microcode updates.
Source0: https://cdn.amazonlinux.com/blobstore/6d7f707779f6aff41c89bad00f7abe69dc70919cee29a8d3e5060f8070efe71d/linux-firmware-20200421-79.git78c0348.amzn2.src.rpm
Source1: https://cdn.amazonlinux.com/blobstore/76e8f9f15ec2b27c70aff3ca15a28df51790b25c73fc8dc1bf1f28a9069b15e8/microcode_ctl-2.1-47.amzn2.0.9.src.rpm

# Lets us install "microcode" to pull in the AMD and Intel updates.
Requires: %{_cross_os}microcode-amd
Requires: %{_cross_os}microcode-intel

%description
%{summary}.

%package amd
Summary: Microcode for AMD processors
License: LicenseRef-scancode-amd-linux-firmware-export
URL: https://git.kernel.org/pub/scm/linux/kernel/git/firmware/linux-firmware.git/tree/amd-ucode
Requires: %{_cross_os}microcode-amd-license

%description amd
%{summary}.

%package amd-license
Summary: License files for microcode for AMD processors
License: LicenseRef-scancode-amd-linux-firmware-export
URL: https://git.kernel.org/pub/scm/linux/kernel/git/firmware/linux-firmware.git/plain/LICENSE.amd-ucode

%description amd-license
%{summary}.

%package intel
Summary: Microcode for Intel processors
License: LicenseRef-scancode-intel-mcu-2018
URL: https://github.com/intel/Intel-Linux-Processor-Microcode-Data-Files
Requires: %{_cross_os}microcode-intel-license

%description intel
%{summary}.

%package intel-license
Summary: License files for microcode for Intel processors
License: LicenseRef-scancode-intel-mcu-2018
URL: https://github.com/intel/Intel-Linux-Processor-Microcode-Data-Files/blob/main/license

%description intel-license
%{summary}.

# Lets us install "microcode-licenses" for just the license files.
%package licenses
Summary: License files for microcode for AMD and Intel processors
License: LicenseRef-scancode-amd-linux-firmware-export AND LicenseRef-scancode-intel-mcu-2018
URL: https://github.com/bottlerocket-os/bottlerocket/tree/develop/packages/microcode
Requires: %{_cross_os}microcode-amd-license
Requires: %{_cross_os}microcode-intel-license

%description licenses
%{summary}.

%prep
rpm2cpio %{SOURCE0} | cpio -iu %{amd_ucode_archive}
rpm2cpio %{SOURCE1} | cpio -iu %{intel_ucode_archive}
mkdir amd intel
tar -C amd -xof %{amd_ucode_archive}
tar -C intel -xof %{intel_ucode_archive}
cp {amd/,}LICENSE.amd-ucode
cp intel/intel-ucode-with-caveats/* intel/intel-ucode
cp intel/license LICENSE.intel-ucode

# Create links to the SPDX identifiers we're using, so they're easier to match
# up with the license text.
ln -s LICENSE.intel-ucode LicenseRef-scancode-intel-mcu-2018
ln -s LICENSE.amd-ucode LicenseRef-scancode-amd-linux-firmware-export

%build

%install
install -d %{buildroot}%{_cross_libdir}/firmware/{amd,intel}-ucode
install -p -m 0644 amd/amd-ucode/*.bin %{buildroot}%{_cross_libdir}/firmware/amd-ucode
install -p -m 0644 intel/intel-ucode/* %{buildroot}%{_cross_libdir}/firmware/intel-ucode

%files

%files amd
%dir %{_cross_libdir}/firmware
%dir %{_cross_libdir}/firmware/amd-ucode
%{_cross_libdir}/firmware/amd-ucode/microcode_amd*.bin

%files amd-license
%license LICENSE.amd-ucode LicenseRef-scancode-amd-linux-firmware-export

%files intel
%dir %{_cross_libdir}/firmware
%dir %{_cross_libdir}/firmware/intel-ucode
%{_cross_libdir}/firmware/intel-ucode/??-??-??

%files intel-license
%license LICENSE.intel-ucode LicenseRef-scancode-intel-mcu-2018

%files licenses
%{_cross_attribution_file}

%changelog
