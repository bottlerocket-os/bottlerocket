Name: %{_cross_os}libinih
Version: 57
Release: 1%{?dist}
Summary: Simple INI file parser library
License: BSD-3-Clause
URL: https://github.com/benhoyt/inih
Source0: https://github.com/benhoyt/inih/archive/refs/tags/r%{version}.tar.gz#/inih-r%{version}.tar.gz

BuildRequires: %{_cross_os}glibc-devel
BuildRequires: meson

%description
%{summary}.

%package devel
Summary: Files for development using the simple INI file parser library
Requires: %{name}

%description devel
%{summary}.

%prep
%autosetup -n inih-r%{version}

%build
CONFIGURE_OPTS=(
  -Dwith_INIReader=false
)

%cross_meson "${CONFIGURE_OPTS[@]}"
%cross_meson_build

%install
%cross_meson_install

%files
%license LICENSE.txt
%{_cross_attribution_file}
%{_cross_libdir}/libinih.so.0

%files devel
%{_cross_pkgconfigdir}/inih.pc
%{_cross_libdir}/libinih.so
%{_cross_includedir}/ini.h
