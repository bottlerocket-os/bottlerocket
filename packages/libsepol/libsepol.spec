Name: %{_cross_os}libsepol
Version: 3.5
Release: 1%{?dist}
Summary: Library for SELinux policy manipulation
License: LGPL-2.1-or-later
URL: https://github.com/SELinuxProject/
Source0: https://github.com/SELinuxProject/selinux/releases/download/%{version}/libsepol-%{version}.tar.gz
BuildRequires: %{_cross_os}glibc-devel

%description
%{summary}.

%package devel
Summary: Files for development using the library for SELinux policy manipulation
Requires: %{name}

%description devel
%{summary}.

%prep
%autosetup -n libsepol-%{version} -p1

%global set_env \
%set_cross_build_flags \\\
export CC="%{_cross_target}-gcc" \\\
export DESTDIR='%{buildroot}' \\\
export PREFIX='%{_cross_prefix}' \\\
export SHLIBDIR='%{_cross_libdir}' \\\
%{nil}

%build
%set_env
%make_build

%install
%set_env
%make_install

%files
%license LICENSE
%{_cross_attribution_file}
%{_cross_libdir}/*.so.*
%exclude %{_cross_bindir}
%exclude %{_cross_mandir}

%files devel
%{_cross_libdir}/*.a
%{_cross_libdir}/*.so
%dir %{_cross_includedir}/sepol
%{_cross_includedir}/sepol/*
%{_cross_pkgconfigdir}/*.pc

%changelog
