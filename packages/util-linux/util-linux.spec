Name: %{_cross_os}util-linux
Version: 2.33.1
Release: 1%{?dist}
Summary: A collection of basic system utilities
License: GPLv2 and GPLv2+ and LGPLv2+ and BSD with advertising and Public Domain
URL: http://en.wikipedia.org/wiki/Util-linux
Source0: https://www.kernel.org/pub/linux/utils/util-linux/v2.33/util-linux-%{version}.tar.xz
BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}libacl-devel
BuildRequires: %{_cross_os}libxcrypt-devel
BuildRequires: %{_cross_os}ncurses-devel
Requires: %{_cross_os}libacl
Requires: %{_cross_os}libxcrypt
Requires: %{_cross_os}ncurses
Requires: %{_cross_os}libblkid
Requires: %{_cross_os}libmount
Requires: %{_cross_os}libsmartcols
Requires: %{_cross_os}libuuid

%description
%{summary}.

%package -n %{_cross_os}libblkid
Summary: Block device ID library
License: LGPLv2+

%description -n %{_cross_os}libblkid
%{summary}.

%package -n %{_cross_os}libblkid-devel
Summary: Files for development using the block device ID library
License: LGPLv2+
Requires: %{_cross_os}libblkid

%description -n %{_cross_os}libblkid-devel
%{summary}.

%package -n %{_cross_os}libmount
Summary: Device mounting library
License: LGPLv2+

%description -n %{_cross_os}libmount
%{summary}.

%package -n %{_cross_os}libmount-devel
Summary: Files for development using the device mounting library
License: LGPLv2+
Requires: %{_cross_os}libmount

%description -n %{_cross_os}libmount-devel
%{summary}.

%package -n %{_cross_os}libsmartcols
Summary: Formatting library for ls-like programs
License: LGPLv2+

%description -n %{_cross_os}libsmartcols
%{summary}.

%package -n %{_cross_os}libsmartcols-devel
Summary: Files for development using the formatting library for ls-like programs
License: LGPLv2+
Requires: %{_cross_os}libsmartcols

%description -n %{_cross_os}libsmartcols-devel
%{summary}.

%package -n %{_cross_os}libuuid
Summary: Universally unique ID library
License: BSD

%description -n %{_cross_os}libuuid
%{summary}.

%package -n %{_cross_os}libuuid-devel
Summary: Files for development using the universally unique ID library
License: BSD
Requires: %{_cross_os}libuuid

%description -n %{_cross_os}libuuid-devel
%{summary}.

%prep
%autosetup -n util-linux-%{version} -p1

%build
%cross_configure \
  --disable-libfdisk \
  --disable-makeinstall-chown \
  --disable-nls \
  --disable-rpath \
  --enable-all-programs \
  --enable-libblkid \
  --enable-libmount \
  --enable-libsmartcols \
  --enable-libuuid \
  --enable-usrdir-path \
  --without-audit \
  --without-python \
  --without-readline \
  --without-selinux \
  --without-systemd \
  --without-udev \
  --without-utempter \

sed -i 's|^hardcode_libdir_flag_spec=.*|hardcode_libdir_flag_spec=""|g' libtool
sed -i 's|^runpath_var=LD_RUN_PATH|runpath_var=DIE_RPATH_DIE|g' libtool

%make_build

%install
%make_install

%files
%{_cross_bindir}/chmem
%{_cross_bindir}/choom
%{_cross_bindir}/chrt
%{_cross_bindir}/dmesg
%{_cross_bindir}/fallocate
%{_cross_bindir}/findmnt
%{_cross_bindir}/flock
%{_cross_bindir}/ionice
%{_cross_bindir}/ipcmk
%{_cross_bindir}/ipcrm
%{_cross_bindir}/ipcs
%{_cross_bindir}/kill
%{_cross_bindir}/lsblk
%{_cross_bindir}/lscpu
%{_cross_bindir}/lsipc
%{_cross_bindir}/lslocks
%{_cross_bindir}/lsmem
%{_cross_bindir}/lsns
%{_cross_bindir}/more
%{_cross_bindir}/mount
%{_cross_bindir}/newgrp
%{_cross_bindir}/nsenter
%{_cross_bindir}/prlimit
%{_cross_bindir}/renice
%{_cross_bindir}/setsid
%{_cross_bindir}/taskset
%{_cross_bindir}/umount
%{_cross_bindir}/unshare
%{_cross_bindir}/uuidgen
%{_cross_bindir}/uuidparse
%exclude %{_cross_bindir}/cal
%exclude %{_cross_bindir}/col
%exclude %{_cross_bindir}/colcrt
%exclude %{_cross_bindir}/colrm
%exclude %{_cross_bindir}/column
%exclude %{_cross_bindir}/eject
%exclude %{_cross_bindir}/fincore
%exclude %{_cross_bindir}/getopt
%exclude %{_cross_bindir}/hexdump
%exclude %{_cross_bindir}/isosize
%exclude %{_cross_bindir}/last
%exclude %{_cross_bindir}/lastb
%exclude %{_cross_bindir}/line
%exclude %{_cross_bindir}/linux32
%exclude %{_cross_bindir}/linux64
%exclude %{_cross_bindir}/logger
%exclude %{_cross_bindir}/look
%exclude %{_cross_bindir}/lslogins
%exclude %{_cross_bindir}/mcookie
%exclude %{_cross_bindir}/mesg
%exclude %{_cross_bindir}/mountpoint
%exclude %{_cross_bindir}/namei
%exclude %{_cross_bindir}/pg
%exclude %{_cross_bindir}/rename
%exclude %{_cross_bindir}/rev
%exclude %{_cross_bindir}/script
%exclude %{_cross_bindir}/scriptreplay
%exclude %{_cross_bindir}/setarch
%exclude %{_cross_bindir}/setterm
%exclude %{_cross_bindir}/ul
%exclude %{_cross_bindir}/uname26
%exclude %{_cross_bindir}/utmpdump
%exclude %{_cross_bindir}/wall
%exclude %{_cross_bindir}/wdctl
%exclude %{_cross_bindir}/whereis
%exclude %{_cross_bindir}/write
%if "%{_cross_arch}" == "x86_64"
%exclude %{_cross_bindir}/i386
%exclude %{_cross_bindir}/x86_64
%endif

