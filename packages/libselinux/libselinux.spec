Name: %{_cross_os}libselinux
Version: 3.5
Release: 1%{?dist}
Summary: Library for SELinux
License: LicenseRef-SELinux-PD
URL: https://github.com/SELinuxProject/
Source0: https://github.com/SELinuxProject/selinux/releases/download/%{version}/libselinux-%{version}.tar.gz
BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}libpcre-devel
BuildRequires: %{_cross_os}libsepol-devel
Requires: %{_cross_os}libpcre

%description
%{summary}.

%package utils
Summary: A set of utilities for SELinux
Requires: %{name}
Requires: %{_cross_os}libpcre
Requires: %{_cross_os}libsepol

%description utils
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
%license LICENSE
%{_cross_attribution_file}
%{_cross_libdir}/*.so.*
%exclude %{_cross_mandir}

%files utils
%{_cross_sbindir}/avcstat
%{_cross_sbindir}/sefcontext_compile
%exclude %{_cross_sbindir}/compute_av
%exclude %{_cross_sbindir}/compute_create
%exclude %{_cross_sbindir}/compute_member
%exclude %{_cross_sbindir}/compute_relabel
%exclude %{_cross_sbindir}/getconlist
%exclude %{_cross_sbindir}/getdefaultcon
%exclude %{_cross_sbindir}/getenforce
%exclude %{_cross_sbindir}/getfilecon
%exclude %{_cross_sbindir}/getpidcon
%exclude %{_cross_sbindir}/getsebool
%exclude %{_cross_sbindir}/getseuser
%exclude %{_cross_sbindir}/matchpathcon
%exclude %{_cross_sbindir}/policyvers
%exclude %{_cross_sbindir}/selabel_digest
%exclude %{_cross_sbindir}/selabel_get_digests_all_partial_matches
%exclude %{_cross_sbindir}/selabel_lookup
%exclude %{_cross_sbindir}/selabel_lookup_best_match
%exclude %{_cross_sbindir}/selabel_partial_match
%exclude %{_cross_sbindir}/selinux_check_access
%exclude %{_cross_sbindir}/selinux_check_securetty_context
%exclude %{_cross_sbindir}/getpidprevcon
%exclude %{_cross_sbindir}/selinuxenabled
%exclude %{_cross_sbindir}/selinuxexeccon
%exclude %{_cross_sbindir}/setenforce
%exclude %{_cross_sbindir}/setfilecon
%exclude %{_cross_sbindir}/togglesebool
%exclude %{_cross_sbindir}/validatetrans

%files devel
%{_cross_libdir}/*.a
%{_cross_libdir}/*.so
%dir %{_cross_includedir}/selinux
%{_cross_includedir}/selinux/*
%{_cross_pkgconfigdir}/*.pc

%changelog
