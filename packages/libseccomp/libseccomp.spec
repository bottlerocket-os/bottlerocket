Name: %{_cross_os}libseccomp
Version: 2.5.4
Release: 1%{?dist}
Summary: Library for enhanced seccomp
License: LGPL-2.1-only
URL: https://github.com/seccomp/libseccomp
Source0: https://github.com/seccomp/libseccomp/releases/download/v%{version}/libseccomp-%{version}.tar.gz
BuildRequires: %{_cross_os}glibc-devel

%description
%{summary}.

%package devel
Summary: Files for development using the library for enhanced seccomp
Requires: %{name}

%description devel
%{summary}.

%prep
%autosetup -n libseccomp-%{version} -p1

%build
%cross_configure

# "fix" rpath
sed -i 's|^hardcode_libdir_flag_spec=.*|hardcode_libdir_flag_spec=""|g' libtool
sed -i 's|^runpath_var=LD_RUN_PATH|runpath_var=DIE_RPATH_DIE|g' libtool

%make_build

%install
%make_install

%files
%license LICENSE
%{_cross_attribution_file}
%{_cross_libdir}/*.so.*
%exclude %{_cross_bindir}/scmp_sys_resolver
%exclude %{_cross_mandir}

%files devel
%{_cross_libdir}/*.a
%{_cross_libdir}/*.so
%{_cross_includedir}/*.h
%{_cross_pkgconfigdir}/*.pc
%exclude %{_cross_libdir}/*.la

%changelog
