Name: %{_cross_os}findutils
Version: 4.9.0
Release: 1%{?dist}
Summary: A set of GNU tools for finding
License: GPL-3.0-or-later
URL: http://www.gnu.org/software/findutils/
Source0: https://ftp.gnu.org/pub/gnu/findutils/findutils-%{version}.tar.xz
BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}libselinux-devel
Requires: %{_cross_os}libselinux

%description
%{summary}.

%prep
%autosetup -n findutils-%{version} -p1

%build
%cross_configure
%make_build

%install
%make_install

%files
%license COPYING
%{_cross_attribution_file}
%{_cross_bindir}/find
%{_cross_bindir}/xargs
%exclude %{_cross_bindir}/locate
%exclude %{_cross_bindir}/updatedb
%exclude %{_cross_infodir}
%exclude %{_cross_libexecdir}
%exclude %{_cross_localedir}
%exclude %{_cross_mandir}

%changelog