%{_cross_sbindir}/addpart
%{_cross_sbindir}/agetty
%{_cross_sbindir}/blkdiscard
%{_cross_sbindir}/blkid
%{_cross_sbindir}/blkzone
%{_cross_sbindir}/blockdev
%{_cross_sbindir}/chcpu
%{_cross_sbindir}/delpart
%{_cross_sbindir}/findfs
%{_cross_sbindir}/fsck
%{_cross_sbindir}/fsfreeze
%{_cross_sbindir}/fstrim
%{_cross_sbindir}/losetup
%{_cross_sbindir}/mkfs
%{_cross_sbindir}/nologin
%{_cross_sbindir}/partx
%{_cross_sbindir}/pivot_root
%{_cross_sbindir}/resizepart
%{_cross_sbindir}/switch_root
%{_cross_sbindir}/wipefs
%exclude %{_cross_sbindir}/hwclock
%exclude %{_cross_sbindir}/ctrlaltdel
%exclude %{_cross_sbindir}/fdformat
%exclude %{_cross_sbindir}/fsck.minix
%exclude %{_cross_sbindir}/ldattach
%exclude %{_cross_sbindir}/mkfs.bfs
%exclude %{_cross_sbindir}/mkfs.minix
%exclude %{_cross_sbindir}/mkswap
%exclude %{_cross_sbindir}/raw
%exclude %{_cross_sbindir}/readprofile
%exclude %{_cross_sbindir}/rfkill
%exclude %{_cross_sbindir}/rtcwake
%exclude %{_cross_sbindir}/sulogin
%exclude %{_cross_sbindir}/swaplabel
%exclude %{_cross_sbindir}/swapoff
%exclude %{_cross_sbindir}/swapon
%exclude %{_cross_sbindir}/tunelp
%exclude %{_cross_sbindir}/uuidd
%exclude %{_cross_sbindir}/vigr
%exclude %{_cross_sbindir}/vipw
%exclude %{_cross_sbindir}/zramctl

%exclude %{_cross_bashdir}
%exclude %{_cross_docdir}
%exclude %{_cross_mandir}

%files -n %{_cross_os}libblkid
%{_cross_libdir}/libblkid.so.*

%files -n %{_cross_os}libblkid-devel
%{_cross_libdir}/libblkid.a
%{_cross_libdir}/libblkid.so
%dir %{_cross_includedir}/blkid
%{_cross_includedir}/blkid/blkid.h
%{_cross_pkgconfigdir}/blkid.pc
%exclude %{_cross_libdir}/libblkid.la

%files -n %{_cross_os}libmount
%{_cross_libdir}/libmount.so.*

%files -n %{_cross_os}libmount-devel
%{_cross_libdir}/libmount.a
%{_cross_libdir}/libmount.so
%dir %{_cross_includedir}/libmount
%{_cross_includedir}/libmount/libmount.h
%{_cross_pkgconfigdir}/mount.pc
%exclude %{_cross_libdir}/libmount.la

%files -n %{_cross_os}libsmartcols
%{_cross_libdir}/libsmartcols.so.*

%files -n %{_cross_os}libsmartcols-devel
%{_cross_libdir}/libsmartcols.a
%{_cross_libdir}/libsmartcols.so
%dir %{_cross_includedir}/libsmartcols
%{_cross_includedir}/libsmartcols/libsmartcols.h
%{_cross_pkgconfigdir}/smartcols.pc
%exclude %{_cross_libdir}/libsmartcols.la

%files -n %{_cross_os}libuuid
%{_cross_libdir}/libuuid.so.*

%files -n %{_cross_os}libuuid-devel
%{_cross_libdir}/libuuid.a
%{_cross_libdir}/libuuid.so
%dir %{_cross_includedir}/uuid
%{_cross_includedir}/uuid/uuid.h
%{_cross_pkgconfigdir}/uuid.pc
%exclude %{_cross_libdir}/libuuid.la

%changelog
