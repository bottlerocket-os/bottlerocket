Name: %{_cross_os}libaudit
Version: 3.0.2
Release: 1%{?dist}
Summary: Library for the audit subsystem
License: GPL-2.0-or-later AND LGPL-2.1-or-later
URL: https://github.com/linux-audit/audit-userspace/
Source0: https://github.com/linux-audit/audit-userspace/archive/v%{version}/audit-userspace-%{version}.tar.gz
Source10: audit-rules.service
Source11: audit.rules
BuildRequires: %{_cross_os}glibc-devel

%description
%{summary}.

%package devel
Summary: Files for development using the library for the audit subsystem
Requires: %{name}

%description devel
%{summary}.

%package -n %{_cross_os}audit
Summary: Tools for the audit subsystem
Requires: %{name}

%description -n %{_cross_os}audit
%{summary}.

%prep
%autosetup -n audit-userspace-%{version} -p1

%build
autoreconf -fi
%cross_configure \
  --disable-listener \
  --disable-gssapi-krb5 \
  --disable-systemd \
  --disable-zos-remote \
  --with-aarch64 \
  --with-warn \
  --without-alpha \
  --without-arm \
  --without-apparmor \
  --without-debug \
  --without-golang \
  --without-libcap-ng \
  --without-prelude \
  --without-python \
  --without-python3 \

# "fix" rpath
sed -i 's|^hardcode_libdir_flag_spec=.*|hardcode_libdir_flag_spec=""|g' libtool
sed -i 's|^runpath_var=LD_RUN_PATH|runpath_var=DIE_RPATH_DIE|g' libtool

%make_build

%install
make DESTDIR=%{buildroot} -C lib install
make DESTDIR=%{buildroot} -C src install
make DESTDIR=%{buildroot} -C auparse install

# fix libtool sadness
for b in auditctl auditd aureport ausearch autrace ; do
  mv %{buildroot}%{_cross_sbindir}/{%{_cross_target}-${b},${b}}
done

install -d %{buildroot}%{_cross_unitdir}
install -p -m 0644 %{S:10} %{buildroot}%{_cross_unitdir}

install -d %{buildroot}%{_cross_datadir}/audit
install -p -m 0644 %{S:11} %{buildroot}%{_cross_datadir}/audit


%files
%license COPYING COPYING.LIB
%{_cross_attribution_file}
%{_cross_libdir}/*.so.*

%files devel
%{_cross_libdir}/*.a
%{_cross_libdir}/*.so
%{_cross_includedir}/*.h
%{_cross_pkgconfigdir}/*.pc
%exclude %{_cross_libdir}/*.la

%files -n %{_cross_os}audit
%{_cross_sbindir}/auditctl
%{_cross_unitdir}/audit-rules.service
%{_cross_datadir}/audit/audit.rules
%exclude %{_cross_sbindir}/auditd
%exclude %{_cross_sbindir}/aureport
%exclude %{_cross_sbindir}/ausearch
%exclude %{_cross_sbindir}/autrace

%changelog
