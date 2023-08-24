Name: %{_cross_os}xfsprogs
Version: 6.4.0
Release: 1%{?dist}
Summary: Utilities for managing the XFS filesystem
License: GPL-2.0-only AND LGPL-2.1-only
URL: https://xfs.wiki.kernel.org
Source0: http://kernel.org/pub/linux/utils/fs/xfs/xfsprogs/xfsprogs-%{version}.tar.xz
Patch1: 0001-libxfs-do-not-try-to-run-the-crc32selftest.patch

BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}libuuid-devel
BuildRequires: %{_cross_os}libinih-devel
BuildRequires: %{_cross_os}liburcu-devel
BuildRequires: %{_cross_os}libblkid-devel

Requires: %{_cross_os}liburcu
Requires: %{_cross_os}libinih

%description
%{summary}.

%package devel
Summary: XFS filesystem-specific headers
Requires: %{name}

%description devel
%{summary}.

%prep
%autosetup -n xfsprogs-%{version} -p1

%build
%cross_configure \
  --enable-blkid=yes \
  --enable-lto=no \
  --enable-editline=no \
  --enable-scrub=no

%make_build

%install
make DIST_ROOT=%{buildroot} install install-dev \
  PKG_ROOT_SBIN_DIR=%{_cross_sbindir} PKG_ROOT_LIB_DIR=%{_cross_libdir}

rm -f %{buildroot}/%{_cross_libdir}/*.{la,a}

%files
%license LICENSES/GPL-2.0 LICENSES/LGPL-2.1
%{_cross_attribution_file}
%{_cross_libdir}/*.so.*
%{_cross_sbindir}/mkfs.xfs
%{_cross_sbindir}/xfs_copy
%{_cross_sbindir}/xfs_db
%{_cross_sbindir}/xfs_estimate
%{_cross_sbindir}/xfs_fsr
%{_cross_sbindir}/xfs_growfs
%{_cross_sbindir}/xfs_io
%{_cross_sbindir}/xfs_logprint
%{_cross_sbindir}/xfs_mdrestore
%{_cross_sbindir}/xfs_quota
%{_cross_sbindir}/xfs_repair
%{_cross_sbindir}/xfs_rtcp
%{_cross_sbindir}/xfs_spaceman
%{_cross_datadir}/xfsprogs/mkfs/*.conf

# Exclude shell scripts
%exclude %{_cross_sbindir}/fsck.xfs
%exclude %{_cross_sbindir}/xfs_admin
%exclude %{_cross_sbindir}/xfs_bmap
%exclude %{_cross_sbindir}/xfs_freeze
%exclude %{_cross_sbindir}/xfs_info
%exclude %{_cross_sbindir}/xfs_metadump
%exclude %{_cross_sbindir}/xfs_mkfile
%exclude %{_cross_sbindir}/xfs_ncheck

%exclude %{_cross_mandir}
%exclude %{_cross_localedir}
%exclude %{_cross_docdir}


%files devel
%dir %{_cross_includedir}/xfs
%{_cross_includedir}/xfs/*.h
%{_cross_libdir}/*.so
