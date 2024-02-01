%global _cross_first_party 1
%undefine _debugsource_packages

Name: %{_cross_os}netdog
Version: 0.1.0
Release: 0%{?dist}
Summary: Bottlerocket network configuration helper
License: Apache-2.0 OR MIT
URL: https://github.com/bottlerocket-os/bottlerocket

Source2: netdog-tmpfiles.conf

Source10: run-netdog.mount
Source11: write-network-status.service
Source12: generate-network-config.service
Source13: disable-udp-offload.service

Source20: 00-resolved.conf

BuildRequires: %{_cross_os}glibc-devel
Requires: %{_cross_os}netdog

%description
%{summary}.

%package -n %{_cross_os}netdog-common
Summary: Common configuration for Bottlerocket's network configuration helper
%description -n %{_cross_os}netdog-common
%{summary}.

%package -n %{_cross_os}netdog-systemd-networkd
Summary: Bottlerocket network configuration helper
Provides: %{_cross_os}netdog = 2:
Requires: %{_cross_os}netdog-common
Requires: %{_cross_os}systemd-networkd
Requires: %{_cross_os}systemd-resolved
Supplements: %{_cross_os}systemd-networkd
%description -n %{_cross_os}netdog-systemd-networkd
%{summary}.

%package -n %{_cross_os}netdog-wicked
Summary: Bottlerocket network configuration helper
Provides: %{_cross_os}netdog = 1:
Requires: %{_cross_os}netdog-common
Requires: %{_cross_os}wicked
Supplements: %{_cross_os}wicked
%description -n %{_cross_os}netdog-wicked
%{summary}.

%prep
%setup -T -c
%cargo_prep

%build
mkdir bin

echo "** Build Netdog Binaries"
%cargo_build --manifest-path %{_builddir}/sources/Cargo.toml \
    -p netdog \
    --features default \
    --target-dir=${HOME}/.cache/networkd
%cargo_build --manifest-path %{_builddir}/sources/Cargo.toml \
    -p netdog \
    --features wicked \
    --target-dir=${HOME}/.cache/wicked

%install
install -d %{buildroot}%{_cross_bindir}
install -p -m 0755 ${HOME}/.cache/networkd/%{__cargo_target}/release/netdog %{buildroot}%{_cross_bindir}/netdog-systemd-networkd
install -p -m 0755 ${HOME}/.cache/wicked/%{__cargo_target}/release/netdog %{buildroot}%{_cross_bindir}/netdog-wicked

install -d %{buildroot}%{_cross_tmpfilesdir}
install -p -m 0644 %{S:2} %{buildroot}%{_cross_tmpfilesdir}/netdog.conf

install -d %{buildroot}%{_cross_unitdir}
install -p -m 0644 %{S:10} %{S:11} %{S:12} %{S:13} %{buildroot}%{_cross_unitdir}

install -d %{buildroot}%{_cross_libdir}
install -d %{buildroot}%{_cross_libdir}/systemd/resolved.conf.d
install -p -m 0644 %{S:20} %{buildroot}%{_cross_libdir}/systemd/resolved.conf.d

%post -n %{_cross_os}netdog-wicked -p <lua>
posix.link("%{_cross_bindir}/netdog-wicked", "%{_cross_bindir}/netdog")

%post -n %{_cross_os}netdog-systemd-networkd -p <lua>
posix.link("%{_cross_bindir}/netdog-systemd-networkd", "%{_cross_bindir}/netdog")

%files -n %{_cross_os}netdog-common
%{_cross_tmpfilesdir}/netdog.conf
%{_cross_unitdir}/generate-network-config.service
%{_cross_unitdir}/disable-udp-offload.service
%{_cross_unitdir}/run-netdog.mount

%files -n %{_cross_os}netdog-systemd-networkd
%{_cross_bindir}/netdog-systemd-networkd
%{_cross_unitdir}/write-network-status.service
%dir %{_cross_libdir}/systemd/resolved.conf.d
%{_cross_libdir}/systemd/resolved.conf.d/00-resolved.conf

%files -n %{_cross_os}netdog-wicked
%{_cross_bindir}/netdog-wicked
