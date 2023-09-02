# Skip check-rpaths since we expect them for systemd.
%global __brp_check_rpaths %{nil}

Name: %{_cross_os}systemd
Version: 252.13
Release: 1%{?dist}
Summary: System and Service Manager
License: GPL-2.0-or-later AND GPL-2.0-only AND LGPL-2.1-or-later
URL: https://www.freedesktop.org/wiki/Software/systemd
Source0: https://github.com/systemd/systemd-stable/archive/v%{version}/systemd-stable-%{version}.tar.gz
Source1: systemd-tmpfiles.conf
Source2: systemd-modules-load.conf
Source3: journald.conf
Source4: issue
Source5: systemd-journald.conf

# Local patch to work around the fact that /var is a bind mount from
# /local/var, and we want the /local/var/run symlink to point to /run.
Patch9001: 9001-use-absolute-path-for-var-run-symlink.patch

# TODO: this could potentially be submitted upstream, but needs a better
# way to be configured at build time or during execution first.
Patch9002: 9002-core-add-separate-timeout-for-system-shutdown.patch

# TODO: this could be submitted upstream as well, but needs to account for
# the dom0 case first, where the UUID is all zeroes and hence not unique.
Patch9003: 9003-machine-id-setup-generate-stable-ID-under-Xen-and-VM.patch

# Local patch to mount /tmp with "noexec".
Patch9004: 9004-units-mount-tmp-with-noexec.patch

# Local patch to mount additional filesystems with "noexec".
Patch9005: 9005-mount-setup-apply-noexec-to-more-mounts.patch

# Local patch to handle mounting /etc with our SELinux label.
Patch9006: 9006-mount-setup-mount-etc-with-specific-label.patch

# We need `prefix` to be configurable for our own packaging so we can avoid
# dependencies on the host OS.
Patch9007: 9007-pkg-config-stop-hardcoding-prefix-to-usr.patch

# Local patch to stop overriding rp_filter defaults with wildcard values.
Patch9008: 9008-sysctl-do-not-set-rp_filter-via-wildcard.patch

# Local patch to set root's shell to /sbin/nologin rather than /bin/sh.
Patch9009: 9009-sysusers-set-root-shell-to-sbin-nologin.patch

# Local patch to keep modprobe units running to avoid repeated log entries.
Patch9010: 9010-units-keep-modprobe-service-units-running.patch

# Local patch to conditionalize systemd-networkd calls to hostname and timezone
# DBUS services not used in Bottlerocket
Patch9011: 9011-systemd-networkd-Conditionalize-hostnamed-timezoned-.patch

# Local patch to adjust the default mount rate limit to 25 per second.
# Carried as a patch so that SYSTEMD_DEFAULT_MOUNT_RATE_LIMIT_BURST can be used
# as a kernel command line parameter to override.
Patch9012: 9012-core-mount-increase-mount-rate-limit-burst-to-25.patch

# Local patch to work around a potentially non-compliant Option 15 in the DHCP
# lease in EC2.
Patch9013: 9013-sd-dhcp-lease-parse-multiple-domains-in-option-15.patch

BuildRequires: gperf
BuildRequires: intltool
BuildRequires: meson
BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}kmod-devel
BuildRequires: %{_cross_os}libacl-devel
BuildRequires: %{_cross_os}libattr-devel
BuildRequires: %{_cross_os}libblkid-devel
BuildRequires: %{_cross_os}libcap-devel
BuildRequires: %{_cross_os}libfdisk-devel
BuildRequires: %{_cross_os}libmount-devel
BuildRequires: %{_cross_os}libseccomp-devel
BuildRequires: %{_cross_os}libselinux-devel
BuildRequires: %{_cross_os}libuuid-devel
BuildRequires: %{_cross_os}libxcrypt-devel
Requires: %{_cross_os}kmod
Requires: %{_cross_os}libacl
Requires: %{_cross_os}libattr
Requires: %{_cross_os}libblkid
Requires: %{_cross_os}libcap
Requires: %{_cross_os}libfdisk
Requires: %{_cross_os}libmount
Requires: %{_cross_os}libseccomp
Requires: %{_cross_os}libselinux
Requires: %{_cross_os}libuuid
Requires: %{_cross_os}libxcrypt

