%global _cross_first_party 1

Name: %{_cross_os}release
Version: 0.0
Release: 0%{?dist}
Summary: Bottlerocket release
License: Apache-2.0 OR MIT

Source10: hosts
Source11: nsswitch.conf
Source97: release-sysctl.conf
Source98: release-systemd-system.conf
Source99: release-tmpfiles.conf

Source200: motd.template
Source201: proxy-env

Source1000: eth0.xml
Source1001: multi-user.target
Source1002: configured.target
Source1003: preconfigured.target
Source1004: activate-configured.service
Source1005: activate-multi-user.service

# Mounts for writable local storage.
Source1006: prepare-local.service
Source1007: var.mount
Source1008: opt.mount
Source1009: var-lib-bottlerocket.mount
Source1010: etc-cni.mount

# CD-ROM mount & associated udev rules
Source1015: media-cdrom.mount
Source1016: mount-cdrom.rules

# Mounts that require build-time edits.
Source1020: var-lib-kernel-devel-lower.mount.in
Source1021: usr-src-kernels.mount.in
Source1022: usr-share-licenses.mount.in

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
Requires: %{_cross_os}selinux-policy
Requires: %{_cross_os}policycoreutils
Requires: %{_cross_os}procps
Requires: %{_cross_os}systemd
Requires: %{_cross_os}util-linux
Requires: %{_cross_os}wicked
Requires: %{_cross_os}os

%description
%{summary}.

%prep

%build

%install
install -d %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}
install -p -m 0644 %{S:10} %{S:11} %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}

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
  %{S:1006} %{S:1007} %{S:1008} %{S:1009} %{S:1010} %{S:1015} \
  %{buildroot}%{_cross_unitdir}

LOWERPATH=$(systemd-escape --path %{_cross_sharedstatedir}/kernel-devel/lower)
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

install -d %{buildroot}%{_cross_udevrulesdir}
install -p -m 0644 %{S:1016} %{buildroot}%{_cross_udevrulesdir}/61-mount-cdrom.rules

ln -s %{_cross_unitdir}/preconfigured.target %{buildroot}%{_cross_unitdir}/default.target

%files
%{_cross_factorydir}%{_cross_sysconfdir}/hosts
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
%{_cross_unitdir}/prepare-local.service
%{_cross_unitdir}/var.mount
%{_cross_unitdir}/opt.mount
%{_cross_unitdir}/etc-cni.mount
%{_cross_unitdir}/media-cdrom.mount
%{_cross_unitdir}/*-lower.mount
%{_cross_unitdir}/*-kernels.mount
%{_cross_unitdir}/*-licenses.mount
%{_cross_unitdir}/var-lib-bottlerocket.mount
%dir %{_cross_templatedir}
%{_cross_templatedir}/motd
%{_cross_templatedir}/proxy-env
%{_cross_udevrulesdir}/61-mount-cdrom.rules

%changelog
