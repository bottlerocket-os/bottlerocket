%global templatedir %{_cross_datadir}/templates

Name: %{_cross_os}release
Version: 1.0
Release: 1%{?dist}
Summary: Thar release
License: Public Domain

Source0: preinit
Source1: login

Source10: hosts
Source11: nsswitch.conf
Source99: release.conf

# FIXME What should own system-level file templates?
Source200: hostname.template

Source1000: 00-any.network
Source1001: var-lib-thar.mount

BuildArch: noarch
Requires: %{_cross_os}apiserver
Requires: %{_cross_os}bash
Requires: %{_cross_os}ca-certificates
Requires: %{_cross_os}coreutils
Requires: %{_cross_os}filesystem
Requires: %{_cross_os}grub
Requires: %{_cross_os}kernel
Requires: %{_cross_os}moondog
Requires: %{_cross_os}ripgrep
Requires: %{_cross_os}signpost
Requires: %{_cross_os}sundog
Requires: %{_cross_os}systemd
Requires: %{_cross_os}thar-be-settings
Requires: %{_cross_os}util-linux

%description
%{summary}.

%prep

%build

%install
install -d %{buildroot}%{_cross_sbindir}
install -p -m 0755 %{S:0} %{buildroot}%{_cross_sbindir}

install -d %{buildroot}%{_cross_bindir}
install -p -m 0755 %{S:1} %{buildroot}%{_cross_bindir}

install -d %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}
install -p -m 0644 %{S:10} %{S:11} %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}

install -d %{buildroot}%{_cross_tmpfilesdir}
install -p -m 0644 %{S:99} %{buildroot}%{_cross_tmpfilesdir}/release.conf

install -d %{buildroot}%{_cross_libdir}/systemd/network
install -p -m 0644 %{S:1000} %{buildroot}%{_cross_libdir}/systemd/network

install -d %{buildroot}%{_cross_unitdir}
install -p -m 0644 %{S:1001} %{buildroot}%{_cross_unitdir}

install -d %{buildroot}%{_cross_unitdir}/multi-user.target.wants
ln -s ../systemd-networkd.service %{buildroot}%{_cross_unitdir}/multi-user.target.wants
ln -s ../var-lib-thar.mount %{buildroot}%{_cross_unitdir}/multi-user.target.wants

mkdir -p %{buildroot}%{templatedir}
install -m 0644 %{S:200} %{buildroot}%{templatedir}/hostname

%files
%{_cross_bindir}/login
%{_cross_sbindir}/preinit
%{_cross_factorydir}%{_cross_sysconfdir}/hosts
%{_cross_factorydir}%{_cross_sysconfdir}/nsswitch.conf
%{_cross_tmpfilesdir}/release.conf
%{_cross_libdir}/systemd/network/00-any.network
%{_cross_unitdir}/var-lib-thar.mount
%{_cross_unitdir}/multi-user.target.wants/systemd-networkd.service
%{_cross_unitdir}/multi-user.target.wants/var-lib-thar.mount
%dir %{templatedir}
%{templatedir}/hostname

%changelog
