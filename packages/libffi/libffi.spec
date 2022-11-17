Name: %{_cross_os}libffi
Version: 3.4.4
Release: 1%{?dist}
Summary: Library for FFI
License: MIT
URL: https://sourceware.org/libffi/
Source0: https://github.com/libffi/libffi/releases/download/v%{version}/libffi-%{version}.tar.gz
BuildRequires: %{_cross_os}glibc-devel

%description
%{summary}.

%package devel
Summary: Files for development using the library for FFI
Requires: %{name}

%description devel
%{summary}.

%prep
%autosetup -n libffi-%{version} -p1

%build
%cross_configure \
  --disable-docs \
  --disable-multi-os-directory \

%make_build

%install
%make_install

%files
%license LICENSE
%{_cross_attribution_file}
%{_cross_libdir}/*.so.*
%exclude %{_cross_mandir}

%files devel
%{_cross_libdir}/*.a
%{_cross_libdir}/*.so
%{_cross_includedir}/*.h
%{_cross_pkgconfigdir}/*.pc
%exclude %{_cross_libdir}/*.la

%changelog
