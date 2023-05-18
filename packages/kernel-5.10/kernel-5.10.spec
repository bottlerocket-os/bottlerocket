%global debug_package %{nil}

Name: %{_cross_os}kernel-5.10
Version: 5.10.178
Release: 1%{?dist}
Summary: The Linux kernel
License: GPL-2.0 WITH Linux-syscall-note
URL: https://www.kernel.org/
# Use latest-srpm-url.sh to get this.
Source0: https://cdn.amazonlinux.com/blobstore/13be720c0258208a986213f02d549940509f5125eac626729bc5dd3612bef2f8/kernel-5.10.178-162.673.amzn2.src.rpm
Source100: config-bottlerocket
Source101: config-bottlerocket-aws
Source102: config-bottlerocket-metal
Source103: config-bottlerocket-vmware

# Help out-of-tree module builds run `make prepare` automatically.
Patch1001: 1001-Makefile-add-prepare-target-for-external-modules.patch
# Enable INITRAMFS_FORCE config option for our use case.
Patch1002: 1002-initramfs-unlink-INITRAMFS_FORCE-from-CMDLINE_-EXTEN.patch

# Add zstd support for compressed kernel modules
Patch2000: 2000-kbuild-move-module-strip-compression-code-into-scrip.patch
Patch2001: 2001-kbuild-add-support-for-zstd-compressed-modules.patch

# Backport from v5.10.180 upstream, drop when Amazon Linux base is v5.10.180 or later
Patch5001: 5001-netfilter-nf_tables-deactivate-anonymous-set-from-pr.patch

BuildRequires: bc
BuildRequires: elfutils-devel
BuildRequires: hostname
BuildRequires: kmod
BuildRequires: openssl-devel

# CPU microcode updates are included as "extra firmware" so the files don't
# need to be installed on the root filesystem. However, we want the license and
# attribution files to be available in the usual place.
%if "%{_cross_arch}" == "x86_64"
BuildRequires: %{_cross_os}microcode
Requires: %{_cross_os}microcode-licenses
%endif

# Pull in expected modules and development files.
Requires: %{name}-modules = %{version}-%{release}
Requires: %{name}-devel = %{version}-%{release}

%global kernel_sourcedir %{_cross_usrsrc}/kernels
%global kernel_libdir %{_cross_libdir}/modules/%{version}

%description
%{summary}.

%package devel
Summary: Configured Linux kernel source for module building

%description devel
%{summary}.

%package archive
Summary: Archived Linux kernel source for module building

%description archive
%{summary}.

%package modules
Summary: Modules for the Linux kernel

%description modules
%{summary}.

%package headers
Summary: Header files for the Linux kernel for use by glibc

%description headers
%{summary}.

