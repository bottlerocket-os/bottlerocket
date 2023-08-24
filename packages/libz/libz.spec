Name: %{_cross_os}libz
Version: 1.3
Release: 1%{?dist}
Summary: Library for zlib compression
URL: https://www.zlib.net/
License: Zlib
Source: https://www.zlib.net/zlib-%{version}.tar.xz
BuildRequires: %{_cross_os}glibc-devel

%description
%{summary}.

%package devel
Summary: Files for development using the library for zlib compression
Requires: %{name}

%description devel
%{summary}.

%prep
%setup -n zlib-%{version}

# Sets cross build flags, target cross compiler, and env variables
# required to `make install` libz
%global set_env \
%set_cross_build_flags \\\
export CROSS_PREFIX="%{_cross_target}-" \\\
%{nil}

%build
%set_env
# zlib only reads prefix from this argument, not the environment
./configure --prefix='%{_cross_prefix}'
%make_build

%install
%set_env
%make_install

%files
%license README
%{_cross_attribution_file}
%{_cross_libdir}/*.so.*
%exclude %{_cross_mandir}

%files devel
%{_cross_libdir}/*.so
%{_cross_includedir}/*.h
%{_cross_libdir}/*.a
%{_cross_pkgconfigdir}/*.pc
