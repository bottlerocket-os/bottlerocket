%global _cross_first_party 1

Name: %{_cross_os}login
Version: 0.0.1
Release: 1%{?dist}
Summary: A login helper
License: Apache-2.0 OR MIT
Source0: login
BuildRequires: %{_cross_os}glibc-devel
Requires: %{_cross_os}bash
Requires: %{_cross_os}systemd-console

%description
%{summary}.

%prep

%build

%install
install -d %{buildroot}%{_cross_bindir}
install -p -m 0755 %{S:0} %{buildroot}%{_cross_bindir}/login

%files
%{_cross_bindir}/login

%changelog
