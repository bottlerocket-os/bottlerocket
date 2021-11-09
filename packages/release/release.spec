%global _cross_first_party 1

Name: %{_cross_os}release
Version: 0.0
Release: 0%{?dist}
Summary: Bottlerocket release
License: Apache-2.0 OR MIT

Source11: nsswitch.conf
Source97: release-sysctl.conf
Source98: release-systemd-system.conf
Source99: release-tmpfiles.conf

Source200: motd.template
Source201: proxy-env
Source202: hostname-env
Source203: hosts.template

Source1000: eth0.xml
Source1001: multi-user.target
Source1002: configured.target
Source1003: preconfigured.target
Source1004: activate-configured.service
Source1005: activate-multi-user.service
Source1011: set-hostname.service

# Mounts for writable local storage.
Source1006: var.mount
Source1007: opt.mount
Source1008: var-lib-bottlerocket.mount
Source1009: etc-cni.mount
Source1010: mnt.mount

# CD-ROM mount & associated udev rules
Source1015: media-cdrom.mount
Source1016: mount-cdrom.rules

# Mounts that require build-time edits.
Source1020: var-lib-kernel-devel-lower.mount.in
Source1021: usr-src-kernels.mount.in
Source1022: usr-share-licenses.mount.in

# Mounts that require helper programs
Source1040: prepare-boot.service
Source1041: prepare-local.service

# Services for kdump support
Source1060: capture-kernel-dump.service
Source1061: disable-kexec-load.service
Source1062: load-crash-kernel.service

# systemd cgroups/slices
Source1080: runtime.slice

# Drop-in units to override defaults
Source1100: systemd-tmpfiles-setup-service-debug.conf

BuildArch: noarch
Requires: %{_cross_os}acpid
Requires: %{_cross_os}audit
Requires: %{_cross_os}ca-certificates
Requires: %{_cross_os}chrony
Requires: %{_cross_os}conntrack-tools
Requires: %{_cross_os}containerd
Requires: %{_cross_os}coreutils
Requires: %{_cross_os}dbus-broker
Requires: %{_cross_os}e2fsprogs
Requires: %{_cross_os}libgcc
Requires: %{_cross_os}libstd-rust
Requires: %{_cross_os}filesystem
Requires: %{_cross_os}findutils
Requires: %{_cross_os}glibc
Requires: %{_cross_os}grep
Requires: %{_cross_os}grub
Requires: %{_cross_os}host-ctr
Requires: %{_cross_os}iproute
Requires: %{_cross_os}iptables
Requires: %{_cross_os}kexec-tools
Requires: %{_cross_os}makedumpfile
Requires: %{_cross_os}os
Requires: %{_cross_os}policycoreutils
Requires: %{_cross_os}procps
Requires: %{_cross_os}selinux-policy
Requires: %{_cross_os}systemd
Requires: %{_cross_os}util-linux
Requires: %{_cross_os}wicked

%description
%{summary}.

%prep

%build

%install
install -d %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}
install -p -m 0644 %{S:11} %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}

install -d %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}/wicked/ifconfig
install -p -m 0644 %{S:1000} %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}/wicked/ifconfig

install -d %{buildroot}%{_cross_sysctldir}
install -p -m 0644 %{S:97} %{buildroot}%{_cross_sysctldir}/80-release.conf

install -d %{buildroot}%{_cross_libdir}/systemd/system.conf.d
install -p -m 0644 %{S:98} %{buildroot}%{_cross_libdir}/systemd/system.conf.d/80-release.conf

install -d %{buildroot}%{_cross_tmpfilesdir}
install -p -m 0644 %{S:99} %{buildroot}%{_cross_tmpfilesdir}/release.conf

cat >%{buildroot}%{_cross_libdir}/os-release <<EOF
NAME=Bottlerocket
ID=bottlerocket
EOF

install -d %{buildroot}%{_cross_unitdir}
install -p -m 0644 \
  %{S:1001} %{S:1002} %{S:1003} %{S:1004} %{S:1005} \
  %{S:1006} %{S:1007} %{S:1008} %{S:1009} %{S:1010} %{S:1011} \
  %{S:1015} %{S:1040} %{S:1041} %{S:1060} %{S:1061} %{S:1062} %{S:1080} \
  %{buildroot}%{_cross_unitdir}

