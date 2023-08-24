Name: %{_cross_os}ethtool
Version: 6.4
Release: 1%{?dist}
Summary: Settings tool for Ethernet NICs
License: GPL-2.0-only AND GPL-2.0-or-later
URL: https://www.kernel.org/pub/software/network/ethtool/
Source0: https://www.kernel.org/pub/software/network/ethtool/ethtool-%{version}.tar.xz
BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}libmnl-devel

%description
%{summary}.

%prep
%setup -n ethtool-%{version}

%build
%cross_configure
%make_build

%install
%make_install

%files
%license COPYING LICENSE
%{_cross_attribution_file}
%{_cross_sbindir}/ethtool
%exclude %{_cross_datadir}/bash-completion
%exclude %{_cross_mandir}
