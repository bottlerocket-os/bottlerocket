Name: %{_cross_os}release
Version: 1
Release: 1%{?dist}
Summary: Thar release
License: Public Domain

Source1: login

Source10: hosts
Source11: nsswitch.conf
Source99: release.conf

# FIXME What should own system-level file templates?
Source200: hostname.template

Source1000: eth0.xml
Source1001: var-lib-thar.mount
Source1002: configured.target

BuildArch: noarch
Requires: %{_cross_os}apiclient
Requires: %{_cross_os}apiserver
Requires: %{_cross_os}bash
Requires: %{_cross_os}ca-certificates
Requires: %{_cross_os}chrony
Requires: %{_cross_os}coreutils
Requires: %{_cross_os}dbus-broker
Requires: %{_cross_os}filesystem
Requires: %{_cross_os}grub
Requires: %{_cross_os}iproute
Requires: %{_cross_os}kernel
Requires: %{_cross_os}kernel-modules
Requires: %{_cross_os}moondog
Requires: %{_cross_os}netdog
Requires: %{_cross_os}signpost
Requires: %{_cross_os}sundog
Requires: %{_cross_os}pluto
Requires: %{_cross_os}storewolf
Requires: %{_cross_os}settings-committer
Requires: %{_cross_os}systemd
Requires: %{_cross_os}thar-be-settings
Requires: %{_cross_os}migration
Requires: %{_cross_os}updog
Requires: %{_cross_os}util-linux
Requires: %{_cross_os}amazon-ssm-agent
Requires: %{_cross_os}preinit
Requires: %{_cross_os}wicked

%description
%{summary}.

%prep

%build

%install

install -d %{buildroot}%{_cross_bindir}
install -p -m 0755 %{S:1} %{buildroot}%{_cross_bindir}

install -d %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}
install -p -m 0644 %{S:10} %{S:11} %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}

install -d %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}/wicked/ifconfig
install -p -m 0644 %{S:1000} %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}/wicked/ifconfig

install -d %{buildroot}%{_cross_tmpfilesdir}
install -p -m 0644 %{S:99} %{buildroot}%{_cross_tmpfilesdir}/release.conf

cat >%{buildroot}%{_cross_libdir}/os-release <<EOF
NAME=Thar
PRETTY_NAME="Thar, The Operating System"
ID=thar
VERSION_ID=%{version}
EOF

install -d %{buildroot}%{_cross_unitdir}
install -p -m 0644 %{S:1001} %{S:1002} %{buildroot}%{_cross_unitdir}

install -d %{buildroot}%{_cross_unitdir}/multi-user.target.wants
ln -s ../systemd-networkd.service %{buildroot}%{_cross_unitdir}/multi-user.target.wants
ln -s ../var-lib-thar.mount %{buildroot}%{_cross_unitdir}/multi-user.target.wants

install -d %{buildroot}%{_cross_templatedir}
install -p -m 0644 %{S:200} %{buildroot}%{_cross_templatedir}/hostname

%files
%{_cross_bindir}/login
%{_cross_factorydir}%{_cross_sysconfdir}/hosts
%{_cross_factorydir}%{_cross_sysconfdir}/nsswitch.conf
%{_cross_factorydir}%{_cross_sysconfdir}/wicked/ifconfig/eth0.xml
%{_cross_tmpfilesdir}/release.conf
%{_cross_libdir}/os-release
%{_cross_unitdir}/configured.target
%{_cross_unitdir}/var-lib-thar.mount
%{_cross_unitdir}/multi-user.target.wants/systemd-networkd.service
%{_cross_unitdir}/multi-user.target.wants/var-lib-thar.mount
%dir %{_cross_templatedir}
%{_cross_templatedir}/hostname

%changelog
