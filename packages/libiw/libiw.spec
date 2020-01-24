Name: %{_cross_os}libiw
Version: 29
Release: 1%{?dist}
Summary: Library for wireless
License: GPL-2.0-or-later
URL: https://hewlettpackard.github.io/wireless-tools/
Source0: https://hewlettpackard.github.io/wireless-tools/wireless_tools.%{version}.tar.gz
Patch1: wireless-tools-29-makefile.patch

BuildRequires: %{_cross_os}glibc-devel

%description
%{summary}.

%package devel
Summary: Files for development using the library for wireless
Requires: %{name}

%description devel
%{summary}.

%prep
%autosetup -n wireless_tools.%{version} -p1

%build
make \
  CC="%{_cross_target}-gcc" \
  OPTFLAGS="%{_cross_cflags}" \
  LDFLAGS="%{_cross_ldflags}" \
  BUILD_SHARED=1 \

%install
make \
  INSTALL_INC=%{buildroot}/%{_cross_includedir} \
  INSTALL_LIB=%{buildroot}/%{_cross_libdir} \
  install-dynamic install-hdr

%files
%license COPYING
%{_cross_attribution_file}
%{_cross_libdir}/*.so.*

%files devel
%{_cross_libdir}/*.so
%{_cross_includedir}/*.h

%changelog
