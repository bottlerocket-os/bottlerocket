Name: %{_cross_os}readline
Version: 8.2
Release: 1%{?dist}
Summary: A library for editing typed command lines
License: GPL-3.0-or-later
URL: https://tiswww.case.edu/php/chet/readline/rltop.html
Source0: https://ftp.gnu.org/gnu/readline/readline-%{version}.tar.gz
Patch1: readline-8.2-shlib.patch
BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}libncurses-devel
Requires: %{_cross_os}libncurses

%description
%{summary}.

%package devel
Summary: Files for development using a library for editing typed command lines
Requires: %{name}

%description devel
%{summary}.

%prep
%autosetup -n readline-%{version} -p1

%build
%cross_configure --with-curses --disable-install-examples
%make_build

%install
%make_install

%files
%license COPYING
%{_cross_attribution_file}
%{_cross_libdir}/*.so.*
%exclude %{_cross_infodir}
%exclude %{_cross_mandir}
%exclude %{_cross_datadir}/doc/readline/*

%files devel
%{_cross_libdir}/*.a
%{_cross_libdir}/*.so
%dir %{_cross_includedir}/readline
%{_cross_includedir}/readline/*.h
%{_cross_pkgconfigdir}/*.pc

%changelog
