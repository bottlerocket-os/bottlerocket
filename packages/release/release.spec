Name: %{_cross_os}release
Version: 1.0
Release: 1%{?dist}
Summary: Thar release
License: Public Domain
BuildArch: noarch
Requires: %{_cross_os}bash
Requires: %{_cross_os}filesystem
Requires: %{_cross_os}util-linux
Requires: %{_cross_os}systemd

%description
%{summary}.

%prep

%build

%install

%files

%changelog
