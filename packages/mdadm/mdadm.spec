Name: %{_cross_os}mdadm
Version: 4.3
Release: 1%{?dist}
Summary: mdadm is used for controlling Linux md devices (aka RAID arrays)
License: GPL-2.0-only
URL: https://cdn.kernel.org/pub/linux/utils/raid/mdadm/
Source0: https://cdn.kernel.org/pub/linux/utils/raid/mdadm/mdadm-%{version}.tar.xz
BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}systemd-devel

Source100: mdadm-tmpfiles.conf
Patch100: 0001-report-monitor-output-to-syslog.patch

%description
%{summary}.

%global set_env \
%set_cross_build_flags \\\
export CC=%{_cross_target}-gcc \\\
CXFLAGS="%{_cross_cflags} -DNO_COROSYNC -DNO_DLM" \
%{nil}

%prep
%autosetup -n mdadm-%{version} -p1

%build
%set_env
make LDFLAGS="%{_cross_ldflags}"

%install
%set_env
make install-bin DESTDIR=%{buildroot}%{_cross_rootdir}/usr
make install-udev DESTDIR=%{buildroot}
make install-systemd DESTDIR= SYSTEMD_DIR=%{buildroot}%{_cross_unitdir}

install -d %{buildroot}%{_cross_tmpfilesdir}
install -p -m 0644 %{S:100} %{buildroot}%{_cross_tmpfilesdir}/mdadm.conf

%files
%license COPYING
%{_cross_attribution_file}

%{_cross_sbindir}/mdadm
%{_cross_sbindir}/mdmon

%{_cross_tmpfilesdir}/mdadm.conf

%{_cross_udevrulesdir}/01-md-raid-creating.rules
%{_cross_udevrulesdir}/63-md-raid-arrays.rules
%{_cross_udevrulesdir}/64-md-raid-assembly.rules
%{_cross_udevrulesdir}/69-md-clustered-confirm-device.rules

%{_cross_unitdir}/mdadm-last-resort@.service
%{_cross_unitdir}/mdadm-last-resort@.timer
%{_cross_unitdir}/mdadm-grow-continue@.service
%{_cross_unitdir}/mdmon@.service
%{_cross_unitdir}/mdmonitor.service
%{_cross_unitdir}-shutdown/mdadm.shutdown

# periodically runs an mdcheck bash script
%exclude %{_cross_unitdir}/mdcheck_continue.service
%exclude %{_cross_unitdir}/mdcheck_continue.timer
%exclude %{_cross_unitdir}/mdcheck_start.service
%exclude %{_cross_unitdir}/mdcheck_start.timer

# no mail address or alert command, so no-ops
%exclude %{_cross_unitdir}/mdmonitor-oneshot.service
%exclude %{_cross_unitdir}/mdmonitor-oneshot.timer
