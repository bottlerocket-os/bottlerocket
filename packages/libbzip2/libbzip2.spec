Name: %{_cross_os}libbzip2
Version: 1.0.8
Release: 1%{?dist}
Summary: Library for bzip2 compression
License: bzip2-1.0.6
URL: http://www.bzip.org
Source0: https://sourceware.org/pub/bzip2/bzip2-%{version}.tar.gz
Source1: bzip2.pc.in
Patch1: 0001-simplify-shared-object-build.patch
BuildRequires: %{_cross_os}glibc-devel

%description
%{summary}.

%package devel
Summary: Files for development using the library for bzip2 compression
Requires: %{name}

%description devel
%{summary}.

%prep
%autosetup -n bzip2-%{version} -p1
sed \
  -e "s,__PREFIX__,%{_cross_prefix},g" \
  -e "s,__EXEC_PREFIX__,%{_cross_exec_prefix},g" \
  -e "s,__LIBDIR__,%{_cross_libdir},g" \
  -e "s,__INCLUDEDIR__,%{_cross_includedir},g" \
  -e "s,__VERSION__,%{version},g" \
  -e "s,__DESCRIPTION__,%{description},g" \
  %{S:1} > bzip2.pc

%global set_env \
%set_cross_build_flags \\\
export CC="%{_cross_target}-gcc" \\\
export CFLAGS="${CFLAGS} -fpic -fPIC" \\\
%{nil}

%build
%set_env
%make_build -f Makefile-libbz2_so all

%install
install -d %{buildroot}{%{_cross_libdir},%{_cross_includedir},%{_cross_pkgconfigdir}}
install -m 755 libbz2.so.%{version} %{buildroot}%{_cross_libdir}
ln -s libbz2.so.%{version} %{buildroot}%{_cross_libdir}/libbz2.so.1
ln -s libbz2.so.1 %{buildroot}%{_cross_libdir}/libbz2.so
install -m 644 bzlib.h %{buildroot}%{_cross_includedir}
install -m 644 bzip2.pc %{buildroot}%{_cross_pkgconfigdir}

%files
%license LICENSE
%{_cross_attribution_file}
%{_cross_libdir}/*.so.*

%files devel
%{_cross_libdir}/*.so
%{_cross_includedir}/*.h
%{_cross_pkgconfigdir}/*.pc

%changelog
