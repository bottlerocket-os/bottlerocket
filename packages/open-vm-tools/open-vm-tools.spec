%global buildver 21855600

Name: %{_cross_os}open-vm-tools
Version: 12.2.5
Release: 1%{?dist}
Summary: Tools for VMware
License: LGPL-2.1-or-later
URL: https://github.com/vmware/open-vm-tools
Source0: https://github.com/vmware/open-vm-tools/releases/download/stable-%{version}/open-vm-tools-%{version}-%{buildver}.tar.gz
Source1: vmtoolsd.service
Source2: tools.conf
Source3: open-vm-tools-tmpfiles.conf
Patch0001: 0001-no_cflags_werror.patch
Patch0002: 0002-dont-force-cppflags.patch
Patch0003: 0003-Update-shutdown-code-to-work-for-Bottlerocket.patch
BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}libglib-devel
BuildRequires: %{_cross_os}libtirpc-devel
BuildRequires: %{_cross_os}libxcrypt-devel
Requires: %{_cross_os}libglib
Requires: %{_cross_os}libtirpc
Requires: %{_cross_os}libxcrypt

%description
%{summary}.

%package devel
Summary: Files for development using the tools for VMware
Requires: %{name}

%description devel
%{summary}.

%prep
%autosetup -n open-vm-tools-%{version}-%{buildver} -p1

%build
autoreconf -fi
%cross_configure \
  --disable-deploypkg \
  --disable-docs \
  --disable-libappmonitor \
  --disable-multimon \
  --disable-resolutionkms \
  --disable-servicediscovery \
  --disable-tests \
  --disable-vgauth \
  --disable-containerinfo \
  --with-tirpc \
  --with-udev-rules-dir=%{_cross_udevrulesdir} \
  --without-dnet \
  --without-gtk2 \
  --without-gtk3 \
  --without-gtkmm \
  --without-gtkmm3 \
  --without-icu \
  --without-kernel-modules \
  --without-pam \
  --without-ssl \
  --without-x \
  --without-xerces \
  --without-xml2 \
  --without-xmlsec1 \
  --without-xmlsecurity \

# "fix" rpath
sed -i 's|^hardcode_libdir_flag_spec=.*|hardcode_libdir_flag_spec=""|g' libtool
sed -i 's|^runpath_var=LD_RUN_PATH|runpath_var=DIE_RPATH_DIE|g' libtool

%make_build

%install
%make_install

install -d %{buildroot}%{_cross_unitdir}
install -p -m 0644 %{S:1} %{buildroot}%{_cross_unitdir}/vmtoolsd.service

install -d %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}/vmware-tools
install -p -m 0644 %{S:2} %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}/vmware-tools

install -d %{buildroot}%{_cross_tmpfilesdir}
install -p -m 0644 %{S:3} %{buildroot}%{_cross_tmpfilesdir}/open-vm-tools.conf

find %{buildroot} -name '*.la' -delete

%files
%license COPYING LICENSE
%{_cross_attribution_file}
%{_cross_bindir}/vmtoolsd
%{_cross_bindir}/vmware-toolbox-cmd
%{_cross_unitdir}/vmtoolsd.service
%dir %{_cross_factorydir}%{_cross_sysconfdir}/vmware-tools
%{_cross_factorydir}%{_cross_sysconfdir}/vmware-tools/tools.conf
%{_cross_tmpfilesdir}/open-vm-tools.conf

%{_cross_libdir}/*.so.*
%dir %{_cross_libdir}/open-vm-tools
%{_cross_libdir}/open-vm-tools/*
%dir %{_cross_datadir}/open-vm-tools
%{_cross_datadir}/open-vm-tools/*

%exclude %{_cross_bindir}/vmware-checkvm
%exclude %{_cross_bindir}/vmware-namespace-cmd
%exclude %{_cross_bindir}/vmware-rpctool
%exclude %{_cross_bindir}/vmware-xferlogs
%exclude %{_cross_bindir}/vmware-hgfsclient
%exclude %{_cross_sysconfdir}
%exclude %{_cross_udevrulesdir}

%exclude %{_bindir}
%exclude /sbin

%files devel
%{_cross_libdir}/*.a
%{_cross_libdir}/*.so
%dir %{_cross_includedir}/vmGuestLib
%{_cross_includedir}/vmGuestLib/*
%{_cross_pkgconfigdir}/*.pc

%changelog
