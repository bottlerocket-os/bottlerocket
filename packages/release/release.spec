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

%files

%changelog
