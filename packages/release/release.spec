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

mkdir -p %{buildroot}%{_cross_tmpfilesdir}
cat <<'EOF' > %{buildroot}%{_cross_tmpfilesdir}/release.conf
L+ %{_cross_sysconfdir}/machine-id - - - - ../run/machine-id
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
%{_cross_tmpfilesdir}/release.conf
%{_cross_bindir}/login

%changelog
