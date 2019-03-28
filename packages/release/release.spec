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

Source1000: 00-any.network
Source1001: var-lib-thar.mount

BuildArch: noarch
Requires: %{_cross_os}bash
Requires: %{_cross_os}coreutils
Requires: %{_cross_os}filesystem
Requires: %{_cross_os}grub
Requires: %{_cross_os}kernel
Requires: %{_cross_os}systemd
Requires: %{_cross_os}util-linux

%description
%{summary}.

%prep

%build

%install
mkdir -p %{buildroot}%{_cross_sbindir}
install -m0755 %{SOURCE0} %{buildroot}%{_cross_sbindir}

mkdir -p %{buildroot}%{_cross_bindir}
install -m0755 %{SOURCE1} %{buildroot}%{_cross_bindir}

mkdir -p %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}
install -m0644 %{SOURCE10} %{SOURCE11} %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}

mkdir -p %{buildroot}%{_cross_tmpfilesdir}
install -m0644 %{SOURCE99} %{buildroot}%{_cross_tmpfilesdir}/release.conf

mkdir -p %{buildroot}%{_cross_libdir}/systemd/system/multi-user.target.wants

mkdir -p %{buildroot}%{_cross_libdir}/systemd/network
install -m0644 %{SOURCE1000} %{buildroot}%{_cross_libdir}/systemd/network
ln -s ../systemd-networkd.service %{buildroot}%{_cross_libdir}/systemd/system/multi-user.target.wants

mkdir -p %{buildroot}%{_cross_libdir}/systemd/system
install -m0644 %{SOURCE1001} %{buildroot}%{_cross_libdir}/systemd/system
ln -s ../var-lib-thar.mount %{buildroot}%{_cross_libdir}/systemd/system/multi-user.target.wants

%files
%{_cross_bindir}/login
%{_cross_sbindir}/preinit
%{_cross_factorydir}%{_cross_sysconfdir}/hosts
%{_cross_factorydir}%{_cross_sysconfdir}/nsswitch.conf
%{_cross_tmpfilesdir}/release.conf
%{_cross_libdir}/systemd/network/00-any.network
%{_cross_libdir}/systemd/system/var-lib-thar.mount
%{_cross_libdir}/systemd/system/multi-user.target.wants/systemd-networkd.service
%{_cross_libdir}/systemd/system/multi-user.target.wants/var-lib-thar.mount

%changelog
