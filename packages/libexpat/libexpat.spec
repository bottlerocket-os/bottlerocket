%global unversion 2_4_1

Name: %{_cross_os}libexpat
Version: %(echo %{unversion} | sed 's/_/./g')
Release: 1%{?dist}
Summary: Library for XML parsing
License: MIT
URL: https://libexpat.github.io/
Source0: https://github.com/libexpat/libexpat/releases/download/R_%{unversion}/expat-%{version}.tar.xz
BuildRequires: %{_cross_os}glibc-devel

%description
%{summary}.

%package devel
Summary: Files for development using the library for XML parsing
Requires: %{name}

%description devel
%{summary}.

%prep
%autosetup -n expat-%{version} -p1

%build
%cross_configure \
  --without-docbook \
  --without-xmlwf \

%make_build

%install
%make_install

%files
%license COPYING
%{_cross_attribution_file}
%{_cross_libdir}/*.so.*
%exclude %{_cross_docdir}

%files devel
%{_cross_libdir}/*.a
%{_cross_libdir}/*.so
%{_cross_includedir}/*.h
%{_cross_pkgconfigdir}/*.pc
%exclude %{_cross_libdir}/*.la
%exclude %{_cross_libdir}/cmake

%changelog
