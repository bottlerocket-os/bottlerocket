Name: %{_cross_os}libsemanage
Version: 3.5
Release: 1%{?dist}
Summary: Library for SELinux binary policy manipulation
License: LGPL-2.1-or-later
URL: https://github.com/SELinuxProject/
Source0: https://github.com/SELinuxProject/selinux/releases/download/%{version}/libsemanage-%{version}.tar.gz
BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}libaudit-devel
BuildRequires: %{_cross_os}libbzip2-devel
BuildRequires: %{_cross_os}libselinux-devel
BuildRequires: %{_cross_os}libsepol-devel
Requires: %{_cross_os}libaudit
Requires: %{_cross_os}libbzip2
Requires: %{_cross_os}libselinux
Requires: %{_cross_os}libsepol

%description
%{summary}.

%package devel
Summary: Files for development using the library for SELinux binary policy manipulation
Requires: %{name}
Requires: %{_cross_os}libbzip2-devel

%description devel
%{summary}.

%prep
%autosetup -n libsemanage-%{version} -p1

%global set_env \
%set_cross_build_flags \\\
export CC="%{_cross_target}-gcc" \\\
export DESTDIR='%{buildroot}' \\\
export PREFIX='%{_cross_prefix}' \\\
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
%exclude %{_cross_libexecdir}
%exclude %{_cross_mandir}
%exclude %{_cross_sysconfdir}

%files devel
%{_cross_libdir}/*.a
%{_cross_libdir}/*.so
%dir %{_cross_includedir}/semanage
%{_cross_includedir}/semanage/*
%{_cross_pkgconfigdir}/*.pc

%changelog