install -d %{buildroot}%{_cross_unitdir}/systemd-tmpfiles-setup.service.d
install -p -m 0644 %{S:1100} \
  %{buildroot}%{_cross_unitdir}/systemd-tmpfiles-setup.service.d/00-debug.conf

LOWERPATH=$(systemd-escape --path %{_cross_sharedstatedir}/kernel-devel/.overlay/lower)
sed -e 's|PREFIX|%{_cross_prefix}|' %{S:1020} > ${LOWERPATH}.mount
install -p -m 0644 ${LOWERPATH}.mount %{buildroot}%{_cross_unitdir}

# Mounting on usr/src/kernels requires using the real path: %{_cross_usrsrc}/kernels
KERNELPATH=$(systemd-escape --path %{_cross_usrsrc}/kernels)
sed -e 's|PREFIX|%{_cross_prefix}|' %{S:1021} > ${KERNELPATH}.mount
install -p -m 0644 ${KERNELPATH}.mount %{buildroot}%{_cross_unitdir}

# Mounting on usr/share/licenses requires using the real path: %{_cross_datadir}/licenses
LICENSEPATH=$(systemd-escape --path %{_cross_licensedir})
sed -e 's|PREFIX|%{_cross_prefix}|' %{S:1022} > ${LICENSEPATH}.mount
install -p -m 0644 ${LICENSEPATH}.mount %{buildroot}%{_cross_unitdir}

install -d %{buildroot}%{_cross_templatedir}
install -p -m 0644 %{S:200} %{buildroot}%{_cross_templatedir}/motd
install -p -m 0644 %{S:201} %{buildroot}%{_cross_templatedir}/proxy-env
install -p -m 0644 %{S:202} %{buildroot}%{_cross_templatedir}/hostname-env
install -p -m 0644 %{S:203} %{buildroot}%{_cross_templatedir}/hosts

install -d %{buildroot}%{_cross_udevrulesdir}
install -p -m 0644 %{S:1016} %{buildroot}%{_cross_udevrulesdir}/61-mount-cdrom.rules

ln -s %{_cross_unitdir}/preconfigured.target %{buildroot}%{_cross_unitdir}/default.target

%files
%{_cross_factorydir}%{_cross_sysconfdir}/nsswitch.conf
%{_cross_factorydir}%{_cross_sysconfdir}/wicked/ifconfig/eth0.xml
%{_cross_sysctldir}/80-release.conf
%{_cross_tmpfilesdir}/release.conf
%{_cross_libdir}/os-release
%{_cross_libdir}/systemd/system.conf.d/80-release.conf
%{_cross_unitdir}/configured.target
%{_cross_unitdir}/preconfigured.target
%{_cross_unitdir}/multi-user.target
%{_cross_unitdir}/default.target
%{_cross_unitdir}/activate-configured.service
%{_cross_unitdir}/activate-multi-user.service
%{_cross_unitdir}/disable-kexec-load.service
%{_cross_unitdir}/capture-kernel-dump.service
%{_cross_unitdir}/load-crash-kernel.service
%{_cross_unitdir}/prepare-boot.service
%{_cross_unitdir}/prepare-local.service
%{_cross_unitdir}/var.mount
%{_cross_unitdir}/opt.mount
%{_cross_unitdir}/mnt.mount
%{_cross_unitdir}/etc-cni.mount
%{_cross_unitdir}/media-cdrom.mount
%{_cross_unitdir}/*-lower.mount
%{_cross_unitdir}/*-kernels.mount
%{_cross_unitdir}/*-licenses.mount
%{_cross_unitdir}/var-lib-bottlerocket.mount
%{_cross_unitdir}/runtime.slice
%{_cross_unitdir}/set-hostname.service
%dir %{_cross_unitdir}/systemd-tmpfiles-setup.service.d
%{_cross_unitdir}/systemd-tmpfiles-setup.service.d/00-debug.conf
%dir %{_cross_templatedir}
%{_cross_templatedir}/motd
%{_cross_templatedir}/proxy-env
%{_cross_templatedir}/hostname-env
%{_cross_templatedir}/hosts
%{_cross_udevrulesdir}/61-mount-cdrom.rules

%changelog
