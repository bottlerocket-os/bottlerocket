# This is a wrapper package for binary-only microcode from Intel and AMD.
%global debug_package %{nil}

# These are specific to the upstream source RPM, and will likely need to be
# updated for each new version.
%global amd_ucode_version 20230804
%global intel_ucode_version 20230808

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

Source0: https://www.kernel.org/pub/linux/kernel/firmware/linux-firmware-%{amd_ucode_version}.tar.xz
Source1: https://github.com/intel/Intel-Linux-Processor-Microcode-Data-Files/archive/refs/tags/microcode-%{intel_ucode_version}.tar.gz

Patch1: 0001-linux-firmware-Update-AMD-cpu-microcode.patch

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
mkdir amd intel
tar -C amd --strip-components=1 -xof %{SOURCE0}
tar -C intel --strip-components=1 -xof %{SOURCE1}
# CVE-2023-20569 - "AMD Inception"
# This is adding new microcode for Zen3/Zen4 AMD cpus. The patch was taken
# directly from the linux-firmware repository, but has not been part of a
# release there, yet.
# Unfortunately the setup here with two separate sources being brought into
# separate directories and the patch only affecting one of the two is not conducive
# of using the standard way of applying git binary patches through `autosetup -S git ...`
# Hence we have to extract some of the parts from that macro to let the patch
# apply.
#
# As soon as we update to a release that includes this patch everything from here...
pushd amd
%global __scm git
%__scm_setup_git
%autopatch -p1
popd
# ... to here can be dropped
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
%exclude %{_cross_libdir}/firmware/intel-ucode/??-??-??_DUPLICATE

%files intel-license
%license LICENSE.intel-ucode LicenseRef-scancode-intel-mcu-2018

%files licenses
%{_cross_attribution_file}

%changelog
