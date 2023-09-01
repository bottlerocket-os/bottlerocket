%global majorminor 2.39
%global version %{majorminor}.2

Name: %{_cross_os}util-linux
Version: %{version}
Release: 1%{?dist}
Summary: A collection of basic system utilities
License: BSD-3-Clause AND BSD-4-Clause-UC AND GPL-1.0-or-later AND GPL-2.0-only AND GPL-2.0-or-later AND GPL-3.0-or-later AND LGPL-2.0-or-later AND LGPL-2.1-or-later AND MIT
URL: http://en.wikipedia.org/wiki/Util-linux
Source0: https://www.kernel.org/pub/linux/utils/util-linux/v%{majorminor}/util-linux-%{version}.tar.xz

BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}libacl-devel
BuildRequires: %{_cross_os}libncurses-devel
BuildRequires: %{_cross_os}libselinux-devel
BuildRequires: %{_cross_os}libxcrypt-devel
Requires: %{_cross_os}libacl
Requires: %{_cross_os}libblkid
Requires: %{_cross_os}libncurses
Requires: %{_cross_os}libmount
Requires: %{_cross_os}libselinux
Requires: %{_cross_os}libsmartcols
Requires: %{_cross_os}libuuid
Requires: %{_cross_os}libxcrypt

%description
%{summary}.

%package -n %{_cross_os}libblkid
Summary: Block device ID library
License: LGPL-2.1-or-later

%description -n %{_cross_os}libblkid
%{summary}.

%package -n %{_cross_os}libblkid-devel
Summary: Files for development using the block device ID library
License: LGPL-2.1-or-later
Requires: %{_cross_os}libblkid

%description -n %{_cross_os}libblkid-devel
%{summary}.

%package -n %{_cross_os}libfdisk
Summary: Partition table library
License: LGPL-2.1-or-later

%description -n %{_cross_os}libfdisk
%{summary}.

%package -n %{_cross_os}libfdisk-devel
Summary: Files for development using the partition table library
License: LGPL-2.1-or-later
Requires: %{_cross_os}libfdisk

%description -n %{_cross_os}libfdisk-devel
%{summary}.

%package -n %{_cross_os}libmount
Summary: Device mounting library
License: LGPL-2.1-or-later
Requires: %{_cross_os}libblkid
Requires: %{_cross_os}libselinux

%description -n %{_cross_os}libmount
%{summary}.

%package -n %{_cross_os}libmount-devel
Summary: Files for development using the device mounting library
License: LGPL-2.1-or-later
Requires: %{_cross_os}libblkid-devel
Requires: %{_cross_os}libmount

%description -n %{_cross_os}libmount-devel
%{summary}.

%package -n %{_cross_os}libsmartcols
Summary: Formatting library for ls-like programs
License: LGPL-2.1-or-later

%description -n %{_cross_os}libsmartcols
%{summary}.

%package -n %{_cross_os}libsmartcols-devel
Summary: Files for development using the formatting library for ls-like programs
License: LGPL-2.1-or-later
Requires: %{_cross_os}libsmartcols

%description -n %{_cross_os}libsmartcols-devel
%{summary}.

%package -n %{_cross_os}libuuid
Summary: Universally unique ID library
License: BSD-3-Clause

%description -n %{_cross_os}libuuid
%{summary}.

%package -n %{_cross_os}libuuid-devel
Summary: Files for development using the universally unique ID library
License: BSD-3-Clause
Requires: %{_cross_os}libuuid

%description -n %{_cross_os}libuuid-devel
%{summary}.

%prep
%autosetup -n util-linux-%{version} -p1

cp Documentation/licenses/COPYING.* .

%build

%cross_configure \
  --disable-makeinstall-chown \
  --disable-nls \
  --disable-rpath \
  --enable-all-programs \
  --enable-libblkid \
  --enable-libfdisk \
  --enable-libmount \
  --enable-libsmartcols \
  --enable-libuuid \
  --enable-usrdir-path \
  --with-selinux \
  --without-audit \
  --without-python \
  --without-readline \
  --without-systemd \
  --without-udev \
  --without-utempter \

sed -i 's|^hardcode_libdir_flag_spec=.*|hardcode_libdir_flag_spec=""|g' libtool
sed -i 's|^runpath_var=LD_RUN_PATH|runpath_var=DIE_RPATH_DIE|g' libtool

%make_build

%install
%make_install

# add attribution.txt files for lib subpackages that need them, since the
# default macro only generates attribution.txt for the main package
for lib in lib{blkid,fdisk,mount,smartcols,uuid}; do
    mkdir -p %{buildroot}%{_cross_licensedir}/${lib}
    echo "${lib} - %{url}" >> %{buildroot}%{_cross_licensedir}/${lib}/attribution.txt
done

# most lib subpackages are LGPL-2.1-or-later
for lib in lib{blkid,fdisk,mount,smartcols}; do
    echo "SPDX-License-Identifier: LGPL-2.1-or-later" \
      | tee -a %{buildroot}%{_cross_licensedir}/${lib}/attribution.txt >/dev/null
    cp -a COPYING.LGPL-2.1-or-later %{buildroot}%{_cross_licensedir}/${lib}
done

# libuuid is BSD-3-Clause
for lib in libuuid; do
    echo "SPDX-License-Identifier: BSD-3-Clause" \
      | tee -a %{buildroot}%{_cross_licensedir}/${lib}/attribution.txt >/dev/null
    cp -a COPYING.BSD-3-Clause %{buildroot}%{_cross_licensedir}/${lib}
