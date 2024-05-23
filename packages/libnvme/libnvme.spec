Name: %{_cross_os}libnvme
Version: 1.9
Release: 1%{?dist}
Summary: Library for NVM Express
License: LGPL-2.1-only AND CC0-1.0 AND MIT
URL: https://github.com/linux-nvme/libnvme
Source0: https://github.com/linux-nvme/libnvme/archive/v%{version}/libnvme-%{version}.tar.gz
Patch0001: 0001-linux-Fix-uninitialized-variables.patch

BuildRequires: meson
BuildRequires: %{_cross_os}glibc-devel

%package devel
Summary: Files for development using the library for NVM Express
Requires: %{_cross_os}libnvme

%description devel
%{summary}.

%description
%{summary}.

%prep
%autosetup -n libnvme-%{version} -p1

%build
CONFIGURE_OPTS=(
 -Dpython=disabled
 -Dopenssl=disabled
 -Djson-c=disabled
 -Dkeyutils=disabled

 -Ddocs-build=false
)

%cross_meson "${CONFIGURE_OPTS[@]}"
%cross_meson_build

%install
%cross_meson_install

%files
%license COPYING ccan/licenses/BSD-MIT ccan/licenses/CC0
%{_cross_libdir}/*.so.*
%{_cross_attribution_file}

%files devel
%{_cross_includedir}/*.h
%{_cross_includedir}/nvme/*.h
%{_cross_libdir}/*.so
%{_cross_libdir}/pkgconfig/*.pc

%changelog
