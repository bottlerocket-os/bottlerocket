%global debug_package %{nil}

Name: %{_cross_os}kernel-6.1
Version: 6.1.84
Release: 1%{?dist}
Summary: The Linux kernel
License: GPL-2.0 WITH Linux-syscall-note
URL: https://www.kernel.org/
# Use latest-srpm-url.sh to get this.
Source0: https://cdn.amazonlinux.com/al2023/blobstore/bdca6b79db0d3d5ad549b61951208fbf474daebe38ca619f8c706070dc252239/kernel-6.1.84-99.169.amzn2023.src.rpm
Source100: config-bottlerocket

# This list of FIPS modules is extracted from /etc/fipsmodules in the initramfs
# after placing AL2023 in FIPS mode.
Source200: check-fips-modules.drop-in.conf.in
Source201: fipsmodules-x86_64
Source202: fipsmodules-aarch64

# Help out-of-tree module builds run `make prepare` automatically.
Patch1001: 1001-Makefile-add-prepare-target-for-external-modules.patch
# Expose tools/* targets for out-of-tree module builds.
Patch1002: 1002-Revert-kbuild-hide-tools-build-targets-from-external.patch
# Enable INITRAMFS_FORCE config option for our use case.
Patch1003: 1003-initramfs-unlink-INITRAMFS_FORCE-from-CMDLINE_-EXTEN.patch
# Increase default of sysctl net.unix.max_dgram_qlen to 512.
Patch1004: 1004-af_unix-increase-default-max_dgram_qlen-to-512.patch
# Drop AL revert of upstream patch to minimize delta. The necessary dependency
# options for nvidia are instead included through DRM_SIMPLE
Patch1005: 1005-Revert-Revert-drm-fb_helper-improve-CONFIG_FB-depend.patch

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

# Pull in FIPS-related files if needed.
Requires: (%{name}-fips if %{_cross_os}image-feature(fips))

%global _cross_ksrcdir %{_cross_usrsrc}/kernels
%global _cross_kmoddir %{_cross_libdir}/modules/%{version}

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

%package fips
Summary: FIPS related configuration for the Linux kernel
Requires: (%{_cross_os}image-feature(fips) and %{name})
Conflicts: %{_cross_os}image-feature(no-fips)

%description fips
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
  %{SOURCE100}

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
install -d %{buildroot}%{_cross_ksrcdir}

# Replace the incorrect links from modules_install. These will be bound
# into a host container (and unused in the host) so they must not point
# to %{_cross_usrsrc} (eg. /x86_64-bottlerocket-linux-gnu/sys-root/...)
rm -f %{buildroot}%{_cross_kmoddir}/build %{buildroot}%{_cross_kmoddir}/source
ln -sf %{_usrsrc}/kernels/%{version} %{buildroot}%{_cross_kmoddir}/build
ln -sf %{_usrsrc}/kernels/%{version} %{buildroot}%{_cross_kmoddir}/source

# Ensure that each required FIPS module is loaded as a dependency of the
# check-fips-module.service. The list of FIPS modules is different across
# kernels but the check is consistent: it loads the "tcrypt" module after
# the other modules are loaded.
mkdir -p %{buildroot}%{_cross_unitdir}/check-fips-modules.service.d
i=0
for fipsmod in $(cat %{_sourcedir}/fipsmodules-%{_cross_arch}) ; do
  [ "${fipsmod}" == "tcrypt" ] && continue
  drop_in="$(printf "%03d\n" "${i}")-${fipsmod}.conf"
  sed -e "s|__FIPS_MODULE__|${fipsmod}|g" %{S:200} \
    > %{buildroot}%{_cross_unitdir}/check-fips-modules.service.d/"${drop_in}"
  (( i+=1 ))
done

%files
%license COPYING LICENSES/preferred/GPL-2.0 LICENSES/exceptions/Linux-syscall-note
%{_cross_attribution_file}
/boot/vmlinuz
/boot/config

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
%dir %{_cross_ksrcdir}
%{_cross_datadir}/bottlerocket/kernel-devel.squashfs
%{_cross_kmoddir}/source
%{_cross_kmoddir}/build

%files archive
%{_cross_datadir}/bottlerocket/kernel-devel.tar.xz

