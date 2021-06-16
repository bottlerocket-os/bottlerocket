Name: %{_cross_os}chrony
Version: 4.1
Release: 1%{?dist}
Summary: A versatile implementation of the Network Time Protocol
License: GPL-2.0-only
URL: https://chrony.tuxfamily.org
Source0: https://download.tuxfamily.org/chrony/chrony-%{version}.tar.gz
Source1: chronyd.service
Source2: chrony-conf
Source3: chrony-sysusers.conf
Source4: chrony-tmpfiles.conf
BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}libcap-devel
BuildRequires: %{_cross_os}libseccomp-devel
BuildRequires: %{_cross_os}ncurses-devel
BuildRequires: %{_cross_os}readline-devel
Requires: %{_cross_os}libcap
Requires: %{_cross_os}libseccomp

%description
%{summary}.

%package tools
Summary: Command-line interface for chrony daemon
Requires: %{_cross_os}chrony
Requires: %{_cross_os}readline
Requires: %{_cross_os}ncurses

%description tools
%{summary}.

%prep
%autosetup -n chrony-%{version} -p1

%build
# chrony uses a custom hand-rolled configure script
%set_cross_build_flags \
CC=%{_cross_target}-gcc \
./configure \
 --prefix="%{_cross_prefix}" \
 --enable-scfilter

%make_build

%install
%make_install

install -d %{buildroot}%{_cross_unitdir}
install -p -m 0644 %{S:1} %{buildroot}%{_cross_unitdir}/chronyd.service
install -d %{buildroot}%{_cross_templatedir}
install -p -m 0644 %{S:2} %{buildroot}%{_cross_templatedir}/chrony-conf
install -d %{buildroot}%{_cross_sysusersdir}
install -p -m 0644 %{S:3} %{buildroot}%{_cross_sysusersdir}/chrony.conf
install -d %{buildroot}%{_cross_tmpfilesdir}
install -p -m 0644 %{S:4} %{buildroot}%{_cross_tmpfilesdir}/chrony.conf

%files
%license COPYING
%{_cross_attribution_file}
%dir %{_cross_templatedir}
%{_cross_sbindir}/chronyd
%{_cross_templatedir}/chrony-conf
%{_cross_unitdir}/chronyd.service
%{_cross_sysusersdir}/chrony.conf
%{_cross_tmpfilesdir}/chrony.conf
%exclude %{_cross_mandir}

%files tools
%{_cross_bindir}/chronyc

%changelog
