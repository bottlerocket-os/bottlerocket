Name: %{_cross_os}kmod
Version: 26
Release: 1%{?dist}
Summary: Tools for kernel module loading and unloading
License: LGPLv2+
URL: http://git.kernel.org/?p=utils/kernel/kmod/kmod.git;a=summary
Source0: https://www.kernel.org/pub/linux/utils/kernel/kmod/kmod-%{version}.tar.xz
BuildRequires: gcc-%{_cross_target}
BuildRequires: %{_cross_os}glibc-devel
Requires: %{_cross_os}glibc

%description
%{summary}.

%package devel
Summary: Files for development using the tools for kernel module loading and unloading
Requires: %{name}

%description devel
%{summary}.

%prep
%autosetup -n kmod-%{version} -p1

%build
%cross_configure \
  --without-openssl \
  --without-zlib \
  --without-xz

%make_build

%install
%make_install

for b in depmod insmod lsmod modinfo modprobe rmmod ; do
  ln -s kmod %{buildroot}%{_cross_bindir}/${b}
done

install -d %{buildroot}%{_cross_sbindir}
ln -s ../bin/kmod %{buildroot}%{_cross_sbindir}/modprobe

%files
%{_cross_bindir}/kmod
%{_cross_bindir}/depmod
%{_cross_bindir}/insmod
%{_cross_bindir}/lsmod
%{_cross_bindir}/modinfo
%{_cross_bindir}/modprobe
%{_cross_bindir}/rmmod
%{_cross_sbindir}/modprobe
%{_cross_libdir}/*.so.*
%exclude %{_cross_datadir}/bash-completion
%exclude %{_cross_mandir}

%files devel
%{_cross_libdir}/*.so
%{_cross_includedir}/*.h
%{_cross_pkgconfigdir}/*.pc
%exclude %{_cross_libdir}/*.la

%changelog