%description
%{summary}.

%package console
Summary: Files for console login using the System and Service Manager

%description console
%{summary}.

%package devel
Summary: Files for development using the System and Service Manager
Requires: %{name}

%description devel
%{summary}.

%package networkd
Summary: Files for networkd

%description networkd
%{summary}.

%package resolved
Summary: Files for resolved

%description resolved
%{summary}.

%prep
%autosetup -n systemd-stable-%{version} -p1

%build
CONFIGURE_OPTS=(
 -Dmode=release

 -Dsplit-usr=false
 -Dsplit-bin=true
 -Drootprefix='%{_cross_prefix}'
 -Drootlibdir='%{_cross_libdir}'
 -Dlink-udev-shared=true
 -Dlink-systemctl-shared=true
 -Dlink-networkd-shared=false
 -Dlink-timesyncd-shared=false
 -Dlink-boot-shared=false
 -Dstatic-libsystemd=false
 -Dstatic-libudev=false

 -Dsysvinit-path=''
 -Dsysvrcnd-path=''
 -Dinitrd=false
 -Dnscd=false

 -Dutmp=false
 -Dhibernate=false
 -Dldconfig=true
 -Dresolve=true
 -Defi=true
 -Dtpm=false
 -Denvironment-d=false
 -Dbinfmt=false
 -Drepart=true
 -Dcoredump=false
 -Dpstore=true
 -Doomd=false
 -Dlogind=true
 -Dhostnamed=false
 -Dlocaled=false
 -Dmachined=false
 -Dportabled=false
 -Dsysext=false
 -Dsysupdate=false
 -Duserdb=false
 -Dhomed=false
 -Dnetworkd=true
 -Dtimedated=false
 -Dtimesyncd=false
 -Dremote=false
 -Dnss-myhostname=false
 -Dnss-mymachines=false
 -Dnss-resolve=true
 -Dnss-systemd=false
 -Dfirstboot=false
 -Drandomseed=true
 -Dbacklight=false
 -Dvconsole=false
 -Dquotacheck=false
 -Dsysusers=true
 -Dtmpfiles=true
 -Dimportd=false
 -Dhwdb=false
 -Drfkill=false
 -Dxdg-autostart=false
 -Dman=false
 -Dhtml=false
 -Dtranslations=false
 -Dlog-message-verification=false

 -Dcertificate-root='%{_cross_sysconfdir}/ssl'
 -Dpkgconfigdatadir='%{_cross_pkgconfigdir}'
 -Dpkgconfiglibdir='%{_cross_pkgconfigdir}'

 %if %{with unified_cgroup_hierarchy}
 -Ddefault-hierarchy=unified
 %else
 -Ddefault-hierarchy=hybrid
 %endif

 -Dadm-group=false
 -Dwheel-group=false

 -Dgshadow=true

 -Ddefault-dnssec=no
 -Ddefault-dns-over-tls=no
 -Ddefault-mdns=no
 -Ddefault-llmnr=no
 -Ddns-over-tls=false
 -Ddns-servers=""

 -Dsupport-url="https://github.com/bottlerocket-os/bottlerocket/discussions"

 -Dseccomp=auto
 -Dselinux=auto
 -Dapparmor=false
 -Dsmack=false
 -Dpolkit=false
 -Dima=false

 -Dacl=true
 -Daudit=false
 -Dblkid=true
 -Dfdisk=true
 -Dkmod=true
 -Dpam=false
 -Dpwquality=false
 -Dmicrohttpd=false
 -Dlibcryptsetup=false
 -Dlibcurl=false
 -Didn=false
 -Dlibidn2=false
 -Dlibidn=false
 -Dlibiptc=false
 -Dqrencode=false
 -Dgcrypt=false
 -Dgnutls=false
 -Dopenssl=false
 -Dp11kit=false
 -Dlibfido2=false
 -Dtpm2=false
 -Delfutils=false
 -Dzlib=false
 -Dbzip2=false
 -Dxz=false
 -Dlz4=false
 -Dzstd=false
 -Dxkbcommon=false
 -Dpcre2=false
 -Dglib=false
 -Ddbus=false

 -Dgnu-efi=false
 -Defi-tpm-pcr-compat=false

 -Dbashcompletiondir=no
 -Dzshcompletiondir=no

 -Dtests=false
 -Dslow-tests=false
 -Dfuzz-tests=false
 -Dinstall-tests=false

 -Durlify=false
 -Dfexecve=false

 -Doss-fuzz=false
 -Dllvm-fuzz=false
 -Dkernel-install=false
 -Danalyze=true

 -Dbpf-framework=false
)