%files fips
%{_cross_unitdir}/check-fips-modules.service.d/*.conf

%files modules
%dir %{_cross_libdir}/modules
%dir %{_cross_kmoddir}
%{_cross_kmoddir}/modules.alias
%{_cross_kmoddir}/modules.alias.bin
%{_cross_kmoddir}/modules.builtin
%{_cross_kmoddir}/modules.builtin.alias.bin
%{_cross_kmoddir}/modules.builtin.bin
%{_cross_kmoddir}/modules.builtin.modinfo
%{_cross_kmoddir}/modules.dep
%{_cross_kmoddir}/modules.dep.bin
%{_cross_kmoddir}/modules.devname
%{_cross_kmoddir}/modules.order
%{_cross_kmoddir}/modules.softdep
%{_cross_kmoddir}/modules.symbols
%{_cross_kmoddir}/modules.symbols.bin

%if "%{_cross_arch}" == "x86_64"
%{_cross_kmoddir}/kernel/arch/x86/crypto/aesni-intel.ko.*
%{_cross_kmoddir}/kernel/arch/x86/crypto/blowfish-x86_64.ko.*
%{_cross_kmoddir}/kernel/arch/x86/crypto/camellia-aesni-avx2.ko.*
%{_cross_kmoddir}/kernel/arch/x86/crypto/camellia-aesni-avx-x86_64.ko.*
%{_cross_kmoddir}/kernel/arch/x86/crypto/camellia-x86_64.ko.*
%{_cross_kmoddir}/kernel/arch/x86/crypto/cast5-avx-x86_64.ko.*
%{_cross_kmoddir}/kernel/arch/x86/crypto/cast6-avx-x86_64.ko.*
%{_cross_kmoddir}/kernel/arch/x86/crypto/chacha-x86_64.ko.*
%{_cross_kmoddir}/kernel/arch/x86/crypto/crc32c-intel.ko.*
%{_cross_kmoddir}/kernel/arch/x86/crypto/crc32-pclmul.ko.*
%{_cross_kmoddir}/kernel/arch/x86/crypto/curve25519-x86_64.ko.*
%{_cross_kmoddir}/kernel/arch/x86/crypto/des3_ede-x86_64.ko.*
%{_cross_kmoddir}/kernel/arch/x86/crypto/ghash-clmulni-intel.ko.*
%{_cross_kmoddir}/kernel/arch/x86/crypto/poly1305-x86_64.ko.*
%{_cross_kmoddir}/kernel/arch/x86/crypto/serpent-avx2.ko.*
%{_cross_kmoddir}/kernel/arch/x86/crypto/serpent-avx-x86_64.ko.*
%{_cross_kmoddir}/kernel/arch/x86/crypto/serpent-sse2-x86_64.ko.*
%{_cross_kmoddir}/kernel/arch/x86/crypto/twofish-avx-x86_64.ko.*
%{_cross_kmoddir}/kernel/arch/x86/crypto/twofish-x86_64-3way.ko.*
%{_cross_kmoddir}/kernel/arch/x86/crypto/twofish-x86_64.ko.*
%{_cross_kmoddir}/kernel/arch/x86/kvm/kvm-amd.ko.*
%{_cross_kmoddir}/kernel/arch/x86/kvm/kvm-intel.ko.*
%{_cross_kmoddir}/kernel/arch/x86/kvm/kvm.ko.*
%{_cross_kmoddir}/kernel/arch/x86/platform/intel/iosf_mbi.ko.*
%endif
%if "%{_cross_arch}" == "aarch64"
%{_cross_kmoddir}/kernel/arch/arm64/crypto/aes-arm64.ko.*
%{_cross_kmoddir}/kernel/arch/arm64/crypto/aes-ce-blk.ko.*
%{_cross_kmoddir}/kernel/arch/arm64/crypto/aes-ce-ccm.ko.*
%{_cross_kmoddir}/kernel/arch/arm64/crypto/aes-ce-cipher.ko.*
%{_cross_kmoddir}/kernel/arch/arm64/crypto/aes-neon-blk.ko.*
%{_cross_kmoddir}/kernel/arch/arm64/crypto/aes-neon-bs.ko.*
%{_cross_kmoddir}/kernel/arch/arm64/crypto/chacha-neon.ko.*
%{_cross_kmoddir}/kernel/arch/arm64/crypto/ghash-ce.ko.*
%{_cross_kmoddir}/kernel/arch/arm64/crypto/poly1305-neon.ko.*
%{_cross_kmoddir}/kernel/arch/arm64/crypto/sha1-ce.ko.*
%{_cross_kmoddir}/kernel/arch/arm64/crypto/sha256-arm64.ko.*
%{_cross_kmoddir}/kernel/arch/arm64/crypto/sha2-ce.ko.*
%{_cross_kmoddir}/kernel/arch/arm64/crypto/sha3-ce.ko.*
%{_cross_kmoddir}/kernel/arch/arm64/crypto/sha512-arm64.ko.*
%{_cross_kmoddir}/kernel/arch/arm64/crypto/sha512-ce.ko.*
%{_cross_kmoddir}/kernel/arch/arm64/crypto/sm3-ce.ko.*
%{_cross_kmoddir}/kernel/arch/arm64/crypto/sm4-ce-cipher.ko.*
%{_cross_kmoddir}/kernel/arch/arm64/lib/xor-neon.ko.*
%endif
%{_cross_kmoddir}/kernel/crypto/af_alg.ko.*
%{_cross_kmoddir}/kernel/crypto/algif_aead.ko.*
%{_cross_kmoddir}/kernel/crypto/algif_hash.ko.*
%{_cross_kmoddir}/kernel/crypto/algif_rng.ko.*
%{_cross_kmoddir}/kernel/crypto/algif_skcipher.ko.*
%{_cross_kmoddir}/kernel/crypto/ansi_cprng.ko.*
%{_cross_kmoddir}/kernel/crypto/anubis.ko.*
%{_cross_kmoddir}/kernel/crypto/arc4.ko.*
%{_cross_kmoddir}/kernel/crypto/asymmetric_keys/pkcs7_test_key.ko.*
%{_cross_kmoddir}/kernel/crypto/async_tx/async_memcpy.ko.*
%{_cross_kmoddir}/kernel/crypto/async_tx/async_pq.ko.*
%{_cross_kmoddir}/kernel/crypto/async_tx/async_raid6_recov.ko.*
%{_cross_kmoddir}/kernel/crypto/async_tx/async_tx.ko.*
%{_cross_kmoddir}/kernel/crypto/async_tx/async_xor.ko.*
%{_cross_kmoddir}/kernel/crypto/authencesn.ko.*
%{_cross_kmoddir}/kernel/crypto/authenc.ko.*
%{_cross_kmoddir}/kernel/crypto/blake2b_generic.ko.*
%{_cross_kmoddir}/kernel/crypto/blowfish_common.ko.*
%{_cross_kmoddir}/kernel/crypto/blowfish_generic.ko.*
%{_cross_kmoddir}/kernel/crypto/camellia_generic.ko.*
%{_cross_kmoddir}/kernel/crypto/cast5_generic.ko.*
%{_cross_kmoddir}/kernel/crypto/cast6_generic.ko.*
%{_cross_kmoddir}/kernel/crypto/cast_common.ko.*
%{_cross_kmoddir}/kernel/crypto/cbc.ko.*
%{_cross_kmoddir}/kernel/crypto/ccm.ko.*
%{_cross_kmoddir}/kernel/crypto/cfb.ko.*
%{_cross_kmoddir}/kernel/crypto/chacha20poly1305.ko.*
%{_cross_kmoddir}/kernel/crypto/chacha_generic.ko.*
%{_cross_kmoddir}/kernel/crypto/cmac.ko.*
%{_cross_kmoddir}/kernel/crypto/crc32_generic.ko.*
%{_cross_kmoddir}/kernel/crypto/cryptd.ko.*
%{_cross_kmoddir}/kernel/crypto/crypto_user.ko.*
%{_cross_kmoddir}/kernel/crypto/cts.ko.*
%{_cross_kmoddir}/kernel/crypto/des_generic.ko.*
%{_cross_kmoddir}/kernel/crypto/ecb.ko.*
%{_cross_kmoddir}/kernel/crypto/echainiv.ko.*
%{_cross_kmoddir}/kernel/crypto/essiv.ko.*
%{_cross_kmoddir}/kernel/crypto/fcrypt.ko.*
%{_cross_kmoddir}/kernel/crypto/gcm.ko.*
%{_cross_kmoddir}/kernel/crypto/keywrap.ko.*
%{_cross_kmoddir}/kernel/crypto/khazad.ko.*
%{_cross_kmoddir}/kernel/crypto/lrw.ko.*
%{_cross_kmoddir}/kernel/crypto/lz4hc.ko.*
%{_cross_kmoddir}/kernel/crypto/lz4.ko.*
%{_cross_kmoddir}/kernel/crypto/md4.ko.*
%{_cross_kmoddir}/kernel/crypto/michael_mic.ko.*
%{_cross_kmoddir}/kernel/crypto/ofb.ko.*
%{_cross_kmoddir}/kernel/crypto/pcbc.ko.*
%{_cross_kmoddir}/kernel/crypto/pcrypt.ko.*
%{_cross_kmoddir}/kernel/crypto/poly1305_generic.ko.*
%{_cross_kmoddir}/kernel/crypto/rmd160.ko.*
%{_cross_kmoddir}/kernel/crypto/seed.ko.*
%{_cross_kmoddir}/kernel/crypto/serpent_generic.ko.*
%{_cross_kmoddir}/kernel/crypto/tcrypt.ko.*
%{_cross_kmoddir}/kernel/crypto/tea.ko.*
%{_cross_kmoddir}/kernel/crypto/twofish_common.ko.*
%{_cross_kmoddir}/kernel/crypto/twofish_generic.ko.*
%{_cross_kmoddir}/kernel/crypto/vmac.ko.*
%{_cross_kmoddir}/kernel/crypto/wp512.ko.*
%{_cross_kmoddir}/kernel/crypto/xcbc.ko.*
%{_cross_kmoddir}/kernel/crypto/xor.ko.*
%{_cross_kmoddir}/kernel/crypto/xts.ko.*
%{_cross_kmoddir}/kernel/crypto/xxhash_generic.ko.*
%{_cross_kmoddir}/kernel/crypto/zstd.ko.*
%if "%{_cross_arch}" == "x86_64"
%{_cross_kmoddir}/kernel/crypto/crypto_simd.ko.*
%endif
%if "%{_cross_arch}" == "aarch64"
%{_cross_kmoddir}/kernel/crypto/sm3.ko.*
%{_cross_kmoddir}/kernel/crypto/sm4.ko.*
%endif
%{_cross_kmoddir}/kernel/drivers/acpi/ac.ko.*
%{_cross_kmoddir}/kernel/drivers/acpi/button.ko.*
%{_cross_kmoddir}/kernel/drivers/acpi/thermal.ko.*
%if "%{_cross_arch}" == "x86_64"
%{_cross_kmoddir}/kernel/drivers/acpi/acpi_extlog.ko.*
%{_cross_kmoddir}/kernel/drivers/acpi/acpi_pad.ko.*
%{_cross_kmoddir}/kernel/drivers/acpi/video.ko.*
%endif
%{_cross_kmoddir}/kernel/drivers/amazon/net/efa/efa.ko.*
%{_cross_kmoddir}/kernel/drivers/amazon/net/ena/ena.ko.*
%if "%{_cross_arch}" == "aarch64"
%{_cross_kmoddir}/kernel/drivers/ata/ahci_platform.ko.*
%{_cross_kmoddir}/kernel/drivers/ata/libahci_platform.ko.*
%endif
%{_cross_kmoddir}/kernel/drivers/block/brd.ko.*
%{_cross_kmoddir}/kernel/drivers/block/drbd/drbd.ko.*
%{_cross_kmoddir}/kernel/drivers/block/loop.ko.*
%{_cross_kmoddir}/kernel/drivers/block/nbd.ko.*
%{_cross_kmoddir}/kernel/drivers/block/null_blk/null_blk.ko.*
%{_cross_kmoddir}/kernel/drivers/block/pktcdvd.ko.*
%{_cross_kmoddir}/kernel/drivers/block/rbd.ko.*
%{_cross_kmoddir}/kernel/drivers/block/zram/zram.ko.*
%{_cross_kmoddir}/kernel/drivers/cdrom/cdrom.ko.*
%{_cross_kmoddir}/kernel/drivers/char/ipmi/ipmi_msghandler.ko.*
%{_cross_kmoddir}/kernel/drivers/char/virtio_console.ko.*
%if "%{_cross_arch}" == "x86_64"
%{_cross_kmoddir}/kernel/drivers/char/agp/intel-gtt.ko.*
%{_cross_kmoddir}/kernel/drivers/char/hangcheck-timer.ko.*
%{_cross_kmoddir}/kernel/drivers/char/nvram.ko.*
%endif
%{_cross_kmoddir}/kernel/drivers/char/hw_random/rng-core.ko.*
%{_cross_kmoddir}/kernel/drivers/char/hw_random/virtio-rng.ko.*
%if "%{_cross_arch}" == "x86_64"
%{_cross_kmoddir}/kernel/drivers/char/hw_random/amd-rng.ko.*
%{_cross_kmoddir}/kernel/drivers/char/hw_random/intel-rng.ko.*
%endif
%if "%{_cross_arch}" == "aarch64"
%{_cross_kmoddir}/kernel/drivers/char/hw_random/arm_smccc_trng.ko.*
%{_cross_kmoddir}/kernel/drivers/char/hw_random/cn10k-rng.ko.*
%{_cross_kmoddir}/kernel/drivers/char/hw_random/graviton-rng.ko.*
%endif
%{_cross_kmoddir}/kernel/drivers/cpufreq/cpufreq_conservative.ko.*
%{_cross_kmoddir}/kernel/drivers/cpufreq/cpufreq_ondemand.ko.*
%{_cross_kmoddir}/kernel/drivers/cpufreq/cpufreq_powersave.ko.*
%{_cross_kmoddir}/kernel/drivers/cpufreq/cpufreq_userspace.ko.*
%if "%{_cross_arch}" == "x86_64"
%{_cross_kmoddir}/kernel/drivers/cpufreq/acpi-cpufreq.ko.*
%{_cross_kmoddir}/kernel/drivers/cpufreq/pcc-cpufreq.ko.*
%{_cross_kmoddir}/kernel/drivers/dca/dca.ko.*
%{_cross_kmoddir}/kernel/drivers/dma/ioat/ioatdma.ko.*
%{_cross_kmoddir}/kernel/drivers/edac/amd64_edac.ko.*
%{_cross_kmoddir}/kernel/drivers/edac/e752x_edac.ko.*
%{_cross_kmoddir}/kernel/drivers/edac/i3000_edac.ko.*
%{_cross_kmoddir}/kernel/drivers/edac/i3200_edac.ko.*
%{_cross_kmoddir}/kernel/drivers/edac/i5000_edac.ko.*
%{_cross_kmoddir}/kernel/drivers/edac/i5100_edac.ko.*
%{_cross_kmoddir}/kernel/drivers/edac/i5400_edac.ko.*
%{_cross_kmoddir}/kernel/drivers/edac/i7300_edac.ko.*
%{_cross_kmoddir}/kernel/drivers/edac/i7core_edac.ko.*
%{_cross_kmoddir}/kernel/drivers/edac/i82975x_edac.ko.*
%{_cross_kmoddir}/kernel/drivers/edac/ie31200_edac.ko.*
%{_cross_kmoddir}/kernel/drivers/edac/pnd2_edac.ko.*
%{_cross_kmoddir}/kernel/drivers/edac/sb_edac.ko.*
%{_cross_kmoddir}/kernel/drivers/edac/skx_edac.ko.*
%{_cross_kmoddir}/kernel/drivers/edac/x38_edac.ko.*
%endif
%{_cross_kmoddir}/kernel/drivers/firmware/dmi-sysfs.ko.*
%if "%{_cross_arch}" == "aarch64"
%{_cross_kmoddir}/kernel/drivers/firmware/arm_scpi.ko.*
%{_cross_kmoddir}/kernel/drivers/firmware/scpi_pm_domain.ko.*
%endif
%{_cross_kmoddir}/kernel/drivers/gpu/drm/drm_kms_helper.ko.*
%{_cross_kmoddir}/kernel/drivers/gpu/drm/drm.ko.*
%{_cross_kmoddir}/kernel/drivers/gpu/drm/drm_shmem_helper.ko.*
%{_cross_kmoddir}/kernel/drivers/gpu/drm/tiny/simpledrm.ko.*
%if "%{_cross_arch}" == "x86_64"
%{_cross_kmoddir}/kernel/drivers/gpu/drm/drm_buddy.ko.*
%{_cross_kmoddir}/kernel/drivers/gpu/drm/display/drm_display_helper.ko.*
%{_cross_kmoddir}/kernel/drivers/gpu/drm/i915/i915.ko.*
%{_cross_kmoddir}/kernel/drivers/gpu/drm/ttm/ttm.ko.*
%endif
%{_cross_kmoddir}/kernel/drivers/hid/hid-generic.ko.*
%{_cross_kmoddir}/kernel/drivers/hid/hid-multitouch.ko.*
%{_cross_kmoddir}/kernel/drivers/hid/uhid.ko.*
%{_cross_kmoddir}/kernel/drivers/hid/usbhid/usbhid.ko.*
%if "%{_cross_arch}" == "x86_64"
%{_cross_kmoddir}/kernel/drivers/hid/hid-hyperv.ko.*
%{_cross_kmoddir}/kernel/drivers/hv/hv_balloon.ko.*
%{_cross_kmoddir}/kernel/drivers/hv/hv_utils.ko.*
%{_cross_kmoddir}/kernel/drivers/hv/hv_vmbus.ko.*
%endif
%{_cross_kmoddir}/kernel/drivers/hwmon/acpi_power_meter.ko.*
%{_cross_kmoddir}/kernel/drivers/hwmon/hwmon.ko.*
%{_cross_kmoddir}/kernel/drivers/i2c/algos/i2c-algo-bit.ko.*
%{_cross_kmoddir}/kernel/drivers/i2c/i2c-core.ko.*
%{_cross_kmoddir}/kernel/drivers/infiniband/core/ib_cm.ko.*
%{_cross_kmoddir}/kernel/drivers/infiniband/core/ib_core.ko.*
%{_cross_kmoddir}/kernel/drivers/infiniband/core/ib_uverbs.ko.*
%{_cross_kmoddir}/kernel/drivers/infiniband/core/iw_cm.ko.*
%{_cross_kmoddir}/kernel/drivers/infiniband/core/rdma_cm.ko.*
%{_cross_kmoddir}/kernel/drivers/infiniband/core/rdma_ucm.ko.*
%{_cross_kmoddir}/kernel/drivers/infiniband/hw/mlx5/mlx5_ib.ko.*
%{_cross_kmoddir}/kernel/drivers/input/misc/uinput.ko.*
%{_cross_kmoddir}/kernel/drivers/input/mousedev.ko.*
%{_cross_kmoddir}/kernel/drivers/input/keyboard/atkbd.ko.*
%{_cross_kmoddir}/kernel/drivers/input/mouse/psmouse.ko.*
%{_cross_kmoddir}/kernel/drivers/input/serio/libps2.ko.*
%{_cross_kmoddir}/kernel/drivers/input/serio/serio.ko.*
%{_cross_kmoddir}/kernel/drivers/input/serio/serport.ko.*
%{_cross_kmoddir}/kernel/drivers/input/sparse-keymap.ko.*
%{_cross_kmoddir}/kernel/drivers/input/vivaldi-fmap.ko.*
%if "%{_cross_arch}" == "x86_64"
%{_cross_kmoddir}/kernel/drivers/input/serio/hyperv-keyboard.ko.*
%{_cross_kmoddir}/kernel/drivers/input/serio/i8042.ko.*
%endif
%{_cross_kmoddir}/kernel/drivers/iommu/virtio-iommu.ko.*
%if "%{_cross_arch}" == "aarch64"
%{_cross_kmoddir}/kernel/drivers/mailbox/arm_mhu_db.ko.*
%{_cross_kmoddir}/kernel/drivers/mailbox/arm_mhu.ko.*
%endif
%{_cross_kmoddir}/kernel/drivers/md/bcache/bcache.ko.*
%{_cross_kmoddir}/kernel/drivers/md/dm-bio-prison.ko.*
%{_cross_kmoddir}/kernel/drivers/md/dm-cache.ko.*
%{_cross_kmoddir}/kernel/drivers/md/dm-cache-smq.ko.*
%{_cross_kmoddir}/kernel/drivers/md/dm-crypt.ko.*
%{_cross_kmoddir}/kernel/drivers/md/dm-delay.ko.*
%{_cross_kmoddir}/kernel/drivers/md/dm-dust.ko.*
%{_cross_kmoddir}/kernel/drivers/md/dm-flakey.ko.*
%{_cross_kmoddir}/kernel/drivers/md/dm-integrity.ko.*
%{_cross_kmoddir}/kernel/drivers/md/dm-log.ko.*
%{_cross_kmoddir}/kernel/drivers/md/dm-log-userspace.ko.*
%{_cross_kmoddir}/kernel/drivers/md/dm-log-writes.ko.*
%{_cross_kmoddir}/kernel/drivers/md/dm-mirror.ko.*
%{_cross_kmoddir}/kernel/drivers/md/dm-multipath.ko.*
%{_cross_kmoddir}/kernel/drivers/md/dm-queue-length.ko.*
%{_cross_kmoddir}/kernel/drivers/md/dm-raid.ko.*
%{_cross_kmoddir}/kernel/drivers/md/dm-region-hash.ko.*
%{_cross_kmoddir}/kernel/drivers/md/dm-round-robin.ko.*
%{_cross_kmoddir}/kernel/drivers/md/dm-service-time.ko.*
%{_cross_kmoddir}/kernel/drivers/md/dm-snapshot.ko.*
%{_cross_kmoddir}/kernel/drivers/md/dm-thin-pool.ko.*
%{_cross_kmoddir}/kernel/drivers/md/dm-zero.ko.*
%{_cross_kmoddir}/kernel/drivers/md/faulty.ko.*
%{_cross_kmoddir}/kernel/drivers/md/linear.ko.*
%{_cross_kmoddir}/kernel/drivers/md/persistent-data/dm-persistent-data.ko.*
%{_cross_kmoddir}/kernel/drivers/md/raid0.ko.*
%{_cross_kmoddir}/kernel/drivers/md/raid10.ko.*
%{_cross_kmoddir}/kernel/drivers/md/raid1.ko.*
%{_cross_kmoddir}/kernel/drivers/md/raid456.ko.*
%{_cross_kmoddir}/kernel/drivers/mfd/lpc_ich.ko.*
%{_cross_kmoddir}/kernel/drivers/mfd/lpc_sch.ko.*
%if "%{_cross_arch}" == "x86_64"
%{_cross_kmoddir}/kernel/drivers/mfd/mfd-core.ko.*
%{_cross_kmoddir}/kernel/drivers/misc/vmw_balloon.ko.*
%{_cross_kmoddir}/kernel/drivers/misc/vmw_vmci/vmw_vmci.ko.*
%endif
%{_cross_kmoddir}/kernel/drivers/net/bonding/bonding.ko.*
%{_cross_kmoddir}/kernel/drivers/net/dummy.ko.*
%{_cross_kmoddir}/kernel/drivers/net/ethernet/intel/e1000/e1000.ko.*
%{_cross_kmoddir}/kernel/drivers/net/ethernet/intel/e1000e/e1000e.ko.*
%{_cross_kmoddir}/kernel/drivers/net/ethernet/intel/igb/igb.ko.*
%{_cross_kmoddir}/kernel/drivers/net/ethernet/intel/ixgbevf/ixgbevf.ko.*
%{_cross_kmoddir}/kernel/drivers/net/ethernet/mellanox/mlx5/core/mlx5_core.ko.*
%{_cross_kmoddir}/kernel/drivers/net/ethernet/mellanox/mlxfw/mlxfw.ko.*
%{_cross_kmoddir}/kernel/drivers/net/geneve.ko.*
%if "%{_cross_arch}" == "x86_64"
%{_cross_kmoddir}/kernel/drivers/net/hyperv/hv_netvsc.ko.*
%endif
%{_cross_kmoddir}/kernel/drivers/net/ifb.ko.*
%{_cross_kmoddir}/kernel/drivers/net/ipvlan/ipvlan.ko.*
%{_cross_kmoddir}/kernel/drivers/net/ipvlan/ipvtap.ko.*
%{_cross_kmoddir}/kernel/drivers/net/macvlan.ko.*
%{_cross_kmoddir}/kernel/drivers/net/macvtap.ko.*
%{_cross_kmoddir}/kernel/drivers/net/mdio/acpi_mdio.ko.*
%{_cross_kmoddir}/kernel/drivers/net/mdio/fwnode_mdio.ko.*
%{_cross_kmoddir}/kernel/drivers/net/netdevsim/netdevsim.ko.*
%{_cross_kmoddir}/kernel/drivers/net/net_failover.ko.*
%{_cross_kmoddir}/kernel/drivers/net/nlmon.ko.*
%{_cross_kmoddir}/kernel/drivers/net/phy/fixed_phy.ko.*
%{_cross_kmoddir}/kernel/drivers/net/phy/libphy.ko.*
%{_cross_kmoddir}/kernel/drivers/net/phy/mdio_devres.ko.*
%{_cross_kmoddir}/kernel/drivers/net/tap.ko.*
%{_cross_kmoddir}/kernel/drivers/net/team/team.ko.*
%{_cross_kmoddir}/kernel/drivers/net/team/team_mode_activebackup.ko.*
%{_cross_kmoddir}/kernel/drivers/net/team/team_mode_broadcast.ko.*
%{_cross_kmoddir}/kernel/drivers/net/team/team_mode_loadbalance.ko.*
%{_cross_kmoddir}/kernel/drivers/net/team/team_mode_roundrobin.ko.*
%{_cross_kmoddir}/kernel/drivers/net/tun.ko.*
%{_cross_kmoddir}/kernel/drivers/net/veth.ko.*
%{_cross_kmoddir}/kernel/drivers/net/virtio_net.ko.*
%{_cross_kmoddir}/kernel/drivers/net/vmxnet3/vmxnet3.ko.*
%{_cross_kmoddir}/kernel/drivers/net/vrf.ko.*
%{_cross_kmoddir}/kernel/drivers/net/vxlan/vxlan.ko.*
%{_cross_kmoddir}/kernel/drivers/net/wireguard/wireguard.ko.*
%if "%{_cross_arch}" == "x86_64"
%{_cross_kmoddir}/kernel/drivers/net/xen-netback/xen-netback.ko.*
%endif
%if "%{_cross_arch}" == "aarch64"
%{_cross_kmoddir}/kernel/drivers/net/mdio/of_mdio.ko.*
%endif
%{_cross_kmoddir}/kernel/drivers/nvme/host/nvme-fabrics.ko.*
%{_cross_kmoddir}/kernel/drivers/nvme/host/nvme-tcp.ko.*
%{_cross_kmoddir}/kernel/drivers/pci/hotplug/acpiphp_ibm.ko.*
%{_cross_kmoddir}/kernel/drivers/pci/pci-stub.ko.*
%if "%{_cross_arch}" == "x86_64"
%{_cross_kmoddir}/kernel/drivers/pci/hotplug/cpcihp_generic.ko.*
%{_cross_kmoddir}/kernel/drivers/platform/x86/wmi-bmof.ko.*
%{_cross_kmoddir}/kernel/drivers/platform/x86/wmi.ko.*
%endif
%if "%{_cross_arch}" == "aarch64"
%{_cross_kmoddir}/kernel/drivers/perf/arm-cmn.ko.*
%endif
%{_cross_kmoddir}/kernel/drivers/pps/clients/pps-gpio.ko.*
%{_cross_kmoddir}/kernel/drivers/pps/clients/pps-ldisc.ko.*
%{_cross_kmoddir}/kernel/drivers/pps/pps_core.ko.*
%{_cross_kmoddir}/kernel/drivers/ptp/ptp.ko.*
%{_cross_kmoddir}/kernel/drivers/ptp/ptp_kvm.ko.*
%{_cross_kmoddir}/kernel/drivers/scsi/ch.ko.*
%if "%{_cross_arch}" == "x86_64"
%{_cross_kmoddir}/kernel/drivers/scsi/hv_storvsc.ko.*
%endif
%{_cross_kmoddir}/kernel/drivers/scsi/iscsi_boot_sysfs.ko.*
%{_cross_kmoddir}/kernel/drivers/scsi/iscsi_tcp.ko.*
%{_cross_kmoddir}/kernel/drivers/scsi/libiscsi.ko.*
%{_cross_kmoddir}/kernel/drivers/scsi/libiscsi_tcp.ko.*
%{_cross_kmoddir}/kernel/drivers/scsi/scsi_transport_iscsi.ko.*
%{_cross_kmoddir}/kernel/drivers/scsi/sg.ko.*
%{_cross_kmoddir}/kernel/drivers/scsi/sr_mod.ko.*
%{_cross_kmoddir}/kernel/drivers/scsi/st.ko.*
%if "%{_cross_arch}" == "x86_64"
%{_cross_kmoddir}/kernel/drivers/scsi/vmw_pvscsi.ko.*
%{_cross_kmoddir}/kernel/drivers/scsi/xen-scsifront.ko.*
%endif
%{_cross_kmoddir}/kernel/drivers/staging/lustrefsx/libcfs/libcfs/libcfs.ko.*
%{_cross_kmoddir}/kernel/drivers/staging/lustrefsx/lnet/klnds/o2iblnd/ko2iblnd.ko.*
%{_cross_kmoddir}/kernel/drivers/staging/lustrefsx/lnet/klnds/socklnd/ksocklnd.ko.*
%{_cross_kmoddir}/kernel/drivers/staging/lustrefsx/lnet/lnet/lnet.ko.*
%{_cross_kmoddir}/kernel/drivers/staging/lustrefsx/lnet/selftest/lnet_selftest.ko.*
%{_cross_kmoddir}/kernel/drivers/staging/lustrefsx/lustre/fid/fid.ko.*
%{_cross_kmoddir}/kernel/drivers/staging/lustrefsx/lustre/fld/fld.ko.*
%{_cross_kmoddir}/kernel/drivers/staging/lustrefsx/lustre/llite/lustre.ko.*
%{_cross_kmoddir}/kernel/drivers/staging/lustrefsx/lustre/lmv/lmv.ko.*
%{_cross_kmoddir}/kernel/drivers/staging/lustrefsx/lustre/lov/lov.ko.*
%{_cross_kmoddir}/kernel/drivers/staging/lustrefsx/lustre/mdc/mdc.ko.*
%{_cross_kmoddir}/kernel/drivers/staging/lustrefsx/lustre/mgc/mgc.ko.*
%{_cross_kmoddir}/kernel/drivers/staging/lustrefsx/lustre/obdclass/obdclass.ko.*
%{_cross_kmoddir}/kernel/drivers/staging/lustrefsx/lustre/obdecho/obdecho.ko.*
%{_cross_kmoddir}/kernel/drivers/staging/lustrefsx/lustre/osc/osc.ko.*
%{_cross_kmoddir}/kernel/drivers/staging/lustrefsx/lustre/ptlrpc/ptlrpc.ko.*
%{_cross_kmoddir}/kernel/drivers/target/iscsi/iscsi_target_mod.ko.*
%{_cross_kmoddir}/kernel/drivers/target/loopback/tcm_loop.ko.*
%{_cross_kmoddir}/kernel/drivers/target/target_core_file.ko.*
%{_cross_kmoddir}/kernel/drivers/target/target_core_iblock.ko.*
%{_cross_kmoddir}/kernel/drivers/target/target_core_mod.ko.*
%{_cross_kmoddir}/kernel/drivers/target/target_core_user.ko.*
%if "%{_cross_arch}" == "x86_64"
%{_cross_kmoddir}/kernel/drivers/thermal/intel/x86_pkg_temp_thermal.ko.*
%endif
%{_cross_kmoddir}/kernel/drivers/tty/serial/8250/8250_exar.ko.*
%{_cross_kmoddir}/kernel/drivers/uio/uio_dmem_genirq.ko.*
%{_cross_kmoddir}/kernel/drivers/uio/uio.ko.*
%{_cross_kmoddir}/kernel/drivers/uio/uio_pci_generic.ko.*
%{_cross_kmoddir}/kernel/drivers/uio/uio_pdrv_genirq.ko.*
%if "%{_cross_arch}" == "x86_64"
%{_cross_kmoddir}/kernel/drivers/uio/uio_hv_generic.ko.*
%endif
%{_cross_kmoddir}/kernel/drivers/usb/class/cdc-acm.ko.*
%{_cross_kmoddir}/kernel/drivers/usb/common/usb-common.ko.*
%{_cross_kmoddir}/kernel/drivers/usb/core/usbcore.ko.*
%{_cross_kmoddir}/kernel/drivers/usb/host/ehci-hcd.ko.*
%{_cross_kmoddir}/kernel/drivers/usb/host/ehci-pci.ko.*
%{_cross_kmoddir}/kernel/drivers/usb/host/ehci-platform.ko.*
%{_cross_kmoddir}/kernel/drivers/usb/host/ohci-hcd.ko.*
%{_cross_kmoddir}/kernel/drivers/usb/host/ohci-pci.ko.*
%{_cross_kmoddir}/kernel/drivers/usb/host/ohci-platform.ko.*
%{_cross_kmoddir}/kernel/drivers/usb/host/uhci-hcd.ko.*
%{_cross_kmoddir}/kernel/drivers/usb/host/xhci-hcd.ko.*
%{_cross_kmoddir}/kernel/drivers/usb/host/xhci-pci.ko.*
%{_cross_kmoddir}/kernel/drivers/usb/host/xhci-plat-hcd.ko.*
%{_cross_kmoddir}/kernel/drivers/usb/mon/usbmon.ko.*
%{_cross_kmoddir}/kernel/drivers/usb/serial/cp210x.ko.*
%{_cross_kmoddir}/kernel/drivers/usb/serial/ftdi_sio.ko.*
%{_cross_kmoddir}/kernel/drivers/usb/serial/usbserial.ko.*
%{_cross_kmoddir}/kernel/drivers/usb/storage/uas.ko.*
%{_cross_kmoddir}/kernel/drivers/usb/storage/usb-storage.ko.*
%{_cross_kmoddir}/kernel/drivers/usb/usbip/usbip-core.ko.*
%{_cross_kmoddir}/kernel/drivers/usb/usbip/usbip-host.ko.*
%{_cross_kmoddir}/kernel/drivers/usb/usbip/vhci-hcd.ko.*
%{_cross_kmoddir}/kernel/drivers/vfio/pci/vfio-pci-core.ko.*
%{_cross_kmoddir}/kernel/drivers/vfio/pci/vfio-pci.ko.*
%{_cross_kmoddir}/kernel/drivers/vfio/vfio_iommu_type1.ko.*
%{_cross_kmoddir}/kernel/drivers/vfio/vfio.ko.*
%{_cross_kmoddir}/kernel/drivers/vfio/vfio_virqfd.ko.*
%{_cross_kmoddir}/kernel/drivers/vhost/vhost_iotlb.ko.*
%{_cross_kmoddir}/kernel/drivers/vhost/vhost.ko.*
%{_cross_kmoddir}/kernel/drivers/vhost/vhost_net.ko.*
%{_cross_kmoddir}/kernel/drivers/vhost/vhost_vsock.ko.*
%{_cross_kmoddir}/kernel/drivers/video/backlight/backlight.ko.*
%{_cross_kmoddir}/kernel/drivers/video/backlight/lcd.ko.*
%{_cross_kmoddir}/kernel/drivers/video/fbdev/core/fb_sys_fops.ko.*
%{_cross_kmoddir}/kernel/drivers/video/fbdev/core/syscopyarea.ko.*
%{_cross_kmoddir}/kernel/drivers/video/fbdev/core/sysfillrect.ko.*
%{_cross_kmoddir}/kernel/drivers/video/fbdev/core/sysimgblt.ko.*
%if "%{_cross_arch}" == "x86_64"
%{_cross_kmoddir}/kernel/drivers/virt/coco/sev-guest/sev-guest.ko.*
%{_cross_kmoddir}/kernel/drivers/virt/vboxguest/vboxguest.ko.*
%endif
%if "%{_cross_arch}" == "aarch64"
%{_cross_kmoddir}/kernel/drivers/virt/nitro_enclaves/nitro_enclaves.ko.*
%endif
%{_cross_kmoddir}/kernel/drivers/virtio/virtio_balloon.ko.*
%{_cross_kmoddir}/kernel/drivers/virtio/virtio_mmio.ko.*
%if "%{_cross_arch}" == "x86_64"
%{_cross_kmoddir}/kernel/drivers/virtio/virtio_mem.ko.*
%endif
%{_cross_kmoddir}/kernel/drivers/watchdog/softdog.ko.*
%if "%{_cross_arch}" == "aarch64"
%{_cross_kmoddir}/kernel/drivers/watchdog/gpio_wdt.ko.*
%{_cross_kmoddir}/kernel/drivers/watchdog/sbsa_gwdt.ko.*
%{_cross_kmoddir}/kernel/drivers/watchdog/sp805_wdt.ko.*
%endif
%if "%{_cross_arch}" == "x86_64"
%{_cross_kmoddir}/kernel/drivers/xen/xen-evtchn.ko.*
%{_cross_kmoddir}/kernel/drivers/xen/xenfs/xenfs.ko.*
%{_cross_kmoddir}/kernel/drivers/xen/xen-gntalloc.ko.*
%{_cross_kmoddir}/kernel/drivers/xen/xen-gntdev.ko.*
%{_cross_kmoddir}/kernel/drivers/xen/xen-pciback/xen-pciback.ko.*
%{_cross_kmoddir}/kernel/drivers/xen/xen-privcmd.ko.*
%endif
%{_cross_kmoddir}/kernel/fs/binfmt_misc.ko.*
%{_cross_kmoddir}/kernel/fs/btrfs/btrfs.ko.*
%{_cross_kmoddir}/kernel/fs/cachefiles/cachefiles.ko.*
%{_cross_kmoddir}/kernel/fs/ceph/ceph.ko.*
%{_cross_kmoddir}/kernel/fs/configfs/configfs.ko.*
%{_cross_kmoddir}/kernel/fs/efivarfs/efivarfs.ko.*
%{_cross_kmoddir}/kernel/fs/exfat/exfat.ko.*
%{_cross_kmoddir}/kernel/fs/fat/fat.ko.*
%{_cross_kmoddir}/kernel/fs/fat/msdos.ko.*
%{_cross_kmoddir}/kernel/fs/fat/vfat.ko.*
%{_cross_kmoddir}/kernel/fs/fscache/fscache.ko.*
%{_cross_kmoddir}/kernel/fs/fuse/cuse.ko.*
%{_cross_kmoddir}/kernel/fs/fuse/fuse.ko.*
%{_cross_kmoddir}/kernel/fs/fuse/virtiofs.ko.*
%{_cross_kmoddir}/kernel/fs/isofs/isofs.ko.*
%{_cross_kmoddir}/kernel/fs/lockd/lockd.ko.*
%{_cross_kmoddir}/kernel/fs/netfs/netfs.ko.*
%{_cross_kmoddir}/kernel/fs/nfs/blocklayout/blocklayoutdriver.ko.*
%{_cross_kmoddir}/kernel/fs/nfs_common/grace.ko.*
%{_cross_kmoddir}/kernel/fs/nfs_common/nfs_acl.ko.*
%{_cross_kmoddir}/kernel/fs/nfsd/nfsd.ko.*
%{_cross_kmoddir}/kernel/fs/nfs/filelayout/nfs_layout_nfsv41_files.ko.*
%{_cross_kmoddir}/kernel/fs/nfs/flexfilelayout/nfs_layout_flexfiles.ko.*
%{_cross_kmoddir}/kernel/fs/nfs/nfs.ko.*
%{_cross_kmoddir}/kernel/fs/nfs/nfsv3.ko.*
%{_cross_kmoddir}/kernel/fs/nfs/nfsv4.ko.*
%{_cross_kmoddir}/kernel/fs/nls/mac-celtic.ko.*
%{_cross_kmoddir}/kernel/fs/nls/mac-centeuro.ko.*
%{_cross_kmoddir}/kernel/fs/nls/mac-croatian.ko.*
%{_cross_kmoddir}/kernel/fs/nls/mac-cyrillic.ko.*
%{_cross_kmoddir}/kernel/fs/nls/mac-gaelic.ko.*
%{_cross_kmoddir}/kernel/fs/nls/mac-greek.ko.*
%{_cross_kmoddir}/kernel/fs/nls/mac-iceland.ko.*
%{_cross_kmoddir}/kernel/fs/nls/mac-inuit.ko.*
%{_cross_kmoddir}/kernel/fs/nls/mac-romanian.ko.*
%{_cross_kmoddir}/kernel/fs/nls/mac-roman.ko.*
%{_cross_kmoddir}/kernel/fs/nls/mac-turkish.ko.*
%{_cross_kmoddir}/kernel/fs/nls/nls_ascii.ko.*
%{_cross_kmoddir}/kernel/fs/nls/nls_cp1250.ko.*
%{_cross_kmoddir}/kernel/fs/nls/nls_cp1251.ko.*
%{_cross_kmoddir}/kernel/fs/nls/nls_cp1255.ko.*
%{_cross_kmoddir}/kernel/fs/nls/nls_cp437.ko.*
%{_cross_kmoddir}/kernel/fs/nls/nls_cp737.ko.*
%{_cross_kmoddir}/kernel/fs/nls/nls_cp775.ko.*
%{_cross_kmoddir}/kernel/fs/nls/nls_cp850.ko.*
%{_cross_kmoddir}/kernel/fs/nls/nls_cp852.ko.*
%{_cross_kmoddir}/kernel/fs/nls/nls_cp855.ko.*
%{_cross_kmoddir}/kernel/fs/nls/nls_cp857.ko.*
%{_cross_kmoddir}/kernel/fs/nls/nls_cp860.ko.*
%{_cross_kmoddir}/kernel/fs/nls/nls_cp861.ko.*
%{_cross_kmoddir}/kernel/fs/nls/nls_cp862.ko.*
%{_cross_kmoddir}/kernel/fs/nls/nls_cp863.ko.*
%{_cross_kmoddir}/kernel/fs/nls/nls_cp864.ko.*
%{_cross_kmoddir}/kernel/fs/nls/nls_cp865.ko.*
%{_cross_kmoddir}/kernel/fs/nls/nls_cp866.ko.*
%{_cross_kmoddir}/kernel/fs/nls/nls_cp869.ko.*
%{_cross_kmoddir}/kernel/fs/nls/nls_cp874.ko.*
%{_cross_kmoddir}/kernel/fs/nls/nls_cp932.ko.*
%{_cross_kmoddir}/kernel/fs/nls/nls_cp936.ko.*
%{_cross_kmoddir}/kernel/fs/nls/nls_cp949.ko.*
%{_cross_kmoddir}/kernel/fs/nls/nls_cp950.ko.*
%{_cross_kmoddir}/kernel/fs/nls/nls_euc-jp.ko.*
%{_cross_kmoddir}/kernel/fs/nls/nls_iso8859-13.ko.*
%{_cross_kmoddir}/kernel/fs/nls/nls_iso8859-14.ko.*
%{_cross_kmoddir}/kernel/fs/nls/nls_iso8859-15.ko.*
%{_cross_kmoddir}/kernel/fs/nls/nls_iso8859-1.ko.*
%{_cross_kmoddir}/kernel/fs/nls/nls_iso8859-2.ko.*
%{_cross_kmoddir}/kernel/fs/nls/nls_iso8859-3.ko.*
%{_cross_kmoddir}/kernel/fs/nls/nls_iso8859-4.ko.*
%{_cross_kmoddir}/kernel/fs/nls/nls_iso8859-5.ko.*
%{_cross_kmoddir}/kernel/fs/nls/nls_iso8859-6.ko.*
%{_cross_kmoddir}/kernel/fs/nls/nls_iso8859-7.ko.*
%{_cross_kmoddir}/kernel/fs/nls/nls_iso8859-9.ko.*
%{_cross_kmoddir}/kernel/fs/nls/nls_koi8-r.ko.*
%{_cross_kmoddir}/kernel/fs/nls/nls_koi8-ru.ko.*
%{_cross_kmoddir}/kernel/fs/nls/nls_koi8-u.ko.*
%{_cross_kmoddir}/kernel/fs/nls/nls_utf8.ko.*
%{_cross_kmoddir}/kernel/fs/overlayfs/overlay.ko.*
%{_cross_kmoddir}/kernel/fs/pstore/ramoops.ko.*
%{_cross_kmoddir}/kernel/fs/quota/quota_tree.ko.*
%{_cross_kmoddir}/kernel/fs/quota/quota_v2.ko.*
%{_cross_kmoddir}/kernel/fs/smb/client/cifs.ko.*
%{_cross_kmoddir}/kernel/fs/smb/common/cifs_arc4.ko.*
%{_cross_kmoddir}/kernel/fs/smb/common/cifs_md4.ko.*
%{_cross_kmoddir}/kernel/fs/squashfs/squashfs.ko.*
%{_cross_kmoddir}/kernel/fs/udf/udf.ko.*
%{_cross_kmoddir}/kernel/kernel/bpf/preload/bpf_preload.ko.*
%{_cross_kmoddir}/kernel/lib/asn1_encoder.ko.*
%{_cross_kmoddir}/kernel/lib/crc4.ko.*
%{_cross_kmoddir}/kernel/lib/crc7.ko.*
%{_cross_kmoddir}/kernel/lib/crc8.ko.*
%{_cross_kmoddir}/kernel/lib/crc-itu-t.ko.*
%{_cross_kmoddir}/kernel/lib/crypto/libarc4.ko.*
%{_cross_kmoddir}/kernel/lib/crypto/libchacha20poly1305.ko.*
%{_cross_kmoddir}/kernel/lib/crypto/libchacha.ko.*
%{_cross_kmoddir}/kernel/lib/crypto/libcurve25519-generic.ko.*
%{_cross_kmoddir}/kernel/lib/crypto/libcurve25519.ko.*
%{_cross_kmoddir}/kernel/lib/crypto/libdes.ko.*
%{_cross_kmoddir}/kernel/lib/crypto/libpoly1305.ko.*
%{_cross_kmoddir}/kernel/lib/lru_cache.ko.*
%{_cross_kmoddir}/kernel/lib/lz4/lz4_compress.ko.*
%{_cross_kmoddir}/kernel/lib/lz4/lz4_decompress.ko.*
%{_cross_kmoddir}/kernel/lib/lz4/lz4hc_compress.ko.*
%{_cross_kmoddir}/kernel/lib/raid6/raid6_pq.ko.*
%{_cross_kmoddir}/kernel/lib/reed_solomon/reed_solomon.ko.*
%{_cross_kmoddir}/kernel/lib/test_lockup.ko.*
%{_cross_kmoddir}/kernel/lib/ts_bm.ko.*
%{_cross_kmoddir}/kernel/lib/ts_fsm.ko.*
%{_cross_kmoddir}/kernel/lib/ts_kmp.ko.*
%{_cross_kmoddir}/kernel/lib/zstd/zstd_compress.ko.*
%{_cross_kmoddir}/kernel/mm/z3fold.ko.*
%{_cross_kmoddir}/kernel/mm/zsmalloc.ko.*
%{_cross_kmoddir}/kernel/net/8021q/8021q.ko.*
%{_cross_kmoddir}/kernel/net/802/garp.ko.*
%{_cross_kmoddir}/kernel/net/802/mrp.ko.*
%{_cross_kmoddir}/kernel/net/802/p8022.ko.*
%{_cross_kmoddir}/kernel/net/802/psnap.ko.*
%{_cross_kmoddir}/kernel/net/802/stp.ko.*
%{_cross_kmoddir}/kernel/net/bridge/bridge.ko.*
%{_cross_kmoddir}/kernel/net/bridge/br_netfilter.ko.*
%{_cross_kmoddir}/kernel/net/bridge/netfilter/ebt_802_3.ko.*
%{_cross_kmoddir}/kernel/net/bridge/netfilter/ebtable_broute.ko.*
%{_cross_kmoddir}/kernel/net/bridge/netfilter/ebtable_filter.ko.*
%{_cross_kmoddir}/kernel/net/bridge/netfilter/ebtable_nat.ko.*
%{_cross_kmoddir}/kernel/net/bridge/netfilter/ebtables.ko.*
%{_cross_kmoddir}/kernel/net/bridge/netfilter/ebt_among.ko.*
%{_cross_kmoddir}/kernel/net/bridge/netfilter/ebt_arp.ko.*
%{_cross_kmoddir}/kernel/net/bridge/netfilter/ebt_arpreply.ko.*
%{_cross_kmoddir}/kernel/net/bridge/netfilter/ebt_dnat.ko.*
%{_cross_kmoddir}/kernel/net/bridge/netfilter/ebt_ip6.ko.*
%{_cross_kmoddir}/kernel/net/bridge/netfilter/ebt_ip.ko.*
%{_cross_kmoddir}/kernel/net/bridge/netfilter/ebt_limit.ko.*
%{_cross_kmoddir}/kernel/net/bridge/netfilter/ebt_log.ko.*
%{_cross_kmoddir}/kernel/net/bridge/netfilter/ebt_mark.ko.*
%{_cross_kmoddir}/kernel/net/bridge/netfilter/ebt_mark_m.ko.*
%{_cross_kmoddir}/kernel/net/bridge/netfilter/ebt_nflog.ko.*
%{_cross_kmoddir}/kernel/net/bridge/netfilter/ebt_pkttype.ko.*
%{_cross_kmoddir}/kernel/net/bridge/netfilter/ebt_redirect.ko.*
%{_cross_kmoddir}/kernel/net/bridge/netfilter/ebt_snat.ko.*
%{_cross_kmoddir}/kernel/net/bridge/netfilter/ebt_stp.ko.*
%{_cross_kmoddir}/kernel/net/bridge/netfilter/ebt_vlan.ko.*
%{_cross_kmoddir}/kernel/net/bridge/netfilter/nft_reject_bridge.ko.*
%{_cross_kmoddir}/kernel/net/ceph/libceph.ko.*
%{_cross_kmoddir}/kernel/net/core/failover.ko.*
%{_cross_kmoddir}/kernel/net/core/selftests.ko.*
%{_cross_kmoddir}/kernel/net/dns_resolver/dns_resolver.ko.*
%{_cross_kmoddir}/kernel/net/ife/ife.ko.*
%{_cross_kmoddir}/kernel/net/ipv4/ah4.ko.*
%{_cross_kmoddir}/kernel/net/ipv4/esp4.ko.*
%{_cross_kmoddir}/kernel/net/ipv4/esp4_offload.ko.*
%{_cross_kmoddir}/kernel/net/ipv4/fou.ko.*
%{_cross_kmoddir}/kernel/net/ipv4/gre.ko.*
%{_cross_kmoddir}/kernel/net/ipv4/inet_diag.ko.*
%{_cross_kmoddir}/kernel/net/ipv4/ipcomp.ko.*
%{_cross_kmoddir}/kernel/net/ipv4/ip_gre.ko.*
%{_cross_kmoddir}/kernel/net/ipv4/ipip.ko.*
%{_cross_kmoddir}/kernel/net/ipv4/ip_tunnel.ko.*
%{_cross_kmoddir}/kernel/net/ipv4/ip_vti.ko.*
%{_cross_kmoddir}/kernel/net/ipv4/netfilter/arptable_filter.ko.*
%{_cross_kmoddir}/kernel/net/ipv4/netfilter/arp_tables.ko.*
%{_cross_kmoddir}/kernel/net/ipv4/netfilter/arpt_mangle.ko.*
%{_cross_kmoddir}/kernel/net/ipv4/netfilter/iptable_filter.ko.*
%{_cross_kmoddir}/kernel/net/ipv4/netfilter/iptable_mangle.ko.*
%{_cross_kmoddir}/kernel/net/ipv4/netfilter/iptable_nat.ko.*
%{_cross_kmoddir}/kernel/net/ipv4/netfilter/iptable_raw.ko.*
%{_cross_kmoddir}/kernel/net/ipv4/netfilter/iptable_security.ko.*
%{_cross_kmoddir}/kernel/net/ipv4/netfilter/ipt_ah.ko.*
%{_cross_kmoddir}/kernel/net/ipv4/netfilter/ipt_CLUSTERIP.ko.*
%{_cross_kmoddir}/kernel/net/ipv4/netfilter/ipt_ECN.ko.*
%{_cross_kmoddir}/kernel/net/ipv4/netfilter/ipt_REJECT.ko.*
%{_cross_kmoddir}/kernel/net/ipv4/netfilter/ipt_rpfilter.ko.*
%{_cross_kmoddir}/kernel/net/ipv4/netfilter/ipt_SYNPROXY.ko.*
%{_cross_kmoddir}/kernel/net/ipv4/netfilter/nf_defrag_ipv4.ko.*
%{_cross_kmoddir}/kernel/net/ipv4/netfilter/nf_dup_ipv4.ko.*
%{_cross_kmoddir}/kernel/net/ipv4/netfilter/nf_nat_h323.ko.*
%{_cross_kmoddir}/kernel/net/ipv4/netfilter/nf_nat_pptp.ko.*
%{_cross_kmoddir}/kernel/net/ipv4/netfilter/nf_nat_snmp_basic.ko.*
%{_cross_kmoddir}/kernel/net/ipv4/netfilter/nf_reject_ipv4.ko.*
%{_cross_kmoddir}/kernel/net/ipv4/netfilter/nf_socket_ipv4.ko.*
%{_cross_kmoddir}/kernel/net/ipv4/netfilter/nft_dup_ipv4.ko.*
%{_cross_kmoddir}/kernel/net/ipv4/netfilter/nft_fib_ipv4.ko.*
%{_cross_kmoddir}/kernel/net/ipv4/netfilter/nf_tproxy_ipv4.ko.*
%{_cross_kmoddir}/kernel/net/ipv4/netfilter/nft_reject_ipv4.ko.*
%{_cross_kmoddir}/kernel/net/ipv4/raw_diag.ko.*
%{_cross_kmoddir}/kernel/net/ipv4/tcp_bbr.ko.*
%{_cross_kmoddir}/kernel/net/ipv4/tcp_bic.ko.*
%{_cross_kmoddir}/kernel/net/ipv4/tcp_dctcp.ko.*
%{_cross_kmoddir}/kernel/net/ipv4/tcp_diag.ko.*
%{_cross_kmoddir}/kernel/net/ipv4/tcp_highspeed.ko.*
%{_cross_kmoddir}/kernel/net/ipv4/tcp_htcp.ko.*
%{_cross_kmoddir}/kernel/net/ipv4/tcp_hybla.ko.*
%{_cross_kmoddir}/kernel/net/ipv4/tcp_illinois.ko.*
%{_cross_kmoddir}/kernel/net/ipv4/tcp_lp.ko.*
%{_cross_kmoddir}/kernel/net/ipv4/tcp_scalable.ko.*
%{_cross_kmoddir}/kernel/net/ipv4/tcp_vegas.ko.*
%{_cross_kmoddir}/kernel/net/ipv4/tcp_veno.ko.*
%{_cross_kmoddir}/kernel/net/ipv4/tcp_westwood.ko.*
%{_cross_kmoddir}/kernel/net/ipv4/tcp_yeah.ko.*
%{_cross_kmoddir}/kernel/net/ipv4/tunnel4.ko.*
%{_cross_kmoddir}/kernel/net/ipv4/udp_diag.ko.*
%{_cross_kmoddir}/kernel/net/ipv4/udp_tunnel.ko.*
%{_cross_kmoddir}/kernel/net/ipv4/xfrm4_tunnel.ko.*
%{_cross_kmoddir}/kernel/net/ipv6/ah6.ko.*
%{_cross_kmoddir}/kernel/net/ipv6/esp6.ko.*
%{_cross_kmoddir}/kernel/net/ipv6/esp6_offload.ko.*
%{_cross_kmoddir}/kernel/net/ipv6/fou6.ko.*
%{_cross_kmoddir}/kernel/net/ipv6/ila/ila.ko.*
%{_cross_kmoddir}/kernel/net/ipv6/ip6_gre.ko.*
%{_cross_kmoddir}/kernel/net/ipv6/ip6_tunnel.ko.*
%{_cross_kmoddir}/kernel/net/ipv6/ip6_udp_tunnel.ko.*
%{_cross_kmoddir}/kernel/net/ipv6/ip6_vti.ko.*
%{_cross_kmoddir}/kernel/net/ipv6/ipcomp6.ko.*
%{_cross_kmoddir}/kernel/net/ipv6/mip6.ko.*
%{_cross_kmoddir}/kernel/net/ipv6/netfilter/ip6table_filter.ko.*
%{_cross_kmoddir}/kernel/net/ipv6/netfilter/ip6table_mangle.ko.*
%{_cross_kmoddir}/kernel/net/ipv6/netfilter/ip6table_nat.ko.*
%{_cross_kmoddir}/kernel/net/ipv6/netfilter/ip6table_raw.ko.*
%{_cross_kmoddir}/kernel/net/ipv6/netfilter/ip6table_security.ko.*
%{_cross_kmoddir}/kernel/net/ipv6/netfilter/ip6t_ah.ko.*
%{_cross_kmoddir}/kernel/net/ipv6/netfilter/ip6t_eui64.ko.*
%{_cross_kmoddir}/kernel/net/ipv6/netfilter/ip6t_frag.ko.*
%{_cross_kmoddir}/kernel/net/ipv6/netfilter/ip6t_hbh.ko.*
%{_cross_kmoddir}/kernel/net/ipv6/netfilter/ip6t_ipv6header.ko.*
%{_cross_kmoddir}/kernel/net/ipv6/netfilter/ip6t_mh.ko.*
%{_cross_kmoddir}/kernel/net/ipv6/netfilter/ip6t_REJECT.ko.*
%{_cross_kmoddir}/kernel/net/ipv6/netfilter/ip6t_rpfilter.ko.*
%{_cross_kmoddir}/kernel/net/ipv6/netfilter/ip6t_rt.ko.*
%{_cross_kmoddir}/kernel/net/ipv6/netfilter/ip6t_srh.ko.*
%{_cross_kmoddir}/kernel/net/ipv6/netfilter/ip6t_SYNPROXY.ko.*
%{_cross_kmoddir}/kernel/net/ipv6/netfilter/nf_defrag_ipv6.ko.*
%{_cross_kmoddir}/kernel/net/ipv6/netfilter/nf_dup_ipv6.ko.*
%{_cross_kmoddir}/kernel/net/ipv6/netfilter/nf_reject_ipv6.ko.*
%{_cross_kmoddir}/kernel/net/ipv6/netfilter/nf_socket_ipv6.ko.*
%{_cross_kmoddir}/kernel/net/ipv6/netfilter/nft_dup_ipv6.ko.*
%{_cross_kmoddir}/kernel/net/ipv6/netfilter/nft_fib_ipv6.ko.*
%{_cross_kmoddir}/kernel/net/ipv6/netfilter/nf_tproxy_ipv6.ko.*
%{_cross_kmoddir}/kernel/net/ipv6/netfilter/nft_reject_ipv6.ko.*
%{_cross_kmoddir}/kernel/net/ipv6/sit.ko.*
%{_cross_kmoddir}/kernel/net/ipv6/tunnel6.ko.*
%{_cross_kmoddir}/kernel/net/ipv6/xfrm6_tunnel.ko.*
%{_cross_kmoddir}/kernel/net/key/af_key.ko.*
%{_cross_kmoddir}/kernel/net/llc/llc.ko.*
%{_cross_kmoddir}/kernel/net/mpls/mpls_gso.ko.*
%{_cross_kmoddir}/kernel/net/mpls/mpls_iptunnel.ko.*
%{_cross_kmoddir}/kernel/net/mpls/mpls_router.ko.*
%{_cross_kmoddir}/kernel/net/mptcp/mptcp_diag.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/ipset/ip_set_bitmap_ip.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/ipset/ip_set_bitmap_ipmac.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/ipset/ip_set_bitmap_port.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/ipset/ip_set_hash_ip.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/ipset/ip_set_hash_ipmac.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/ipset/ip_set_hash_ipmark.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/ipset/ip_set_hash_ipportip.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/ipset/ip_set_hash_ipport.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/ipset/ip_set_hash_ipportnet.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/ipset/ip_set_hash_mac.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/ipset/ip_set_hash_netiface.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/ipset/ip_set_hash_net.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/ipset/ip_set_hash_netnet.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/ipset/ip_set_hash_netport.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/ipset/ip_set_hash_netportnet.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/ipset/ip_set.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/ipset/ip_set_list_set.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/ipvs/ip_vs_dh.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/ipvs/ip_vs_fo.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/ipvs/ip_vs_ftp.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/ipvs/ip_vs.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/ipvs/ip_vs_lblc.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/ipvs/ip_vs_lblcr.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/ipvs/ip_vs_lc.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/ipvs/ip_vs_mh.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/ipvs/ip_vs_nq.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/ipvs/ip_vs_ovf.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/ipvs/ip_vs_pe_sip.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/ipvs/ip_vs_rr.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/ipvs/ip_vs_sed.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/ipvs/ip_vs_sh.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/ipvs/ip_vs_wlc.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/ipvs/ip_vs_wrr.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nf_conncount.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nf_conntrack_amanda.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nf_conntrack_broadcast.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nf_conntrack_ftp.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nf_conntrack_h323.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nf_conntrack_irc.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nf_conntrack.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nf_conntrack_netbios_ns.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nf_conntrack_netlink.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nf_conntrack_pptp.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nf_conntrack_sane.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nf_conntrack_sip.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nf_conntrack_snmp.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nf_conntrack_tftp.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nf_dup_netdev.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nf_flow_table_inet.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nf_flow_table.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nf_log_syslog.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nf_nat_amanda.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nf_nat_ftp.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nf_nat_irc.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nf_nat.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nf_nat_sip.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nf_nat_tftp.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nfnetlink_acct.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nfnetlink_cthelper.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nfnetlink_cttimeout.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nfnetlink.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nfnetlink_log.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nfnetlink_osf.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nfnetlink_queue.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nf_synproxy_core.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nf_tables.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nft_chain_nat.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nft_compat.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nft_connlimit.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nft_ct.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nft_dup_netdev.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nft_fib_inet.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nft_fib.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nft_fib_netdev.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nft_flow_offload.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nft_fwd_netdev.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nft_hash.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nft_limit.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nft_log.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nft_masq.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nft_nat.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nft_numgen.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nft_objref.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nft_osf.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nft_queue.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nft_quota.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nft_redir.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nft_reject_inet.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nft_reject.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nft_socket.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nft_synproxy.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nft_tproxy.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nft_tunnel.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/nft_xfrm.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_addrtype.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_AUDIT.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_bpf.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_cgroup.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_CHECKSUM.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_CLASSIFY.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_cluster.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_comment.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_connbytes.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_connlabel.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_connlimit.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_connmark.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_CONNSECMARK.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_conntrack.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_cpu.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_CT.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_devgroup.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_dscp.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_DSCP.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_ecn.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_esp.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_hashlimit.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_helper.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_hl.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_HL.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_HMARK.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_IDLETIMER.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_ipcomp.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_iprange.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_ipvs.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_l2tp.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_length.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_limit.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_LOG.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_mac.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_mark.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_MASQUERADE.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_multiport.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_nat.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_NETMAP.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_nfacct.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_NFLOG.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_NFQUEUE.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_osf.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_owner.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_physdev.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_pkttype.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_policy.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_quota.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_rateest.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_RATEEST.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_realm.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_recent.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_REDIRECT.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_sctp.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_SECMARK.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_set.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_socket.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_state.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_statistic.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_string.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_tcpmss.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_TCPMSS.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_TCPOPTSTRIP.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_TEE.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_time.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_TPROXY.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_TRACE.ko.*
%{_cross_kmoddir}/kernel/net/netfilter/xt_u32.ko.*
%{_cross_kmoddir}/kernel/net/nsh/nsh.ko.*
%{_cross_kmoddir}/kernel/net/openvswitch/openvswitch.ko.*
%{_cross_kmoddir}/kernel/net/openvswitch/vport-geneve.ko.*
%{_cross_kmoddir}/kernel/net/openvswitch/vport-gre.ko.*
%{_cross_kmoddir}/kernel/net/openvswitch/vport-vxlan.ko.*
%{_cross_kmoddir}/kernel/net/packet/af_packet_diag.ko.*
%{_cross_kmoddir}/kernel/net/psample/psample.ko.*
%{_cross_kmoddir}/kernel/net/sched/act_bpf.ko.*
%{_cross_kmoddir}/kernel/net/sched/act_connmark.ko.*
%{_cross_kmoddir}/kernel/net/sched/act_csum.ko.*
%{_cross_kmoddir}/kernel/net/sched/act_gact.ko.*
%{_cross_kmoddir}/kernel/net/sched/act_ipt.ko.*
%{_cross_kmoddir}/kernel/net/sched/act_mirred.ko.*
%{_cross_kmoddir}/kernel/net/sched/act_nat.ko.*
%{_cross_kmoddir}/kernel/net/sched/act_pedit.ko.*
%{_cross_kmoddir}/kernel/net/sched/act_police.ko.*
%{_cross_kmoddir}/kernel/net/sched/act_sample.ko.*
%{_cross_kmoddir}/kernel/net/sched/act_simple.ko.*
%{_cross_kmoddir}/kernel/net/sched/act_skbedit.ko.*
%{_cross_kmoddir}/kernel/net/sched/act_vlan.ko.*
%{_cross_kmoddir}/kernel/net/sched/cls_basic.ko.*
%{_cross_kmoddir}/kernel/net/sched/cls_bpf.ko.*
%{_cross_kmoddir}/kernel/net/sched/cls_cgroup.ko.*
%{_cross_kmoddir}/kernel/net/sched/cls_flower.ko.*
%{_cross_kmoddir}/kernel/net/sched/cls_flow.ko.*
%{_cross_kmoddir}/kernel/net/sched/cls_fw.ko.*
%{_cross_kmoddir}/kernel/net/sched/cls_route.ko.*
%{_cross_kmoddir}/kernel/net/sched/cls_u32.ko.*
%{_cross_kmoddir}/kernel/net/sched/em_cmp.ko.*
%{_cross_kmoddir}/kernel/net/sched/em_ipset.ko.*
%{_cross_kmoddir}/kernel/net/sched/em_ipt.ko.*
%{_cross_kmoddir}/kernel/net/sched/em_meta.ko.*
%{_cross_kmoddir}/kernel/net/sched/em_nbyte.ko.*
%{_cross_kmoddir}/kernel/net/sched/em_text.ko.*
%{_cross_kmoddir}/kernel/net/sched/em_u32.ko.*
%{_cross_kmoddir}/kernel/net/sched/sch_cbs.ko.*
%{_cross_kmoddir}/kernel/net/sched/sch_choke.ko.*
%{_cross_kmoddir}/kernel/net/sched/sch_codel.ko.*
%{_cross_kmoddir}/kernel/net/sched/sch_drr.ko.*
%{_cross_kmoddir}/kernel/net/sched/sch_fq_codel.ko.*
%{_cross_kmoddir}/kernel/net/sched/sch_fq.ko.*
%{_cross_kmoddir}/kernel/net/sched/sch_gred.ko.*
%{_cross_kmoddir}/kernel/net/sched/sch_hfsc.ko.*
%{_cross_kmoddir}/kernel/net/sched/sch_hhf.ko.*
%{_cross_kmoddir}/kernel/net/sched/sch_htb.ko.*
%{_cross_kmoddir}/kernel/net/sched/sch_ingress.ko.*
%{_cross_kmoddir}/kernel/net/sched/sch_mqprio.ko.*
%{_cross_kmoddir}/kernel/net/sched/sch_multiq.ko.*
%{_cross_kmoddir}/kernel/net/sched/sch_netem.ko.*
%{_cross_kmoddir}/kernel/net/sched/sch_pie.ko.*
%{_cross_kmoddir}/kernel/net/sched/sch_plug.ko.*
%{_cross_kmoddir}/kernel/net/sched/sch_prio.ko.*
%{_cross_kmoddir}/kernel/net/sched/sch_qfq.ko.*
%{_cross_kmoddir}/kernel/net/sched/sch_red.ko.*
%{_cross_kmoddir}/kernel/net/sched/sch_sfb.ko.*
%{_cross_kmoddir}/kernel/net/sched/sch_sfq.ko.*
%{_cross_kmoddir}/kernel/net/sched/sch_tbf.ko.*
%{_cross_kmoddir}/kernel/net/sched/sch_teql.ko.*
%{_cross_kmoddir}/kernel/net/sctp/sctp_diag.ko.*
%{_cross_kmoddir}/kernel/net/sctp/sctp.ko.*
%{_cross_kmoddir}/kernel/net/sunrpc/auth_gss/auth_rpcgss.ko.*
%{_cross_kmoddir}/kernel/net/sunrpc/auth_gss/rpcsec_gss_krb5.ko.*
%{_cross_kmoddir}/kernel/net/sunrpc/sunrpc.ko.*
%{_cross_kmoddir}/kernel/net/tls/tls.ko.*
%{_cross_kmoddir}/kernel/net/unix/unix_diag.ko.*
%{_cross_kmoddir}/kernel/net/vmw_vsock/vmw_vsock_virtio_transport_common.ko.*
%{_cross_kmoddir}/kernel/net/vmw_vsock/vmw_vsock_virtio_transport.ko.*
%{_cross_kmoddir}/kernel/net/vmw_vsock/vsock_diag.ko.*
%{_cross_kmoddir}/kernel/net/vmw_vsock/vsock.ko.*
%{_cross_kmoddir}/kernel/net/vmw_vsock/vsock_loopback.ko.*
%if "%{_cross_arch}" == "x86_64"
%{_cross_kmoddir}/kernel/net/vmw_vsock/hv_sock.ko.*
%{_cross_kmoddir}/kernel/net/vmw_vsock/vmw_vsock_vmci_transport.ko.*
%endif
%{_cross_kmoddir}/kernel/net/xfrm/xfrm_algo.ko.*
%{_cross_kmoddir}/kernel/net/xfrm/xfrm_ipcomp.ko.*
%{_cross_kmoddir}/kernel/net/xfrm/xfrm_user.ko.*
%{_cross_kmoddir}/kernel/security/keys/encrypted-keys/encrypted-keys.ko.*
%{_cross_kmoddir}/kernel/security/keys/trusted-keys/trusted.ko.*
%if "%{_cross_arch}" == "x86_64"
%{_cross_kmoddir}/kernel/virt/lib/irqbypass.ko.*
%endif

%if "%{_cross_arch}" == "x86_64"
%{_cross_kmoddir}/kernel/drivers/infiniband/hw/usnic/usnic_verbs.ko.gz
%endif
%{_cross_kmoddir}/kernel/drivers/net/ethernet/amd/xgbe/amd-xgbe.ko.gz
%{_cross_kmoddir}/kernel/drivers/net/ethernet/broadcom/bnx2x/bnx2x.ko.gz
%{_cross_kmoddir}/kernel/drivers/net/ethernet/broadcom/bnxt/bnxt_en.ko.gz
%{_cross_kmoddir}/kernel/drivers/net/ethernet/broadcom/tg3.ko.gz
%{_cross_kmoddir}/kernel/drivers/net/ethernet/chelsio/cxgb4/cxgb4.ko.gz
%{_cross_kmoddir}/kernel/drivers/net/ethernet/chelsio/cxgb4vf/cxgb4vf.ko.gz
%{_cross_kmoddir}/kernel/drivers/net/ethernet/cisco/enic/enic.ko.gz
%{_cross_kmoddir}/kernel/drivers/net/ethernet/emulex/benet/be2net.ko.gz
%{_cross_kmoddir}/kernel/drivers/net/ethernet/huawei/hinic/hinic.ko.gz
%{_cross_kmoddir}/kernel/drivers/net/ethernet/intel/fm10k/fm10k.ko.gz
%{_cross_kmoddir}/kernel/drivers/net/ethernet/intel/i40e/i40e.ko.gz
%{_cross_kmoddir}/kernel/drivers/net/ethernet/intel/ice/ice.ko.gz
%{_cross_kmoddir}/kernel/drivers/net/ethernet/intel/igbvf/igbvf.ko.gz
%{_cross_kmoddir}/kernel/drivers/net/ethernet/intel/ixgb/ixgb.ko.gz
%{_cross_kmoddir}/kernel/drivers/net/ethernet/intel/ixgbe/ixgbe.ko.gz
%{_cross_kmoddir}/kernel/drivers/net/ethernet/myricom/myri10ge/myri10ge.ko.gz
%{_cross_kmoddir}/kernel/drivers/net/ethernet/pensando/ionic/ionic.ko.gz
%{_cross_kmoddir}/kernel/drivers/net/ethernet/qlogic/qed/qed.ko.gz
%{_cross_kmoddir}/kernel/drivers/net/ethernet/qlogic/qede/qede.ko.gz
%{_cross_kmoddir}/kernel/drivers/net/ethernet/qlogic/qlcnic/qlcnic.ko.gz
%{_cross_kmoddir}/kernel/drivers/net/ethernet/sfc/falcon/sfc-falcon.ko.gz
%{_cross_kmoddir}/kernel/drivers/net/ethernet/sfc/sfc.ko.gz
%{_cross_kmoddir}/kernel/drivers/net/mdio.ko.gz
%{_cross_kmoddir}/kernel/drivers/scsi/snic/snic.ko.gz

%changelog
