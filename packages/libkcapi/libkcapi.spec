# libkcapi since 85bce6035b (1.5.0) uses sha512hmac with the same key for all
# self-checks. Earlier versions used sha256hmac with a different key to check
# the shared library.
%global openssl_sha512_hmac openssl sha512 -hmac FIPS-FTW-RHT2009 -hex

# We need to compute the HMAC after the binaries have been stripped.
%define __spec_install_post\
%{?__debug_package:%{__debug_install_post}}\
%{__arch_install_post}\
%{__os_install_post}\
cd %{buildroot}/%{_cross_bindir}\
%openssl_sha512_hmac kcapi-hasher\\\
  | awk '{ print $2 }' > .kcapi-hasher.hmac\
ln -s .kcapi-hasher.hmac .sha512hmac.hmac\
cd %{buildroot}/%{_cross_libdir}\
%openssl_sha512_hmac libkcapi.so.%{version}\\\
  | awk '{ print $2 }' > .libkcapi.so.%{version}.hmac\
ln -s .libkcapi.so.%{version}.hmac .libkcapi.so.1.hmac\
%{nil}

Name: %{_cross_os}libkcapi
Version: 1.5.0
Release: 1%{?dist}
Summary: Library for kernel crypto API
License: BSD-3-Clause OR GPL-2.0-only
URL: https://www.chronox.de/libkcapi/html/index.html
Source0: https://github.com/smuellerDD/libkcapi/archive/v%{version}/libkcapi-%{version}.tar.gz
BuildRequires: %{_cross_os}glibc-devel

%description
%{summary}.

%package devel
Summary: Files for development using the library for kernel crypto API
Requires: %{name}

%description devel
%{summary}.

%prep
%autosetup -n libkcapi-%{version} -p1

%build
autoreconf -fi
%cross_configure \
  --enable-static \
  --enable-shared \
  --enable-kcapi-hasher \

%force_disable_rpath

%make_build

%install
%make_install

ln -s kcapi-hasher %{buildroot}%{_cross_bindir}/sha512hmac
find %{buildroot} -type f -name '*.hmac' -delete

%files
%license COPYING COPYING.bsd COPYING.gplv2
%{_cross_attribution_file}
%{_cross_libdir}/*.so.*
%{_cross_libdir}/.*.so.*.hmac
%{_cross_bindir}/kcapi-hasher
%{_cross_bindir}/.kcapi-hasher.hmac
%{_cross_bindir}/sha512hmac
%{_cross_bindir}/.sha512hmac.hmac

%exclude %{_cross_libexecdir}/libkcapi
%exclude %{_cross_mandir}

%files devel
%{_cross_libdir}/*.a
%{_cross_libdir}/*.so
%{_cross_includedir}/kcapi.h
%{_cross_pkgconfigdir}/*.pc

%changelog
