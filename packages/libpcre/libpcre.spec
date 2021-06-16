Name: %{_cross_os}libpcre
Version: 10.37
Release: 1%{?dist}
Summary: Library for regular expressions
License: BSD-3-Clause
URL: https://www.pcre.org/
Source0: https://ftp.pcre.org/pub/pcre/pcre2-%{version}.tar.bz2
BuildRequires: %{_cross_os}glibc-devel

%description
%{summary}.

%package devel
Summary: Files for development using the library for regular expressions
Requires: %{name}

%description devel
%{summary}.

%prep
%autosetup -n pcre2-%{version} -p1

%build
%cross_configure \
  --enable-newline-is-lf \
  --enable-pcre2-8 \
  --enable-shared \
  --enable-static \
  --enable-unicode \
  --disable-jit \
  --disable-jit-sealloc \
  --disable-pcre2-16 \
  --disable-pcre2-32 \
  --disable-pcre2grep-callout \
  --disable-pcre2grep-callout-fork \
  --disable-pcre2grep-jit \
  --disable-pcre2grep-libbz2 \
  --disable-pcre2grep-libz \
  --disable-pcre2test-libedit \
  --disable-pcre2test-libreadline \

sed -i 's|^hardcode_libdir_flag_spec=.*|hardcode_libdir_flag_spec=""|g' libtool
sed -i 's|^runpath_var=LD_RUN_PATH|runpath_var=DIE_RPATH_DIE|g' libtool

%make_build

%install
%make_install

%files
%license LICENCE
%{_cross_attribution_file}
%{_cross_libdir}/*.so.*
%exclude %{_cross_bindir}
%exclude %{_cross_docdir}
%exclude %{_cross_mandir}

%files devel
%{_cross_libdir}/*.a
%{_cross_libdir}/*.so
%{_cross_includedir}/*.h
%{_cross_pkgconfigdir}/*.pc
%exclude %{_cross_libdir}/*.la

%changelog
