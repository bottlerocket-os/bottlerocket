Name: %{_cross_os}chrony
Version: 3.5
Release: 1%{?dist}
Summary: A versatile implementation of the Network Time Protocol
License: GPL-2.0-only
URL: https://chrony.tuxfamily.org
Source0: https://download.tuxfamily.org/chrony/chrony-3.5.tar.gz
Source1: chronyd.service
Source2: chrony-conf
Source3: chrony-sysusers.conf
BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}libcap-devel
BuildRequires: %{_cross_os}libseccomp-devel
BuildRequires: %{_cross_os}ncurses-devel
BuildRequires: %{_cross_os}readline-devel
Requires: %{_cross_os}libcap
Requires: %{_cross_os}libseccomp
Requires: %{_cross_os}ncurses
Requires: %{_cross_os}readline

%description
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
install -p -m 0644 %{SOURCE1} %{buildroot}%{_cross_unitdir}/chronyd.service
install -d %{buildroot}%{_cross_templatedir}
install -p -m 0644 %{SOURCE2} %{buildroot}%{_cross_templatedir}/chrony-conf
install -d %{buildroot}%{_cross_sysusersdir}
install -p -m 0644 %{SOURCE3} %{buildroot}%{_cross_sysusersdir}/chrony.conf

%files
%license COPYING
%{_cross_attribution_file}
%dir %{_cross_templatedir}
%{_cross_bindir}/chronyc
%{_cross_sbindir}/chronyd
%{_cross_templatedir}/chrony-conf
%{_cross_unitdir}/chronyd.service
%{_cross_sysusersdir}/chrony.conf
%exclude %{_cross_mandir}

%changelog
