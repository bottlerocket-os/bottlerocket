Name: %{_cross_os}conntrack-tools
Version: 1.4.7
Release: 1%{?dist}
Summary: Tools for managing Linux kernel connection tracking
# src/utils.c contains GPLv2-only code from linux
License: GPL-2.0-or-later AND GPL-2.0-only
URL: http://conntrack-tools.netfilter.org/
Source0: https://www.netfilter.org/projects/conntrack-tools/files/conntrack-tools-%{version}.tar.bz2
Patch1: 0001-disable-RPC-helper.patch

BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}libmnl-devel
BuildRequires: %{_cross_os}libnfnetlink-devel
BuildRequires: %{_cross_os}libnetfilter_conntrack-devel
BuildRequires: %{_cross_os}libnetfilter_cthelper-devel
BuildRequires: %{_cross_os}libnetfilter_cttimeout-devel
BuildRequires: %{_cross_os}libnetfilter_queue-devel
Requires: %{_cross_os}libmnl
Requires: %{_cross_os}libnfnetlink
Requires: %{_cross_os}libnetfilter_conntrack
Requires: %{_cross_os}libnetfilter_cthelper
Requires: %{_cross_os}libnetfilter_cttimeout
Requires: %{_cross_os}libnetfilter_queue

%description
%{summary}.

%package devel
Summary: Files for development using the tools for managing Linux kernel connection tracking
Requires: %{name}

%description devel
%{summary}.

%prep
%autosetup -n conntrack-tools-%{version} -p1

%build
autoreconf -fi
%cross_configure

%make_build

%install
%make_install

%files
%license COPYING
%{_cross_attribution_file}
%{_cross_sbindir}/conntrack
%exclude %{_cross_sbindir}/conntrackd
%exclude %{_cross_sbindir}/nfct
%exclude %{_cross_libdir}/conntrack-tools
%exclude %{_cross_mandir}

%changelog