%cross_meson "${CONFIGURE_OPTS[@]}"
%cross_meson_build

%install
%cross_meson_install

install -d %{buildroot}%{_cross_tmpfilesdir}
install -p -m 0644 %{S:1} %{buildroot}%{_cross_tmpfilesdir}/systemd-tmpfiles.conf

install -d %{buildroot}%{_cross_libdir}/modules-load.d
install -p -m 0644 %{S:2} %{buildroot}%{_cross_libdir}/modules-load.d/nf_conntrack.conf

install -d %{buildroot}%{_cross_libdir}/systemd/journald.conf.d
install -p -m 0644 %{S:3} %{buildroot}%{_cross_libdir}/systemd/journald.conf.d/journald.conf

install -d %{buildroot}%{_cross_unitdir}/systemd-journald.service.d
install -p -m 0644 %{S:5} %{buildroot}%{_cross_unitdir}/systemd-journald.service.d/systemd-journald.conf

# Remove all stock network configurations, as they can interfere
# with container networking by attempting to manage veth devices.
rm -f %{buildroot}%{_cross_libdir}/systemd/network/*

# Remove default, multi-user and graphical targets provided by systemd,
# we override default/multi-user in the release spec and graphical
# is never used
rm -f %{buildroot}%{_cross_libdir}/systemd/{system,user}/default.target
rm -f %{buildroot}%{_cross_libdir}/systemd/{system,user}/multi-user.target
rm -f %{buildroot}%{_cross_libdir}/systemd/{system,user}/graphical.target

# Add art to the console
install -d %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}
install -p -m 0644 %{S:4} %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}/issue

%files
%license LICENSE.GPL2 LICENSE.LGPL2.1
%{_cross_attribution_file}
%{_cross_bindir}/busctl
%{_cross_bindir}/journalctl
%{_cross_bindir}/systemctl
%{_cross_bindir}/systemd-analyze
%{_cross_bindir}/systemd-ask-password
%{_cross_bindir}/systemd-cat
%{_cross_bindir}/systemd-cgls
%{_cross_bindir}/systemd-cgtop
%{_cross_bindir}/systemd-creds
%{_cross_bindir}/systemd-dissect
%{_cross_bindir}/systemd-delta
%{_cross_bindir}/systemd-detect-virt
%{_cross_bindir}/systemd-escape
%{_cross_bindir}/systemd-id128
%{_cross_bindir}/systemd-inhibit
%{_cross_bindir}/systemd-machine-id-setup
%{_cross_bindir}/systemd-mount
%{_cross_bindir}/systemd-notify
%{_cross_bindir}/systemd-nspawn
%{_cross_bindir}/systemd-path
%{_cross_bindir}/systemd-repart
%{_cross_bindir}/systemd-run
%{_cross_bindir}/systemd-socket-activate
%{_cross_bindir}/systemd-stdio-bridge
%{_cross_bindir}/systemd-sysusers
%{_cross_bindir}/systemd-tmpfiles
%{_cross_bindir}/systemd-tty-ask-password-agent
%{_cross_bindir}/systemd-umount
%{_cross_bindir}/udevadm
%{_cross_bindir}/loginctl

%{_cross_sbindir}/halt
%{_cross_sbindir}/init
%{_cross_sbindir}/poweroff
%{_cross_sbindir}/reboot
%{_cross_sbindir}/shutdown

%{_cross_libdir}/libsystemd.so.*
%{_cross_libdir}/libudev.so.*

%dir %{_cross_libdir}/modprobe.d
%{_cross_libdir}/modprobe.d/*

%dir %{_cross_libdir}/modules-load.d
%{_cross_libdir}/modules-load.d/nf_conntrack.conf

%dir %{_cross_sysctldir}
%{_cross_sysctldir}/*

%dir %{_cross_libdir}/systemd
%{_cross_libdir}/systemd/*
%exclude %{_cross_libdir}/systemd/systemd-user-runtime-dir
%exclude %{_cross_unitdir}/dbus-org.freedesktop.login1.service
%exclude %{_cross_unitdir}/systemd-repart.service
%exclude %{_cross_unitdir}/user-runtime-dir@.service
%exclude %{_cross_unitdir}/user@.service
%exclude %{_cross_unitdir}/user@.service.d
%exclude %{_cross_unitdir}/user@0.service.d

%dir %{_cross_libdir}/udev
%{_cross_libdir}/udev/*
%exclude %{_cross_libdir}/udev/rules.d/70-uaccess.rules
%exclude %{_cross_libdir}/udev/rules.d/71-seat.rules
%exclude %{_cross_libdir}/udev/rules.d/73-seat-late.rules

%dir %{_cross_sysusersdir}
%{_cross_sysusersdir}/*

%dir %{_cross_tmpfilesdir}
%{_cross_tmpfilesdir}/*
%exclude %{_cross_tmpfilesdir}/x11.conf

%{_cross_datadir}/dbus-1/*
%exclude %{_cross_datadir}/dbus-1/system-services

%dir %{_cross_factorydir}
%{_cross_factorydir}%{_cross_sysconfdir}/issue
%{_cross_factorydir}%{_cross_sysconfdir}/locale.conf
%exclude %{_cross_factorydir}%{_cross_sysconfdir}/nsswitch.conf
%exclude %{_cross_factorydir}%{_cross_sysconfdir}/pam.d
%exclude %{_cross_factorydir}%{_cross_sysconfdir}/pam.d/other
%exclude %{_cross_factorydir}%{_cross_sysconfdir}/pam.d/system-auth

%exclude %{_cross_datadir}/polkit-1
%exclude %{_cross_docdir}
%exclude %{_cross_libdir}/pam.d/systemd-user
%exclude %{_cross_sysconfdir}/systemd/
%exclude %{_cross_sysconfdir}/udev/
%exclude %{_cross_sysconfdir}/X11
%exclude %{_cross_sysconfdir}/xdg

# exclude files for subpackages
%exclude %{_cross_bindir}/systemd-ask-password
%exclude %{_cross_bindir}/systemd-tty-ask-password-agent
%exclude %{_cross_datadir}/dbus-1/system.d/org.freedesktop.network1.conf
%exclude %{_cross_datadir}/dbus-1/system.d/org.freedesktop.resolve1.conf
%exclude %{_cross_libdir}/systemd/resolv.conf
%exclude %{_cross_libdir}/systemd/systemd-networkd
%exclude %{_cross_libdir}/systemd/systemd-networkd-wait-online
%exclude %{_cross_libdir}/systemd/systemd-reply-password
%exclude %{_cross_libdir}/systemd/systemd-resolved
%exclude %{_cross_libdir}/systemd/systemd-sulogin-shell
%exclude %{_cross_systemdgeneratordir}/systemd-debug-generator
%exclude %{_cross_systemdgeneratordir}/systemd-getty-generator
%exclude %{_cross_sysusersdir}/systemd-network.conf
%exclude %{_cross_sysusersdir}/systemd-resolve.conf
%exclude %{_cross_tmpfilesdir}/systemd-network.conf
%exclude %{_cross_tmpfilesdir}/systemd-resolve.conf
%exclude %{_cross_unitdir}/autovt@.service
%exclude %{_cross_unitdir}/console-getty.service
%exclude %{_cross_unitdir}/container-getty@.service
%exclude %{_cross_unitdir}/debug-shell.service
%exclude %{_cross_unitdir}/emergency.service
%exclude %{_cross_unitdir}/emergency.target
%exclude %{_cross_unitdir}/getty@.service
%exclude %{_cross_unitdir}/rescue.service
%exclude %{_cross_unitdir}/rescue.target
%exclude %{_cross_unitdir}/serial-getty@.service
%exclude %{_cross_unitdir}/systemd-ask-password-console.service
%exclude %{_cross_unitdir}/systemd-ask-password-console.path
%exclude %{_cross_unitdir}/systemd-ask-password-wall.path
%exclude %{_cross_unitdir}/systemd-networkd.service
%exclude %{_cross_unitdir}/systemd-networkd-wait-online.service
%exclude %{_cross_unitdir}/systemd-networkd-wait-online@.service
%exclude %{_cross_unitdir}/systemd-networkd.socket
%exclude %{_cross_unitdir}/systemd-resolved.service
%exclude %{_cross_unitdir}/sysinit.target.wants/systemd-ask-password-console.path
%exclude %{_cross_unitdir}/multi-user.target.wants/systemd-ask-password-wall.path

%files devel
%{_cross_libdir}/libsystemd.so
%{_cross_libdir}/libudev.so
%{_cross_includedir}/libudev.h
%dir %{_cross_includedir}/systemd
%{_cross_includedir}/systemd/*.h
%{_cross_pkgconfigdir}/*.pc
%exclude %{_cross_libdir}/rpm/macros.d

%files console
%{_cross_bindir}/systemd-ask-password
%{_cross_bindir}/systemd-tty-ask-password-agent
%{_cross_libdir}/systemd/systemd-sulogin-shell
%{_cross_libdir}/systemd/systemd-reply-password
%{_cross_systemdgeneratordir}/systemd-debug-generator
%{_cross_systemdgeneratordir}/systemd-getty-generator
%{_cross_unitdir}/autovt@.service
%{_cross_unitdir}/console-getty.service
%{_cross_unitdir}/container-getty@.service
%{_cross_unitdir}/debug-shell.service
%{_cross_unitdir}/emergency.service
%{_cross_unitdir}/emergency.target
%{_cross_unitdir}/getty@.service
%{_cross_unitdir}/rescue.service
%{_cross_unitdir}/rescue.target
%{_cross_unitdir}/serial-getty@.service
%{_cross_unitdir}/systemd-ask-password-console.service
%{_cross_unitdir}/systemd-ask-password-console.path
%{_cross_unitdir}/systemd-ask-password-wall.path
%{_cross_unitdir}/sysinit.target.wants/systemd-ask-password-console.path
%{_cross_unitdir}/multi-user.target.wants/systemd-ask-password-wall.path

%files networkd
%{_cross_bindir}/networkctl
%{_cross_libdir}/systemd/systemd-networkd
%{_cross_libdir}/systemd/systemd-networkd-wait-online
%{_cross_sysusersdir}/systemd-network.conf
%{_cross_tmpfilesdir}/systemd-network.conf
%{_cross_unitdir}/systemd-networkd.service
%{_cross_unitdir}/systemd-networkd-wait-online.service
%{_cross_unitdir}/systemd-networkd-wait-online@.service
%{_cross_unitdir}/systemd-networkd.socket
%{_cross_datadir}/dbus-1/system.d/org.freedesktop.network1.conf

%files resolved
%{_cross_bindir}/resolvectl
%{_cross_libdir}/libnss_resolve.so.*
%{_cross_libdir}/systemd/resolv.conf
%{_cross_libdir}/systemd/systemd-resolved
%{_cross_sysusersdir}/systemd-resolve.conf
%{_cross_tmpfilesdir}/systemd-resolve.conf
%{_cross_unitdir}/systemd-resolved.service
%{_cross_datadir}/dbus-1/system.d/org.freedesktop.resolve1.conf
%exclude %{_cross_bindir}/systemd-resolve
%exclude %{_cross_sbindir}/resolvconf

%changelog
