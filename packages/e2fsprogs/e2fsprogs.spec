Name: %{_cross_os}e2fsprogs
Version: 1.47.0
Release: 1%{?dist}
Summary: Tools for managing ext2, ext3, and ext4 file systems
License: GPL-2.0-only AND LGPL-2.0-only AND LGPL-2.0-or-later AND BSD-3-Clause
URL: http://e2fsprogs.sourceforge.net/
Source0: https://mirrors.edge.kernel.org/pub/linux/kernel/people/tytso/e2fsprogs/%{version}/e2fsprogs-%{version}.tar.xz
Source10: mke2fs.conf
Source11: e2fsprogs-tmpfiles.conf

BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}libuuid-devel
BuildRequires: %{_cross_os}libblkid-devel
Requires: %{_cross_os}e2fsprogs-libs

%description
%{summary}.

%package libs
Summary: Libraries for ext2, ext3, and ext4 file systems

%description libs
%{summary}.

%package devel
Summary: Files for development using the libraries for ext2, ext3, and ext4 file systems
Requires: %{_cross_os}e2fsprogs-libs

%description devel
%{summary}.

%prep
%autosetup -n e2fsprogs-%{version} -p1

%build
%cross_configure \
  CFLAGS="${CFLAGS} -fno-strict-aliasing" \
  --enable-elf-shlibs \
  --enable-symlink-install \
  --enable-relative-symlinks \
  --enable-resizer \
  --disable-backtrace \
  --disable-debugfs \
  --disable-defrag \
  --disable-e2initrd-helper \
  --disable-fsck \
  --disable-fuse2fs \
  --disable-imager \
  --disable-libblkid \
  --disable-libuuid \
  --disable-nls \
  --disable-rpath \
  --disable-tdb \
  --disable-uuidd \
  --with-crond-dir=no \
  --with-systemd-unit-dir=no \
  --with-udev-rules-dir=no \

%make_build

%install
%make_install install-libs \
  root_sbindir=%{_cross_sbindir} \
  root_libdir=%{_cross_libdir}

chmod 644 %{buildroot}%{_cross_libdir}/*.a

install -d %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}
install -p -m 0644 %{S:10} %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}

install -d %{buildroot}%{_cross_tmpfilesdir}
install -p -m 0644 %{S:11} %{buildroot}%{_cross_tmpfilesdir}/e2fsprogs.conf

%files
%license debian/copyright
%{_cross_attribution_file}
%{_cross_sbindir}/badblocks
%{_cross_sbindir}/dumpe2fs
%{_cross_sbindir}/e2fsck
%{_cross_sbindir}/fsck.ext2
%{_cross_sbindir}/fsck.ext3
%{_cross_sbindir}/fsck.ext4
%{_cross_sbindir}/mke2fs
%{_cross_sbindir}/mkfs.ext2
%{_cross_sbindir}/mkfs.ext3
%{_cross_sbindir}/mkfs.ext4
%{_cross_sbindir}/resize2fs
%{_cross_sbindir}/tune2fs
%{_cross_factorydir}%{_cross_sysconfdir}/mke2fs.conf
%{_cross_tmpfilesdir}/e2fsprogs.conf

%exclude %{_cross_sbindir}/e2freefrag
%exclude %{_cross_sbindir}/e2label
%exclude %{_cross_sbindir}/e2mmpstatus
%exclude %{_cross_sbindir}/e2scrub
%exclude %{_cross_sbindir}/e2scrub_all
%exclude %{_cross_sbindir}/e2undo
%exclude %{_cross_sbindir}/e4crypt
%exclude %{_cross_sbindir}/filefrag
%exclude %{_cross_sbindir}/logsave
%exclude %{_cross_sbindir}/mklost+found

%exclude %{_cross_bindir}
%exclude %{_cross_mandir}
%exclude %{_cross_sysconfdir}
%exclude %{_cross_datadir}/et
%exclude %{_cross_datadir}/ss

%files libs
%{_cross_libdir}/*.so.*

%files devel
%{_cross_libdir}/*.a
%{_cross_libdir}/*.so
%{_cross_includedir}/*.h
%dir %{_cross_includedir}/e2p
%dir %{_cross_includedir}/et
%dir %{_cross_includedir}/ext2fs
%dir %{_cross_includedir}/ss
%{_cross_includedir}/e2p/*.h
%{_cross_includedir}/et/*.h
%{_cross_includedir}/ext2fs/*.h
%{_cross_includedir}/ss/*.h
%{_cross_pkgconfigdir}/*.pc

%changelog
