Name: %{_cross_os}release
Version: 1.0
Release: 1%{?dist}
Summary: Thar release
License: Public Domain
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
mkdir -p %{buildroot}%{_cross_sysconfdir}
touch %{buildroot}%{_cross_sysconfdir}/machine-id

mkdir -p %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}
cat <<'EOF' > %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}/hosts
127.0.0.1 localhost localhost.localdomain localhost4 localhost4.localdomain4
::1 localhost localhost.localdomain localhost6 localhost6.localdomain6
EOF

mkdir -p %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}
cat <<'EOF' > %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}/nsswitch.conf
passwd: files
group: files
shadow: files
hosts: files dns
EOF

mkdir -p %{buildroot}%{_cross_tmpfilesdir}
cat <<'EOF' > %{buildroot}%{_cross_tmpfilesdir}/release.conf
L+ %{_cross_sysconfdir}/machine-id - - - - ../run/machine-id
C %{_cross_sysconfdir}/hosts - - - -
C %{_cross_sysconfdir}/nsswitch.conf - - - -
EOF

# FIXME: build login from shadow-utils ?
mkdir -p %{buildroot}%{_cross_bindir}
cat <<'EOF' > %{buildroot}%{_cross_bindir}/login
#!/bin/bash
exec bash --login
EOF
chmod +x %{buildroot}%{_cross_bindir}/login

%files
%{_cross_sysconfdir}/machine-id
%{_cross_factorydir}%{_cross_sysconfdir}/hosts
%{_cross_factorydir}%{_cross_sysconfdir}/nsswitch.conf
%{_cross_tmpfilesdir}/release.conf
%{_cross_bindir}/login

%changelog
