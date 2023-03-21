Name: %{_cross_os}keyutils
Version: 1.6.1
Release: 1%{?dist}
Summary: Linux key management utilities
License: GPL-2.0-or-later AND GPL-2.1-or-later
Url: http://people.redhat.com/~dhowells/keyutils/
Source0: http://people.redhat.com/~dhowells/keyutils/keyutils-%{version}.tar.bz2
Source1: keyutils-tmpfiles.conf
Source2: request-key.conf

BuildRequires: %{_cross_os}glibc-devel

%description
%{summary}.

%package devel
Summary: Development package for building Linux key management utilities

%description devel
%{summary}.

%prep
%setup -n keyutils-%{version} -q

%global keyutilsmake \
%set_cross_build_flags \\\
export CC=%{_cross_target}-gcc ; \
%make_build \\\
  NO_ARLIB=1 \\\
  ETCDIR=%{_cross_sysconfdir} \\\
  LIBDIR=%{_cross_libdir} \\\
  USRLIBDIR=%{_cross_libdir} \\\
  BINDIR=%{_cross_bindir} \\\
  SBINDIR=%{_cross_sbindir} \\\
  MANDIR=%{_cross_mandir} \\\
  INCLUDEDIR=%{_cross_includedir} \\\
  SHAREDIR=%{_cross_datadir}/keyutils \\\
  RELEASE=.%{release} \\\
  NO_GLIBC_KEYERR=1 \\\
  CC="${CC}" \\\
  CFLAGS="${CFLAGS}" \\\
  LDFLAGS="${LDFLAGS}" \\\
  DESTDIR=%{buildroot} \\\
%{nil}


%build
%keyutilsmake

%install
%keyutilsmake install

install -d %{buildroot}%{_cross_tmpfilesdir}
install -d %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}

install -p -m 0644 %{S:1} %{buildroot}%{_cross_tmpfilesdir}/keyutils.conf
install -p -m 0644 %{S:2} %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}/request-key.conf

%files
%{_cross_attribution_file}
%license LICENCE.GPL LICENCE.LGPL
%{_cross_tmpfilesdir}/keyutils.conf
%{_cross_bindir}/keyctl
%{_cross_sbindir}/key.dns_resolver
%{_cross_sbindir}/request-key
%{_cross_datadir}/keyutils
%{_cross_libdir}/libkeyutils.so.*
%{_cross_factorydir}%{_cross_sysconfdir}/request-key.conf

%exclude %{_cross_mandir}
%exclude %{_cross_sysconfdir}/request-key.conf

%files devel
%{_cross_libdir}/libkeyutils.so
%{_cross_includedir}/keyutils.h
%{_cross_libdir}/pkgconfig/libkeyutils.pc

%changelog
