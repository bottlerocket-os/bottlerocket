Name: %{_cross_os}libzstd
Version: 1.5.5
Release: 1%{?dist}
Summary: Library for Zstandard compression
License: BSD-3-Clause AND GPL-2.0-only
URL: https://github.com/facebook/zstd/
Source0: https://github.com/faceboot/zstd/releases/download/v%{version}/zstd-%{version}.tar.gz
BuildRequires: %{_cross_os}glibc-devel

%description
%{summary}.

%package devel
Summary: Files for development using the library for Zstandard compression
Requires: %{_cross_os}libzstd

%description devel
%{summary}.

%prep
%autosetup -n zstd-%{version}

%global set_env \
%set_cross_build_flags \\\
export CC=%{_cross_target}-gcc \\\
export PREFIX=%{_cross_prefix} \\\
export DESTDIR=%{buildroot}%{_cross_rootdir} \\\
%{nil}

%build
%set_env
%make_build

%install
%set_env
%make_install

%files
%license COPYING
%{_cross_libdir}/*.so.*
%{_cross_attribution_file}
%exclude %{_cross_bindir}
%exclude %{_cross_mandir}

%files devel
%{_cross_includedir}/*.h
%{_cross_libdir}/*.so
%{_cross_libdir}/*.a
%{_cross_libdir}/pkgconfig/*.pc
