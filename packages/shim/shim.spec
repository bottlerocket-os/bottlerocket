%global debug_package %{nil}
%global __strip %{_bindir}/true

%global efidir /boot/efi/EFI/BOOT
%global boot_efi_image boot%{_cross_efi_arch}.efi
%global grub_efi_image grub%{_cross_efi_arch}.efi
%global shim_efi_image shim%{_cross_efi_arch}.efi
%global mokm_efi_image mm%{_cross_efi_arch}.efi

%global shimver 15.8
%global commit 5914984a1ffeab841f482c791426d7ca9935a5e6

Name: %{_cross_os}shim
Version: %{shimver}
Release: 1%{?dist}
Summary: UEFI shim loader
License: BSD-3-Clause
URL: https://github.com/rhboot/shim/
Source0: https://github.com/rhboot/shim/archive/%{shimver}/shim-%{shimver}.tar.bz2

%description
%{summary}.

%prep
%autosetup -n shim-%{shimver} -p1

# Make sure the `.vendor_cert` section is large enough to cover a replacement
# certificate, or `objcopy` may silently retain the existing section.
# 4096 - 16 (for cert_table structure) = 4080 bytes.
truncate -s 4080 empty.cer

%global shim_make \
make\\\
  ARCH="%{_cross_arch}"\\\
  CROSS_COMPILE="%{_cross_target}-"\\\
  COMMIT_ID="%{commit}"\\\
  RELEASE="%{release}"\\\
  DEFAULT_LOADER="%{grub_efi_image}"\\\
  DISABLE_REMOVABLE_LOAD_OPTIONS=y\\\
  DESTDIR="%{buildroot}"\\\
  EFIDIR="BOOT"\\\
  VENDOR_CERT_FILE="empty.cer"\\\
  POST_PROCESS_PE_FLAGS="-N"\\\
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
