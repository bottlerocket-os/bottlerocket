%global _cross_templatedir %{_cross_datadir}/templates

Name: %{_cross_os}chrony
Version: 3.5
Release: 1%{?dist}
Summary: A versatile implementation of the Network Time Protocol
License: GPLv2
URL: https://chrony.tuxfamily.org
Source0: https://download.tuxfamily.org/chrony/chrony-3.5.tar.gz
Source1: chronyd.service
Source2: chrony-conf
Source3: chrony-tmpfiles.conf
BuildRequires: gcc-%{_cross_target}
BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}libcap-devel
BuildRequires: %{_cross_os}ncurses-devel
BuildRequires: %{_cross_os}readline-devel
Requires: %{_cross_os}glibc
Requires: %{_cross_os}libcap
Requires: %{_cross_os}ncurses
Requires: %{_cross_os}readline

%description
%{summary}.

%prep
%autosetup -n chrony-%{version} -p1

%build
%cross_configure

%make_build

%install
%make_install

install -d %{buildroot}%{_cross_unitdir}
install -p -m 0644 %{SOURCE1} %{buildroot}%{_cross_unitdir}/chronyd.service
install -d %{buildroot}%{_cross_templatedir}
install -p -m 0644 %{SOURCE2} %{buildroot}%{_cross_templatedir}/chrony-conf
install -d %{buildroot}%{_cross_tmpfilesdir}
install -p -m 0644 %{SOURCE3} %{buildroot}%{_cross_tmpfilesdir}/chrony-tmpfiles.conf

%files
%dir %{_cross_templatedir}
%{_cross_bindir}/chronyc
%{_cross_sbindir}/chronyd
%{_cross_templatedir}/chrony-conf
%{_cross_tmpfilesdir}/chrony-tmpfiles.conf
%{_cross_unitdir}/chronyd.service
%exclude %{_cross_mandir}

%changelog
