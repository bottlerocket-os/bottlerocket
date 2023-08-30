Name: %{_cross_os}liblzma
Version: 5.4.4
Release: 1%{?dist}
Summary: Library for XZ and LZMA compressed files
URL: https://tukaani.org/xz
License: LicenseRef-scancode-lzma-sdk-pd
Source: https://tukaani.org/xz/xz-%{version}.tar.xz
BuildRequires: %{_cross_os}glibc-devel

%description
%{summary}.

%package devel
Summary: Files for development using the library for XZ and LZMA compression
Requires: %{name}

%description devel
%{summary}.

%prep
%setup -n xz-%{version}

%build
%cross_configure \
  --disable-doc \
  --disable-lzma-links \
  --disable-lzmadec \
  --disable-lzmainfo \
  --disable-scripts \
  --disable-xz \
  --disable-xzdec
%make_build

%install
%make_install

%files
%license COPYING
%{_cross_attribution_file}
%{_cross_libdir}/*.so.*
%exclude %{_cross_localedir}

%files devel
%{_cross_includedir}/*.h
%{_cross_includedir}/lzma/*.h
%{_cross_libdir}/*.a
%{_cross_libdir}/*.so
%{_cross_pkgconfigdir}/*.pc
%exclude %{_cross_libdir}/*.la
