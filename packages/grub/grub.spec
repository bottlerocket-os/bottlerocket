%global debug_package %{nil}

Name: %{_cross_os}grub
Version: 2.04
Release: 1%{?dist}
Summary: Bootloader with support for Linux and more
License: GPL-3.0-or-later AND Unicode-DFS-2015
URL: https://www.gnu.org/software/grub/
Source0: https://ftp.gnu.org/gnu/grub/grub-%{version}.tar.xz
Source1: core.cfg
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
Patch0041: 0041-gptprio-Use-Thar-boot-partition-type-GUID.patch

BuildRequires: automake
BuildRequires: bison
BuildRequires: flex
BuildRequires: gettext-devel
BuildRequires: grub2-tools
BuildRequires: %{_cross_os}glibc-devel

%description
%{summary}.

%package modules
Summary: Modules for the bootloader with support for Linux and more
BuildArch: noarch

%description modules
%{summary}.

%package tools
Summary: Tools for the bootloader with support for Linux and more

%description tools
%{summary}.

%prep
%autosetup -n grub-%{version} -p1
cp unicode/COPYING COPYING.unicode

%global grub_cflags -pipe -fno-stack-protector -fno-strict-aliasing
%global grub_ldflags -static

%build
export \
  CPP="%{_cross_target}-gcc -E" \
  TARGET_CC="%{_cross_target}-gcc" \
  TARGET_CFLAGS="%{grub_cflags}" \
  TARGET_CPPFLAGS="%{grub_cflags}" \
  TARGET_LDFLAGS="%{grub_ldflags}" \
  TARGET_NM="%{_cross_target}-nm" \
  TARGET_OBJCOPY="%{_cross_target}-objcopy" \
  TARGET_STRIP="%{_cross_target}-strip" \
  PYTHON="python3" \

./autogen.sh
%cross_configure \
  CFLAGS="" \
  LDFLAGS="" \
  --target="%{_cross_grub_target}" \
  --with-platform="%{_cross_grub_platform}" \
  --disable-grub-mkfont \
  --enable-efiemu=no \
  --enable-device-mapper=no \
  --enable-libzfs=no \
  --disable-werror \

%make_build

%install
%make_install

mkdir -p %{buildroot}%{_cross_grubdir}

grub2-mkimage \
  -c %{SOURCE1} \
  -d ./grub-core/ \
  -O "%{_cross_grub_tuple}" \
  -o "%{buildroot}%{_cross_grubdir}/%{_cross_grub_image}" \
  -p "%{_cross_grub_prefix}" \
%if %{_cross_arch} == x86_64
  biosdisk \
%else
  efi_gop \
%endif
  configfile echo ext2 gptprio linux normal part_gpt reboot sleep

%if %{_cross_arch} == x86_64
install -m 0644 ./grub-core/boot.img \
  %{buildroot}%{_cross_grubdir}/boot.img
%endif

%files
%license COPYING COPYING.unicode
%{_cross_attribution_file}
%dir %{_cross_grubdir}
%if %{_cross_arch} == x86_64
%{_cross_grubdir}/boot.img
%endif
%{_cross_grubdir}/%{_cross_grub_image}
%{_cross_sbindir}/grub-bios-setup
%exclude %{_cross_infodir}
%exclude %{_cross_localedir}
%exclude %{_cross_sysconfdir}

%files modules
%dir %{_cross_libdir}/grub
%dir %{_cross_libdir}/grub/%{_cross_grub_tuple}
%{_cross_libdir}/grub/%{_cross_grub_tuple}/*

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
%{_cross_sbindir}/grub-install
%{_cross_sbindir}/grub-macbless
%{_cross_sbindir}/grub-mkconfig
%{_cross_sbindir}/grub-ofpathname
%{_cross_sbindir}/grub-probe
%{_cross_sbindir}/grub-reboot
%{_cross_sbindir}/grub-set-default
%{_cross_sbindir}/grub-sparc64-setup

%dir %{_cross_datadir}/grub
%{_cross_datadir}/grub/grub-mkconfig_lib

%changelog
