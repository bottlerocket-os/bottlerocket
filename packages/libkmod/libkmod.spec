Name: %{_cross_os}libkmod
Version: 26
Release: 1%{?dist}
Summary: Library for kernel module loading and unloading
License: LGPLv2+
URL: http://git.kernel.org/?p=utils/kernel/kmod/kmod.git;a=summary
Source0: https://www.kernel.org/pub/linux/utils/kernel/kmod/kmod-%{version}.tar.xz
BuildRequires: gcc-%{_cross_target}
BuildRequires: %{_cross_os}glibc-devel
Requires: %{_cross_os}glibc

%description
%{summary}.

%package devel
Summary: Files for development using the library for kernel module loading and unloading
Requires: %{name}

%description devel
%{summary}.

%prep
%autosetup -n kmod-%{version} -p1

%build
%cross_configure \
  --without-openssl \
  --without-zlib \
  --without-xz

%make_build

%install
%make_install

%files
%{_cross_libdir}/*.so.*
%exclude %{_cross_bindir}/kmod
%exclude %{_cross_datadir}/bash-completion
%exclude %{_cross_mandir}

%files devel
%{_cross_libdir}/*.so
%{_cross_libdir}/pkgconfig/*.pc
%{_cross_includedir}/*.h
%exclude %{_cross_libdir}/*.la

%changelog
