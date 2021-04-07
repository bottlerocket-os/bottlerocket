Name: %{_cross_os}libattr
Version: 2.5.1
Release: 1%{?dist}
Summary: Library for extended attribute support
License: LGPL-2.1-or-later
URL: https://savannah.nongnu.org/projects/attr
Source0: https://download-mirror.savannah.gnu.org/releases/attr/attr-%{version}.tar.xz
BuildRequires: %{_cross_os}glibc-devel

%description
%{summary}.

%package devel
Summary: Files for development using the library for extended attribute support
Requires: %{name}

%description devel
%{summary}.

%prep
%autosetup -n attr-%{version} -p1

%build
%cross_configure
sed -i 's|^hardcode_libdir_flag_spec=.*|hardcode_libdir_flag_spec=""|g' libtool
sed -i 's|^runpath_var=LD_RUN_PATH|runpath_var=DIE_RPATH_DIE|g' libtool

%make_build

%install
%make_install

%files
%license doc/COPYING.LGPL
%{_cross_attribution_file}
%{_cross_libdir}/*.so.*
%exclude %{_cross_sysconfdir}/xattr.conf
%exclude %{_cross_bindir}
%exclude %{_cross_docdir}
%exclude %{_cross_infodir}
%exclude %{_cross_localedir}
%exclude %{_cross_mandir}

%files devel
%{_cross_libdir}/*.a
%{_cross_libdir}/*.so
%dir %{_cross_includedir}/attr
%{_cross_includedir}/attr/*.h
%{_cross_pkgconfigdir}/*.pc
%exclude %{_cross_libdir}/*.la

%changelog
