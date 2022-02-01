Name: %{_cross_os}libacl
Version: 2.3.1
Release: 1%{?dist}
Summary: Library for access control list support
License: LGPL-2.1-or-later
URL: https://savannah.nongnu.org/projects/acl
Source0: https://download-mirror.savannah.gnu.org/releases/acl/acl-%{version}.tar.gz
BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}libattr-devel
Requires: %{_cross_os}libattr

%description
%{summary}.

%package devel
Summary: Files for development using the library for access control list support
Requires: %{name}

%description devel
%{summary}.

%prep
%autosetup -n acl-%{version} -p1

%build
%cross_configure \
  --disable-nls \
  --disable-rpath \

sed -i 's|^hardcode_libdir_flag_spec=.*|hardcode_libdir_flag_spec=""|g' libtool
sed -i 's|^runpath_var=LD_RUN_PATH|runpath_var=DIE_RPATH_DIE|g' libtool

%make_build

%install
%make_install

%files
%license doc/COPYING.LGPL
%{_cross_attribution_file}
%{_cross_libdir}/*.so.*
%exclude %{_cross_bindir}
%exclude %{_cross_docdir}
%exclude %{_cross_mandir}

%files devel
%{_cross_libdir}/*.a
%{_cross_libdir}/*.so
%dir %{_cross_includedir}/acl
%{_cross_includedir}/acl/*.h
%{_cross_includedir}/sys/acl.h
%{_cross_pkgconfigdir}/*.pc
%exclude %{_cross_libdir}/*.la

%changelog