%prep
rpm2cpio %{SOURCE0} | cpio -iu linux-%{version}.tar config-%{_cross_arch} "*.patch"
tar -xof linux-%{version}.tar; rm linux-%{version}.tar
%setup -TDn linux-%{version}
# Patches from the Source0 SRPM
for patch in ../*.patch; do
    patch -p1 <"$patch"
done
# Patches listed in this spec (Patch0001...)
%autopatch -p1

%if "%{_cross_arch}" == "x86_64"
microcode="$(find %{_cross_libdir}/firmware -type f -path '*/*-ucode/*' -printf '%%P ')"
cat <<EOF > ../config-microcode
CONFIG_EXTRA_FIRMWARE="${microcode}"
CONFIG_EXTRA_FIRMWARE_DIR="%{_cross_libdir}/firmware"
EOF
%endif

export ARCH="%{_cross_karch}"
export CROSS_COMPILE="%{_cross_target}-"

KCONFIG_CONFIG="arch/%{_cross_karch}/configs/%{_cross_vendor}_defconfig" \
scripts/kconfig/merge_config.sh \
  ../config-%{_cross_arch} \
%if "%{_cross_arch}" == "x86_64"
  ../config-microcode \
%endif
  %{SOURCE100} \
  %{_sourcedir}/config-bottlerocket-%{_cross_variant_platform}

rm -f ../config-* ../*.patch

%global kmake \
make -s\\\
  ARCH="%{_cross_karch}"\\\
  CROSS_COMPILE="%{_cross_target}-"\\\
  INSTALL_HDR_PATH="%{buildroot}%{_cross_prefix}"\\\
  INSTALL_MOD_PATH="%{buildroot}%{_cross_prefix}"\\\
  INSTALL_MOD_STRIP=1\\\
%{nil}

%build
%kmake mrproper
%kmake %{_cross_vendor}_defconfig
%kmake %{?_smp_mflags} %{_cross_kimage}
%kmake %{?_smp_mflags} modules

%install
%kmake %{?_smp_mflags} headers_install
%kmake %{?_smp_mflags} modules_install

install -d %{buildroot}/boot
install -T -m 0755 arch/%{_cross_karch}/boot/%{_cross_kimage} %{buildroot}/boot/vmlinuz
install -m 0644 .config %{buildroot}/boot/config

find %{buildroot}%{_cross_prefix} \
   \( -name .install -o -name .check -o \
      -name ..install.cmd -o -name ..check.cmd \) -delete

# For out-of-tree kmod builds, we need to support the following targets:
#   make scripts -> make prepare -> make modules
#
# This requires enough of the kernel tree to build host programs under the
# "scripts" and "tools" directories.

# Any existing ELF objects will not work properly if we're cross-compiling for
# a different architecture, so get rid of them to avoid confusing errors.
find arch scripts tools -type f -executable \
  -exec sh -c "head -c4 {} | grep -q ELF && rm {}" \;

# We don't need to include these files.
find -type f \( -name \*.cmd -o -name \*.gitignore \) -delete

# Avoid an OpenSSL dependency by stubbing out options for module signing and
# trusted keyrings, so `sign-file` and `extract-cert` won't be built. External
# kernel modules do not have access to the keys they would need to make use of
# these tools.
sed -i \
  -e 's,$(CONFIG_MODULE_SIG_FORMAT),n,g' \
  -e 's,$(CONFIG_SYSTEM_TRUSTED_KEYRING),n,g' \
  scripts/Makefile

# Restrict permissions on System.map.
chmod 600 System.map

(
  find * \
    -type f \
    \( -name Build\* -o -name Kbuild\* -o -name Kconfig\* -o -name Makefile\* \) \
    -print

  find arch/%{_cross_karch}/ \
    -type f \
    \( -name module.lds -o -name vmlinux.lds.S -o -name Platform -o -name \*.tbl \) \
    -print

  find arch/%{_cross_karch}/{include,lib}/ -type f ! -name \*.o ! -name \*.o.d -print
  echo arch/%{_cross_karch}/kernel/asm-offsets.s
  echo lib/vdso/gettimeofday.c

  for d in \
    arch/%{_cross_karch}/tools \
    arch/%{_cross_karch}/kernel/vdso ; do
    [ -d "${d}" ] && find "${d}/" -type f -print
  done

  find include -type f -print
  find scripts -type f ! -name \*.l ! -name \*.y ! -name \*.o -print

  find tools/{arch/%{_cross_karch},include,objtool,scripts}/ -type f ! -name \*.o -print
  echo tools/build/fixdep.c
  find tools/lib/subcmd -type f -print
  find tools/lib/{ctype,hweight,rbtree,string,str_error_r}.c

  echo kernel/bounds.c
  echo kernel/time/timeconst.bc
  echo security/selinux/include/classmap.h
  echo security/selinux/include/initial_sid_to_string.h
  echo security/selinux/include/policycap.h
  echo security/selinux/include/policycap_names.h

  echo .config
  echo Module.symvers
  echo System.map
) | sort -u > kernel_devel_files

# Create squashfs of kernel-devel files (ie. /usr/src/kernels/<version>).
#
# -no-exports:
# The filesystem does not need to be exported via NFS.
#
# -all-root:
# Make all files owned by root rather than the build user.
#
# -comp zstd:
# zstd offers compression ratios like xz and decompression speeds like lz4.
SQUASHFS_OPTS="-no-exports -all-root -comp zstd"
mkdir -p src_squashfs/%{version}
tar c -T kernel_devel_files | tar x -C src_squashfs/%{version}
mksquashfs src_squashfs kernel-devel.squashfs ${SQUASHFS_OPTS}

# Create a tarball of the same files, for use outside the running system.
# In theory we could extract these files with `unsquashfs`, but we do not want
# to require it to be installed on the build host, and it errors out when run
# inside Docker unless the limit for open files is lowered.
tar cf kernel-devel.tar src_squashfs/%{version} --transform='s|src_squashfs/%{version}|kernel-devel|'
xz -T0 kernel-devel.tar

install -D kernel-devel.squashfs %{buildroot}%{_cross_datadir}/bottlerocket/kernel-devel.squashfs
install -D kernel-devel.tar.xz %{buildroot}%{_cross_datadir}/bottlerocket/kernel-devel.tar.xz
install -d %{buildroot}%{kernel_sourcedir}

# Replace the incorrect links from modules_install. These will be bound
# into a host container (and unused in the host) so they must not point
# to %{_cross_usrsrc} (eg. /x86_64-bottlerocket-linux-gnu/sys-root/...)
rm -f %{buildroot}%{kernel_libdir}/build %{buildroot}%{kernel_libdir}/source
ln -sf %{_usrsrc}/kernels/%{version} %{buildroot}%{kernel_libdir}/build
ln -sf %{_usrsrc}/kernels/%{version} %{buildroot}%{kernel_libdir}/source

%files
%license COPYING LICENSES/preferred/GPL-2.0 LICENSES/exceptions/Linux-syscall-note
%{_cross_attribution_file}
/boot/vmlinuz
/boot/config

%files modules
%dir %{_cross_libdir}/modules
%{_cross_libdir}/modules/*

%files headers
%dir %{_cross_includedir}/asm
%dir %{_cross_includedir}/asm-generic
%dir %{_cross_includedir}/drm
%dir %{_cross_includedir}/linux
%dir %{_cross_includedir}/misc
%dir %{_cross_includedir}/mtd
%dir %{_cross_includedir}/rdma
%dir %{_cross_includedir}/scsi
%dir %{_cross_includedir}/sound
%dir %{_cross_includedir}/video
%dir %{_cross_includedir}/xen
%{_cross_includedir}/asm/*
%{_cross_includedir}/asm-generic/*
%{_cross_includedir}/drm/*
%{_cross_includedir}/linux/*
%{_cross_includedir}/misc/*
%{_cross_includedir}/mtd/*
%{_cross_includedir}/rdma/*
%{_cross_includedir}/scsi/*
%{_cross_includedir}/sound/*
%{_cross_includedir}/video/*
%{_cross_includedir}/xen/*

%files devel
%dir %{kernel_sourcedir}
%{_cross_datadir}/bottlerocket/kernel-devel.squashfs

%files archive
%{_cross_datadir}/bottlerocket/kernel-devel.tar.xz

%changelog
