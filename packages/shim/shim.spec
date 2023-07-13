%global debug_package %{nil}
%global __strip %{_bindir}/true

%global efidir /boot/efi/EFI/BOOT
%global boot_efi_image boot%{_cross_efi_arch}.efi
%global grub_efi_image grub%{_cross_efi_arch}.efi
%global shim_efi_image shim%{_cross_efi_arch}.efi
%global mokm_efi_image mm%{_cross_efi_arch}.efi

%global shimver 15.7
%global gnuefiver 15.6
%global commit 11491619f4336fef41c3519877ba242161763580

Name: %{_cross_os}shim
Version: %{shimver}
Release: 1%{?dist}
Summary: UEFI shim loader
License: BSD-3-Clause
URL: https://github.com/rhboot/shim/
Source0: https://github.com/rhboot/shim/archive/%{shimver}/shim-%{shimver}.tar.gz
Source1: https://github.com/rhboot/gnu-efi/archive/refs/heads/shim-%{gnuefiver}.tar.gz#/gnu-efi-shim-%{gnuefiver}.tar.gz

%description
%{summary}.

%prep
%autosetup -n shim-%{shimver} -p1
%setup -T -D -n shim-%{shimver} -a 1
rmdir gnu-efi
mv gnu-efi-shim-%{gnuefiver} gnu-efi

# Make sure the `.vendor_cert` section is large enough to cover a replacement
# certificate, or `objcopy` may silently retain the existing section.
# 4096 - 16 (for cert_table structure) = 4080 bytes.
truncate -s 4080 empty.cer

%global shim_make \
%make_build\\\
  ARCH="%{_cross_arch}"\\\
  CROSS_COMPILE="%{_cross_target}-"\\\
  COMMIT_ID="%{commit}"\\\
  RELEASE="%{release}"\\\
  DEFAULT_LOADER="%{grub_efi_image}"\\\
  DISABLE_REMOVABLE_LOAD_OPTIONS=y\\\
  DESTDIR="%{buildroot}"\\\
  EFIDIR="BOOT"\\\
  VENDOR_CERT_FILE="empty.cer"\\\
%{nil}

%build
%shim_make

%install
%shim_make install-as-data
install -d %{buildroot}%{efidir}
find %{buildroot}%{_datadir} -name '%{shim_efi_image}' -exec \
  mv {} "%{buildroot}%{efidir}/%{boot_efi_image}" \;
find %{buildroot}%{_datadir} -name '%{mokm_efi_image}' -exec \
  mv {} "%{buildroot}%{efidir}/%{mokm_efi_image}" \;
rm -rf %{buildroot}%{_datadir}

%files
%license COPYRIGHT
%{_cross_attribution_file}
%dir %{efidir}
%{efidir}/%{boot_efi_image}
%{efidir}/%{mokm_efi_image}
