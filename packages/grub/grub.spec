%global debug_package %{nil}

Name: %{_cross_os}grub
Version: 2.02
Release: 1%{?dist}
Summary: Bootloader with support for Linux and more
License: GPLv3+
URL: https://www.gnu.org/software/grub/
Source0: https://ftp.gnu.org/gnu/grub/grub-%{version}.tar.xz
Source1: core.cfg
Patch1: 0001-x86-64-Treat-R_X86_64_PLT32-as-R_X86_64_PC32.patch
Patch2: gpt.patch

BuildRequires: automake
BuildRequires: bison
BuildRequires: flex
BuildRequires: gcc-%{_cross_target}
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
  biosdisk configfile ext2 gptprio linux normal part_gpt search_fs_uuid

install -m 0644 ./grub-core/boot.img \
  %{buildroot}%{_cross_grubdir}/boot.img

%files
%dir %{_cross_grubdir}
%{_cross_grubdir}/boot.img
%{_cross_grubdir}/%{_cross_grub_image}
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
%{_cross_sbindir}/grub-bios-setup
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
