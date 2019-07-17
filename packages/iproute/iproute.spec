Name: %{_cross_os}iproute
Version: 4.15.0
Release: 1%{?dist}
Summary: Tools for advanced IP routing and network device configuration
License: GPLv2+ and Public Domain
URL: http://kernel.org/pub/linux/utils/net/iproute2/
Source0: http://kernel.org/pub/linux/utils/net/iproute2/iproute2-%{version}.tar.xz

BuildRequires: gcc-%{_cross_target}
BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}libcap-devel
BuildRequires: %{_cross_os}libmnl-devel
Requires: %{_cross_os}glibc
Requires: %{_cross_os}libcap
Requires: %{_cross_os}libmnl

%description
%{summary}.

%prep
%autosetup -n iproute2-%{version} -p1

%global set_dirs \
export DESTDIR='%{buildroot}' \\\
export SBINDIR='%{_cross_sbindir}' \\\
export MANDIR='%{_cross_mandir}' \\\
export LIBDIR='%{_cross_libdir}' \\\
export CONFDIR='%{_cross_sysconfdir}/iproute2' \\\
export DOCDIR='%{_cross_docdir}' \\\
export HDRDIR='%{_cross_includedir}' \\\
export BASH_COMPDIR='%{_cross_bashdir}' \\\
export PKG_CONFIG_PATH='%{_cross_pkgconfigdir}' \\\
%{nil}

%build
%set_dirs
%set_cross_build_flags
./configure
%make_build

%install
%set_dirs
%make_install

install -d %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}
mv %{buildroot}%{_cross_sysconfdir}/iproute2 %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}

install -d %{buildroot}%{_cross_tmpfilesdir}
for f in %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}/iproute2/* ; do
  echo "C %{_cross_sysconfdir}/iproute2/${f##*/} - - - -" >> %{buildroot}%{_cross_tmpfilesdir}/iproute2.conf
done

%files
%{_cross_sbindir}/bridge
%{_cross_sbindir}/ctstat
%{_cross_sbindir}/devlink
%{_cross_sbindir}/genl
%{_cross_sbindir}/ifcfg
%{_cross_sbindir}/ifstat
%{_cross_sbindir}/ip
%{_cross_sbindir}/lnstat
%{_cross_sbindir}/nstat
%{_cross_sbindir}/rdma
%{_cross_sbindir}/routef
%{_cross_sbindir}/routel
%{_cross_sbindir}/rtacct
%{_cross_sbindir}/rtmon
%{_cross_sbindir}/rtpr
%{_cross_sbindir}/rtstat
%{_cross_sbindir}/ss
%{_cross_sbindir}/tc
%{_cross_sbindir}/tipc
%dir %{_cross_libdir}/tc
%{_cross_libdir}/tc/*
%dir %{_cross_factorydir}%{_cross_sysconfdir}/iproute2
%{_cross_factorydir}%{_cross_sysconfdir}/iproute2/*
%{_cross_tmpfilesdir}/iproute2.conf
%exclude %{_cross_bashdir}/*
%exclude %{_cross_docdir}/*
%exclude %{_cross_mandir}/*
%exclude %{_cross_includedir}/*

%changelog
