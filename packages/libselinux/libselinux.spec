Name: %{_cross_os}libselinux
Version: 3.0
Release: 1%{?dist}
Summary: Library for SELinux
License: Public Domain
URL: https://github.com/SELinuxProject/
Source0: https://github.com/SELinuxProject/selinux/releases/download/20191204/libselinux-%{version}.tar.gz
Patch1: 0001-adjust-default-selinux-directory.patch
BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}libpcre-devel
BuildRequires: %{_cross_os}libsepol-devel
Requires: %{_cross_os}libpcre

%description
%{summary}.

%package devel
Summary: Files for development using the library for SELinux
Requires: %{name}
Requires: %{_cross_os}libpcre-devel
Requires: %{_cross_os}libsepol-devel

%description devel
%{summary}.

%prep
%autosetup -n libselinux-%{version} -p1

%global set_env \
%set_cross_build_flags \\\
export CC="%{_cross_target}-gcc" \\\
export DESTDIR='%{buildroot}' \\\
export PREFIX='%{_cross_prefix}' \\\
export SHLIBDIR='%{_cross_libdir}' \\\
export DISABLE_RPM='y' \\\
export USE_PCRE2='y' \\\
%{nil}

%build
%set_env
%make_build

%install
%set_env
%make_install

%files
%{_cross_libdir}/*.so.*
%exclude %{_cross_sbindir}
%exclude %{_cross_mandir}

%files devel
%{_cross_libdir}/*.a
%{_cross_libdir}/*.so
%dir %{_cross_includedir}/selinux
%{_cross_includedir}/selinux/*
%{_cross_pkgconfigdir}/*.pc

%changelog
