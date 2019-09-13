%global unversion 2_2_6

Name: %{_cross_os}libexpat
Version: %(echo %{unversion} | sed 's/_/./g')
Release: 1%{?dist}
Summary: Library for XML parsing
License: MIT
URL: https://libexpat.github.io/
Source0: https://github.com/libexpat/libexpat/archive/R_%{unversion}.tar.gz#/expat-%{version}.tar.gz
BuildRequires: gcc-%{_cross_target}
BuildRequires: %{_cross_os}glibc-devel
Requires: %{_cross_os}glibc

%description
%{summary}.

%package devel
Summary: Files for development using the library for XML parsing
Requires: %{name}

%description devel
%{summary}.

%prep
%autosetup -n libexpat-R_%{unversion}/expat -p1
./buildconf.sh

%build
%cross_configure \
  --without-docbook \
  --without-xmlwf \

%make_build

%install
%make_install

%files
%{_cross_libdir}/*.so.*
%exclude %{_cross_docdir}

%files devel
%{_cross_libdir}/*.a
%{_cross_libdir}/*.so
%{_cross_includedir}/*.h
%{_cross_pkgconfigdir}/*.pc
%exclude %{_cross_libdir}/*.la

%changelog
