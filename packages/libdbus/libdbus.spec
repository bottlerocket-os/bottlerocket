Name: %{_cross_os}libdbus
Version: 1.15.6
Release: 1%{?dist}
Summary: Library for a message bus
License: AFL-2.1 OR GPL-2.0-or-later
URL: http://www.freedesktop.org/Software/dbus/
Source0: https://dbus.freedesktop.org/releases/dbus/dbus-%{version}.tar.xz
BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}libcap-devel
BuildRequires: %{_cross_os}libexpat-devel
Requires: %{_cross_os}libcap
Requires: %{_cross_os}libexpat

%description
%{summary}.

%package devel
Summary: Files for development using the library for a message bus
Requires: %{name}

%description devel
%{summary}.

%prep
%autosetup -n dbus-%{version} -p1

%build
%cross_configure \
  --disable-asserts \
  --disable-doxygen-docs \
  --disable-ducktype-docs \
  --disable-tests \
  --disable-xml-docs \
  --disable-selinux \
  --disable-systemd \
  --with-xml=expat \

sed -i 's|^hardcode_libdir_flag_spec=.*|hardcode_libdir_flag_spec=""|g' libtool
sed -i 's|^runpath_var=LD_RUN_PATH|runpath_var=DIE_RPATH_DIE|g' libtool

%make_build

%install
%make_install

rm -rf %{buildroot}%{_cross_docdir}/dbus/examples

%files
%license COPYING
%{_cross_attribution_file}
%{_cross_libdir}/*.so.*
%exclude %{_cross_bindir}
%exclude %{_cross_datadir}/dbus-1
%exclude %{_cross_datadir}/doc
%exclude %{_cross_datadir}/xml
%exclude %{_cross_libexecdir}
%exclude %{_cross_sysconfdir}

%files devel
%{_cross_libdir}/*.a
%{_cross_libdir}/*.so
%dir %{_cross_libdir}/dbus-1.0
%{_cross_libdir}/dbus-1.0/*
%dir %{_cross_includedir}/dbus-1.0
%{_cross_includedir}/dbus-1.0/*
%{_cross_pkgconfigdir}/*.pc
%exclude %{_cross_libdir}/*.la
%exclude %{_cross_libdir}/cmake

%changelog
