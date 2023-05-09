%global debug_package %{nil}
%global __strip %{_bindir}/true

%global efidir /boot/efi/EFI/BOOT
%global efi_image grub%{_cross_efi_arch}.efi
%global biosdir /boot/grub

# This is specific to the upstream source RPM, and will likely need to be
# updated for each new version.
%global gnulib_version gnulib-9f48fb992a3d7e96610c4ce8be969cff2d61a01b

Name: %{_cross_os}grub
Version: 2.06
Release: 1%{?dist}
Summary: Bootloader with support for Linux and more
License: GPL-3.0-or-later AND Unicode-DFS-2015
URL: https://www.gnu.org/software/grub/
Source0: https://cdn.amazonlinux.com/al2023/blobstore/74f9ee6e75b8f89fe91ccda86896243179968a8664ba045bece11dc5aff61f4e/grub2-2.06-61.amzn2023.0.6.src.rpm
Source1: bios.cfg
Source2: efi.cfg
Source3: sbat.csv.in
Patch0001: 0001-setup-Add-root-device-argument-to-grub-setup.patch
Patch0002: 0002-gpt-start-new-GPT-module.patch
Patch0003: 0003-gpt-rename-misnamed-header-location-fields.patch
Patch0004: 0004-gpt-record-size-of-of-the-entries-table.patch
Patch0005: 0005-gpt-consolidate-crc32-computation-code.patch
Patch0006: 0006-gpt-add-new-repair-function-to-sync-up-primary-and-b.patch
Patch0007: 0007-gpt-add-write-function-and-gptrepair-command.patch
Patch0008: 0008-gpt-add-a-new-generic-GUID-type.patch
Patch0009: 0009-gpt-new-gptprio.next-command-for-selecting-priority-.patch
Patch0010: 0010-gpt-split-out-checksum-recomputation.patch
Patch0011: 0011-gpt-move-gpt-guid-printing-function-to-common-librar.patch
Patch0012: 0012-gpt-switch-partition-names-to-a-16-bit-type.patch
Patch0013: 0013-tests-add-some-partitions-to-the-gpt-unit-test-data.patch
Patch0014: 0014-gpt-add-search-by-partition-label-and-uuid-commands.patch
Patch0015: 0015-gpt-clean-up-little-endian-crc32-computation.patch
Patch0016: 0016-gpt-minor-cleanup.patch
Patch0017: 0017-gpt-add-search-by-disk-uuid-command.patch
Patch0018: 0018-gpt-do-not-use-disk-sizes-GRUB-will-reject-as-invali.patch
Patch0019: 0019-gpt-add-verbose-debug-logging.patch
Patch0020: 0020-gpt-improve-validation-of-GPT-headers.patch
Patch0021: 0021-gpt-refuse-to-write-to-sector-0.patch
Patch0022: 0022-gpt-properly-detect-and-repair-invalid-tables.patch
Patch0023: 0023-gptrepair_test-fix-typo-in-cleanup-trap.patch
Patch0024: 0024-gptprio_test-check-GPT-is-repaired-when-appropriate.patch
Patch0025: 0025-gpt-fix-partition-table-indexing-and-validation.patch
Patch0026: 0026-gpt-prefer-disk-size-from-header-over-firmware.patch
Patch0027: 0027-gpt-add-helper-for-picking-a-valid-header.patch
Patch0028: 0028-gptrepair-fix-status-checking.patch
Patch0029: 0029-gpt-use-inline-functions-for-checking-status-bits.patch
Patch0030: 0030-gpt-allow-repair-function-to-noop.patch
Patch0031: 0031-gpt-do-not-use-an-enum-for-status-bit-values.patch
Patch0032: 0032-gpt-check-header-and-entries-status-bits-together.patch
Patch0033: 0033-gpt-be-more-careful-about-relocating-backup-header.patch
Patch0034: 0034-gpt-selectively-update-fields-during-repair.patch
Patch0035: 0035-gpt-always-revalidate-when-recomputing-checksums.patch
Patch0036: 0036-gpt-include-backup-in-sync-check-in-revalidation.patch
Patch0037: 0037-gpt-read-entries-table-at-the-same-time-as-the-heade.patch
Patch0038: 0038-gpt-report-all-revalidation-errors.patch
Patch0039: 0039-gpt-rename-and-update-documentation-for-grub_gpt_upd.patch
Patch0040: 0040-gpt-write-backup-GPT-first-skip-if-inaccessible.patch
Patch0041: 0041-gptprio-Use-Bottlerocket-boot-partition-type-GUID.patch
Patch0042: 0042-util-mkimage-Bump-EFI-PE-header-size-to-accommodate-.patch
Patch0043: 0043-util-mkimage-avoid-adding-section-table-entry-outsid.patch
Patch0044: 0044-efi-return-virtual-size-of-section-found-by-grub_efi.patch
Patch0045: 0045-mkimage-pgp-move-single-public-key-into-its-own-sect.patch

