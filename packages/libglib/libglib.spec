Name: %{_cross_os}libglib
Version: 2.70.1
Release: 1%{?dist}
Summary: The GLib libraries
# glib2 is LGPL-2.1-only
# pcre is BSD-3-Clause
License: LGPL-2.1-only AND BSD-3-Clause
URL: https://www.gtk.org/
Source0: https://download.gnome.org/sources/glib/2.70/glib-%{version}.tar.xz
# Note: the pcre version is specified in the glib archive in subprojects/libpcre.wrap
Source1: https://ftp.pcre.org/pub/pcre/pcre-8.37.tar.bz2
Source2: https://wrapdb.mesonbuild.com/v2/pcre_8.37-2/get_patch#/pcre_8.37-2_patch.zip
BuildRequires: meson
BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}libffi-devel
BuildRequires: %{_cross_os}libmount-devel
BuildRequires: %{_cross_os}libselinux-devel
BuildRequires: %{_cross_os}libz-devel
Requires: %{_cross_os}libffi
Requires: %{_cross_os}libmount
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
pushd subprojects >/dev/null
tar xf %{S:1}
unzip %{S:2}
popd >/dev/null

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