done

%files
%license COPYING.BSD-3-Clause COPYING.BSD-4-Clause-UC COPYING.GPL-2.0-or-later COPYING.LGPL-2.1-or-later
%{_cross_attribution_file}
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
%{_cross_bindir}/irqtop
%{_cross_bindir}/kill
%{_cross_bindir}/logger
%{_cross_bindir}/lsblk
%{_cross_bindir}/lscpu
%{_cross_bindir}/lsipc
%{_cross_bindir}/lsirq
%{_cross_bindir}/lsfd
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
%{_cross_bindir}/uclampset
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
%exclude %{_cross_bindir}/hardlink
%exclude %{_cross_bindir}/hexdump
%exclude %{_cross_bindir}/isosize
%exclude %{_cross_bindir}/last
%exclude %{_cross_bindir}/lastb
%exclude %{_cross_bindir}/line
%exclude %{_cross_bindir}/linux32
%exclude %{_cross_bindir}/linux64
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
%exclude %{_cross_bindir}/scriptlive
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
%exclude %{_cross_bindir}/fadvise
%exclude %{_cross_bindir}/pipesz
%exclude %{_cross_bindir}/waitpid
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
%exclude %{_cross_sbindir}/cfdisk
%exclude %{_cross_sbindir}/ctrlaltdel
%exclude %{_cross_sbindir}/fdformat
%exclude %{_cross_sbindir}/fdisk
%exclude %{_cross_sbindir}/fsck.minix
%exclude %{_cross_sbindir}/ldattach
%exclude %{_cross_sbindir}/mkfs.bfs
%exclude %{_cross_sbindir}/mkfs.minix
%exclude %{_cross_sbindir}/mkswap
%exclude %{_cross_sbindir}/raw
%exclude %{_cross_sbindir}/readprofile
%exclude %{_cross_sbindir}/rfkill
%exclude %{_cross_sbindir}/rtcwake
%exclude %{_cross_sbindir}/sfdisk
%exclude %{_cross_sbindir}/sulogin
%exclude %{_cross_sbindir}/swaplabel
%exclude %{_cross_sbindir}/swapoff
%exclude %{_cross_sbindir}/swapon
%exclude %{_cross_sbindir}/tunelp
%exclude %{_cross_sbindir}/uuidd
%exclude %{_cross_sbindir}/vigr
%exclude %{_cross_sbindir}/vipw
%exclude %{_cross_sbindir}/zramctl
%exclude %{_cross_sbindir}/blkpr

%exclude %{_cross_bashdir}
%exclude %{_cross_docdir}
%exclude %{_cross_mandir}

%files -n %{_cross_os}libblkid
%license %{_cross_licensedir}/libblkid/COPYING.LGPL-2.1-or-later
%license %{_cross_licensedir}/libblkid/attribution.txt
%{_cross_libdir}/libblkid.so.*

%files -n %{_cross_os}libblkid-devel
%{_cross_libdir}/libblkid.a
%{_cross_libdir}/libblkid.so
%dir %{_cross_includedir}/blkid
%{_cross_includedir}/blkid/blkid.h
%{_cross_pkgconfigdir}/blkid.pc

%files -n %{_cross_os}libfdisk
%license %{_cross_licensedir}/libfdisk/COPYING.LGPL-2.1-or-later
%license %{_cross_licensedir}/libfdisk/attribution.txt
%{_cross_libdir}/libfdisk.so.*

%files -n %{_cross_os}libfdisk-devel
%{_cross_libdir}/libfdisk.a
%{_cross_libdir}/libfdisk.so
%dir %{_cross_includedir}/libfdisk
%{_cross_includedir}/libfdisk/libfdisk.h
%{_cross_pkgconfigdir}/fdisk.pc

%files -n %{_cross_os}libmount
%license %{_cross_licensedir}/libmount/COPYING.LGPL-2.1-or-later
%license %{_cross_licensedir}/libmount/attribution.txt
%{_cross_libdir}/libmount.so.*

%files -n %{_cross_os}libmount-devel
%{_cross_libdir}/libmount.a
%{_cross_libdir}/libmount.so
%dir %{_cross_includedir}/libmount
%{_cross_includedir}/libmount/libmount.h
%{_cross_pkgconfigdir}/mount.pc

%files -n %{_cross_os}libsmartcols
%license %{_cross_licensedir}/libsmartcols/COPYING.LGPL-2.1-or-later
%license %{_cross_licensedir}/libsmartcols/attribution.txt
%{_cross_libdir}/libsmartcols.so.*

%files -n %{_cross_os}libsmartcols-devel
%{_cross_libdir}/libsmartcols.a
%{_cross_libdir}/libsmartcols.so
%dir %{_cross_includedir}/libsmartcols
%{_cross_includedir}/libsmartcols/libsmartcols.h
%{_cross_pkgconfigdir}/smartcols.pc

%files -n %{_cross_os}libuuid
%license %{_cross_licensedir}/libuuid/COPYING.BSD-3-Clause
%license %{_cross_licensedir}/libuuid/attribution.txt
%{_cross_libdir}/libuuid.so.*

%files -n %{_cross_os}libuuid-devel
%{_cross_libdir}/libuuid.a
%{_cross_libdir}/libuuid.so
%dir %{_cross_includedir}/uuid
%{_cross_includedir}/uuid/uuid.h
%{_cross_pkgconfigdir}/uuid.pc

%changelog
