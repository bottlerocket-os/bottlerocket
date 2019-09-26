# To include a shell in Thar, set this to bcond_without.
%bcond_with shell # without

Name: %{_cross_os}release
Version: 0.1.2
Release: 1%{?dist}
Summary: Thar release
License: Public Domain

Source1: login

Source10: hosts
Source11: nsswitch.conf
Source98: release-sysctl.conf
Source99: release-tmpfiles.conf

# FIXME What should own system-level file templates?
Source200: hostname.template
Source201: host-containers-systemd-unit-admin.template
Source202: host-containers-systemd-unit-control.template

Source1000: eth0.xml
Source1002: configured.target
Source1003: host-containerd.service
Source1004: host-containerd-config.toml
Source1006: prepare-local.service
Source1007: var.mount
Source1008: opt.mount
Source1009: prepare-var-lib-thar.service
Source1010: var-lib-thar.mount

BuildArch: noarch
Requires: %{_cross_os}apiclient
Requires: %{_cross_os}apiserver
%if %{with shell}
Requires: %{_cross_os}bash
%endif
Requires: %{_cross_os}ca-certificates
Requires: %{_cross_os}chrony
Requires: %{_cross_os}coreutils
Requires: %{_cross_os}dbus-broker
Requires: %{_cross_os}filesystem
Requires: %{_cross_os}growpart
Requires: %{_cross_os}grub
Requires: %{_cross_os}iproute
Requires: %{_cross_os}kernel
Requires: %{_cross_os}kernel-modules
Requires: %{_cross_os}bork
%if %{without shell}
Requires: %{_cross_os}login
%endif
Requires: %{_cross_os}moondog
Requires: %{_cross_os}netdog
Requires: %{_cross_os}signpost
Requires: %{_cross_os}sundog
Requires: %{_cross_os}pluto
Requires: %{_cross_os}storewolf
Requires: %{_cross_os}servicedog
Requires: %{_cross_os}settings-committer
Requires: %{_cross_os}systemd
Requires: %{_cross_os}thar-be-settings
Requires: %{_cross_os}migration
Requires: %{_cross_os}updog
Requires: %{_cross_os}util-linux
Requires: %{_cross_os}preinit
Requires: %{_cross_os}wicked
Requires: %{_cross_os}host-containers

%description
%{summary}.

%prep

%build

%install

%if %{with shell}
install -d %{buildroot}%{_cross_bindir}
install -p -m 0755 %{S:1} %{buildroot}%{_cross_bindir}
%endif

install -d %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}
install -p -m 0644 %{S:10} %{S:11} %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}

install -d %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}/wicked/ifconfig
install -p -m 0644 %{S:1000} %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}/wicked/ifconfig

install -d %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}/host-containerd
install -p -m 0644 %{S:1004} %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}/host-containerd/config.toml

install -d %{buildroot}%{_cross_sysctldir}
install -p -m 0644 %{S:98} %{buildroot}%{_cross_sysctldir}/80-release.conf

install -d %{buildroot}%{_cross_tmpfilesdir}
install -p -m 0644 %{S:99} %{buildroot}%{_cross_tmpfilesdir}/release.conf

cat >%{buildroot}%{_cross_libdir}/os-release <<EOF
NAME=Thar
PRETTY_NAME="Thar, The Operating System"
ID=thar
VERSION_ID=%{version}
EOF

install -d %{buildroot}%{_cross_unitdir}
install -p -m 0644 %{S:1002} %{S:1003} %{S:1006} %{S:1007} %{S:1008} %{S:1009} %{S:1010} %{buildroot}%{_cross_unitdir}

install -d %{buildroot}%{_cross_templatedir}
install -p -m 0644 %{S:200} %{buildroot}%{_cross_templatedir}/hostname
install -p -m 0644 %{S:201} %{buildroot}%{_cross_templatedir}/host-containers-systemd-unit-admin
install -p -m 0644 %{S:202} %{buildroot}%{_cross_templatedir}/host-containers-systemd-unit-control

%files
%if %{with shell}
%{_cross_bindir}/login
%endif
%{_cross_factorydir}%{_cross_sysconfdir}/hosts
%{_cross_factorydir}%{_cross_sysconfdir}/nsswitch.conf
%{_cross_factorydir}%{_cross_sysconfdir}/wicked/ifconfig/eth0.xml
%{_cross_factorydir}%{_cross_sysconfdir}/host-containerd/config.toml
%{_cross_sysctldir}/80-release.conf
%{_cross_tmpfilesdir}/release.conf
%{_cross_libdir}/os-release
%{_cross_unitdir}/configured.target
%{_cross_unitdir}/host-containerd.service
%{_cross_unitdir}/prepare-local.service
%{_cross_unitdir}/prepare-var-lib-thar.service
%{_cross_unitdir}/var.mount
%{_cross_unitdir}/opt.mount
%{_cross_unitdir}/var-lib-thar.mount
%dir %{_cross_templatedir}
%{_cross_templatedir}/hostname
%{_cross_templatedir}/host-containers-systemd-unit-admin
%{_cross_templatedir}/host-containers-systemd-unit-control

%changelog