BuildRequires: automake
BuildRequires: bison
BuildRequires: flex
BuildRequires: gettext-devel

%description
%{summary}.

%package modules
Summary: Modules for the bootloader with support for Linux and more

%description modules
%{summary}.

%package tools
Summary: Tools for the bootloader with support for Linux and more

%description tools
%{summary}.

%prep
rpm2cpio %{S:0} | cpio -iu grub-%{version}.tar.xz \
  bootstrap bootstrap.conf \
  gitignore %{gnulib_version}.tar.gz \
  "*.patch"

# Mimic prep from upstream spec to prepare for patching.
tar -xof grub-%{version}.tar.xz; rm grub-%{version}.tar.xz
%setup -TDn grub-%{version}
mv ../bootstrap{,.conf} .
mv ../gitignore .gitignore
tar -xof ../%{gnulib_version}.tar.gz; rm ../%{gnulib_version}.tar.gz
mv %{gnulib_version} gnulib
cp unicode/COPYING COPYING.unicode
rm -f configure

# Apply upstream and local patches.
git init
git config user.email 'user@localhost'
git config user.name 'user'
git add .
git commit -a -q -m "base"
git am --whitespace=nowarn ../*.patch %{patches}

# Let bootstrap start from a clean slate and freshly copy in the relevant
# parts from gnulib. In particular remove the configure macros that aren't
# compatible with the copied in version of gnulib.
rm -r build-aux m4

./bootstrap

%global grub_cflags -pipe -fno-stack-protector -fno-strict-aliasing
%global grub_ldflags -static
%global _configure ../configure

%build
export \
  TARGET_CPP="%{_cross_target}-gcc -E" \
  TARGET_CC="%{_cross_target}-gcc" \
  TARGET_CFLAGS="%{grub_cflags}" \
  TARGET_CPPFLAGS="%{grub_cflags}" \
  TARGET_LDFLAGS="%{grub_ldflags}" \
  TARGET_NM="%{_cross_target}-nm" \
  TARGET_OBJCOPY="%{_cross_target}-objcopy" \
  TARGET_STRIP="%{_cross_target}-strip" \
  PYTHON="python3" \

%if "%{_cross_arch}" == "x86_64"
mkdir bios-build
pushd bios-build

%cross_configure \
  CFLAGS="" \
  LDFLAGS="" \
  --host="%{_build}" \
  --target="i386" \
  --with-platform="pc" \
  --with-utils=host \
  --disable-grub-mkfont \
  --disable-rpm-sort \
  --disable-werror \
  --enable-efiemu=no \
  --enable-device-mapper=no \
  --enable-libzfs=no \

%make_build
popd
%endif

mkdir efi-build
pushd efi-build

sed -e "s,__VERSION__,%{version},g" %{S:3} > sbat.csv

%cross_configure \
  CFLAGS="" \
  LDFLAGS="" \
  --host="%{_build}" \
  --target="%{_cross_arch}" \
  --with-platform="efi" \
  --with-utils=host \
  --disable-grub-mkfont \
  --disable-rpm-sort \
  --disable-werror \
  --enable-efiemu=no \
  --enable-device-mapper=no \
  --enable-libzfs=no \

%make_build
popd

%install
MODS=(configfile echo ext2 gptprio linux normal part_gpt reboot sleep zstd search)

# These modules are needed for signature verification, which is currently only
# done for the EFI build of GRUB.
VERIFY_MODS=(pgp crypto gcry_sha256 gcry_sha512 gcry_dsa gcry_rsa)

%if "%{_cross_arch}" == "x86_64"
pushd bios-build
%make_install
mkdir -p %{buildroot}%{biosdir}
%{buildroot}%{_cross_bindir}/grub-mkimage \
  -c %{S:1} \
  -d ./grub-core/ \
  -O "i386-pc" \
  -o "%{buildroot}%{biosdir}/core.img" \
  -p "(hd0,gpt2)/boot/grub" \
  biosdisk serial ${MODS[@]}
install -m 0644 ./grub-core/boot.img \
  %{buildroot}%{biosdir}/boot.img
popd
%endif

pushd efi-build
%make_install
mkdir -p %{buildroot}%{efidir}

# Make sure the `.pubkey` section is large enough to cover a replacement
# certificate, or `objcopy` may silently retain the existing section.
truncate -s 4096 empty.pubkey

%{buildroot}%{_cross_bindir}/grub-mkimage \
  -c %{S:2} \
  -d ./grub-core/ \
  -O "%{_cross_grub_efi_format}" \
  -o "%{buildroot}%{efidir}/%{efi_image}" \
  -p "/EFI/BOOT" \
  --pubkey empty.pubkey \
  --sbat sbat.csv \
  efi_gop ${MODS[@]} ${VERIFY_MODS[@]}
popd

%files
%license COPYING COPYING.unicode
%{_cross_attribution_file}
%if "%{_cross_arch}" == "x86_64"
%dir %{biosdir}
%{biosdir}/boot.img
%{biosdir}/core.img
%endif
%dir %{efidir}
%{efidir}/%{efi_image}
%{_cross_sbindir}/grub-bios-setup
%exclude %{_cross_bashdir}
%exclude %{_cross_infodir}
%exclude %{_cross_libexecdir}
%exclude %{_cross_localedir}
%exclude %{_cross_sysconfdir}
%exclude %{_cross_unitdir}

%files modules
%dir %{_cross_libdir}/grub
%{_cross_libdir}/grub/*

%files tools
%{_cross_bindir}/grub-editenv
%{_cross_bindir}/grub-file
%{_cross_bindir}/grub-fstest
%{_cross_bindir}/grub-glue-efi
%{_cross_bindir}/grub-kbdcomp
%{_cross_bindir}/grub-menulst2cfg
%{_cross_bindir}/grub-mkimage
%{_cross_bindir}/grub-mklayout
%{_cross_bindir}/grub-mknetdir
%{_cross_bindir}/grub-mkpasswd-pbkdf2
%{_cross_bindir}/grub-mkrelpath
%{_cross_bindir}/grub-mkrescue
%{_cross_bindir}/grub-mkstandalone
%{_cross_bindir}/grub-render-label
%{_cross_bindir}/grub-script-check
%{_cross_bindir}/grub-syslinux2cfg
%{_cross_sbindir}/grub-get-kernel-settings
%{_cross_sbindir}/grub-install
%{_cross_sbindir}/grub-macbless
%{_cross_sbindir}/grub-mkconfig
%{_cross_sbindir}/grub-ofpathname
%{_cross_sbindir}/grub-probe
%{_cross_sbindir}/grub-reboot
%{_cross_sbindir}/grub-set-bootflag
%{_cross_sbindir}/grub-set-default
%{_cross_sbindir}/grub-set-password
%{_cross_sbindir}/grub-sparc64-setup
%{_cross_sbindir}/grub-switch-to-blscfg

%dir %{_cross_datadir}/grub
%{_cross_datadir}/grub/grub-mkconfig_lib

%changelog
