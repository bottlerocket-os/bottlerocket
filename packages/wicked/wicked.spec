# wicked is not cross-compilation aware and expects to build and run
# a native binary during execution, to populate `constants.xml` with
# platform specifics. Adding support for this is beyond the scope of
# a simple patch, so instead we opt for a "bootstrap mode" where we
# ship the constants template and the `mkconst` helper and rely on
# the kindness of strangers to generate the correct version for us.
# This can be generated as follows:
#   `mkconst < /usr/share/wicked/schema/constants.xml.in \
#            > constants.xml`
# Thanks!
%bcond_with bootstrap # without

Name: %{_cross_os}wicked
Version: 0.6.66
Release: 1%{?dist}
Summary: Network configuration infrastructure
License: GPL-2.0-or-later AND (GPL-2.0-only OR BSD-3-Clause)
URL: https://github.com/openSUSE/wicked
Source0: https://github.com/openSUSE/wicked/archive/version-%{version}.tar.gz

# Default upstream configuration expects various shell-based helpers,
# so we ship a replacement set.
Source10: wicked-tmpfiles.conf
Source11: client.xml
Source12: common.xml
Source13: nanny.xml
Source14: server.xml

%if %{without bootstrap}
Source99: constants.xml
%endif

# upstream fixes

# local hacks
Patch101: 0001-avoid-gcrypt-dependency.patch
Patch102: 0002-exclude-unused-components.patch
Patch103: 0003-ship-mkconst-and-schema-sources-for-runtime-use.patch

BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}libdbus-devel
BuildRequires: %{_cross_os}libiw-devel
BuildRequires: %{_cross_os}libnl-devel
BuildRequires: %{_cross_os}systemd-devel
Requires: %{_cross_os}libdbus
Requires: %{_cross_os}libiw
Requires: %{_cross_os}libnl
Requires: %{_cross_os}systemd

%description
%{summary}.

%package devel
Summary: Files for development using the network configuration infrastructure
Requires: %{name}

%description devel
%{summary}.

%prep
%autosetup -n wicked-version-%{version} -p1

%build
autoreconf -fi

%cross_configure \
  --disable-teamd \
  --enable-systemd \
  --with-compat=redhat \
  --with-pkgconfigdir=%{_cross_pkgconfigdir} \
  --with-dbus-configdir=%{_cross_datadir}/dbus-1/system.d \
  --without-dbus-servicedir \

# "fix" rpath
sed -i 's|^hardcode_libdir_flag_spec=.*|hardcode_libdir_flag_spec=""|g' libtool
sed -i 's|^runpath_var=LD_RUN_PATH|runpath_var=DIE_RPATH_DIE|g' libtool

%make_build

%install
%make_install

# install custom configuration
rm -rf %{buildroot}%{_cross_sysconfdir}/wicked
install -d %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}/wicked
install -p -m 0644 %{S:11} %{S:12} %{S:13} %{S:14} \
  %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}/wicked

install -d %{buildroot}%{_cross_tmpfilesdir}
install -p -m 0644 %{S:10} %{buildroot}%{_cross_tmpfilesdir}/wicked.conf

%if %{without bootstrap}
# install our pre-generated constants
install -p -m 0644 %{S:99} %{buildroot}%{_cross_datadir}/wicked/schema/constants.xml
%endif

%files
%license COPYING
%{_cross_attribution_file}
%{_cross_sbindir}/wicked
%{_cross_sbindir}/wickedd
%{_cross_sbindir}/wickedd-nanny
%if %{with bootstrap}
%{_cross_sbindir}/mkconst
%else
%exclude %{_cross_sbindir}/mkconst
%endif
%dir %{_cross_libexecdir}/wicked
%{_cross_libexecdir}/wicked/*
%{_cross_libdir}/libwicked-%{version}.so
%{_cross_unitdir}/wicked*.service
%{_cross_datadir}/dbus-1/system.d/*.conf
%dir %{_cross_datadir}/wicked
%{_cross_datadir}/wicked/*
%if %{without bootstrap}
%exclude %{_cross_datadir}/wicked/schema/constants.xml.in
%endif
%dir %{_cross_factorydir}%{_cross_sysconfdir}/wicked
%{_cross_factorydir}%{_cross_sysconfdir}/wicked/client.xml
%{_cross_factorydir}%{_cross_sysconfdir}/wicked/common.xml
%{_cross_factorydir}%{_cross_sysconfdir}/wicked/nanny.xml
%{_cross_factorydir}%{_cross_sysconfdir}/wicked/server.xml
%{_cross_tmpfilesdir}/wicked.conf

%files devel
%{_cross_libdir}/libwicked.so
%dir %{_cross_includedir}/wicked
%{_cross_includedir}/wicked/*.h
%{_cross_pkgconfigdir}/*.pc
%exclude %{_cross_libdir}/*.la

%changelog
