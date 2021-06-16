Name: %{_cross_os}libglib
Version: 2.68.3
Release: 1%{?dist}
Summary: The GLib libraries
License: LGPL-2.1-or-later
URL: https://www.gtk.org/
Source0: https://download.gnome.org/sources/glib/2.68/glib-%{version}.tar.xz
BuildRequires: meson
BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}libffi-devel
BuildRequires: %{_cross_os}libmount-devel
BuildRequires: %{_cross_os}libpcre-devel
BuildRequires: %{_cross_os}libselinux-devel
BuildRequires: %{_cross_os}libz-devel
Requires: %{_cross_os}libffi
Requires: %{_cross_os}libmount
Requires: %{_cross_os}libpcre
Requires: %{_cross_os}libselinux
Requires: %{_cross_os}libz

%description
%{summary}.

%package devel
Summary: Files for development using the GLib libraries
Requires: %{name}
Requires: %{_cross_os}libffi-devel

%description devel
%{summary}.

%prep
%autosetup -n glib-%{version} -p1

%build
CONFIGURE_OPTS=(
 -Dlibmount=enabled
 -Dselinux=enabled

 -Dlibelf=disabled
 -Dnls=disabled

 -Dman=false
 -Dtests=false
)

%cross_meson "${CONFIGURE_OPTS[@]}"
%cross_meson_build

%install
%cross_meson_install

%files
%{_cross_attribution_file}
%{_cross_libdir}/*.so.*
%exclude %{_cross_datadir}

%files devel
%{_cross_bindir}/*
%{_cross_libdir}/*.so
%dir %{_cross_libdir}/glib-2.0
%dir %{_cross_libdir}/glib-2.0/include
%{_cross_libdir}/glib-2.0/include/glibconfig.h
%dir %{_cross_includedir}/gio-unix-2.0
%dir %{_cross_includedir}/glib-2.0
%{_cross_includedir}/gio-unix-2.0/*
%{_cross_includedir}/glib-2.0/*
%{_cross_pkgconfigdir}/*.pc

%changelog
