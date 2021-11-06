%global _cross_first_party 1

Name: %{_cross_os}filesystem
Version: 1.0
Release: 1%{?dist}
Summary: The basic directory layout
License: Apache-2.0 OR MIT
BuildArch: noarch

%description
%{summary}.

%prep

%build

%install
mkdir -p %{buildroot}%{_cross_rootdir}
mkdir -p %{buildroot}%{_cross_prefix}
mkdir -p %{buildroot}%{_cross_bindir}
mkdir -p %{buildroot}%{_cross_sbindir}
mkdir -p %{buildroot}%{_cross_libdir}
mkdir -p %{buildroot}%{_cross_libexecdir}/cni/bin
mkdir -p %{buildroot}%{_cross_includedir}
mkdir -p %{buildroot}%{_cross_sysconfdir}
mkdir -p %{buildroot}%{_cross_datadir}
mkdir -p %{buildroot}%{_cross_infodir}
mkdir -p %{buildroot}%{_cross_mandir}
mkdir -p %{buildroot}%{_cross_localstatedir}
mkdir -p %{buildroot}/{boot,dev,proc,root,run,sys,tmp}
mkdir -p %{buildroot}/{home,local,media,mnt,opt,srv}
mkdir -p %{buildroot}/media/cdrom

ln -s .%{_cross_prefix} %{buildroot}%{_prefix}
ln -s .%{_cross_bindir} %{buildroot}/bin
ln -s .%{_cross_sbindir} %{buildroot}/sbin
ln -s .%{_cross_libdir} %{buildroot}/lib
ln -s .%{_cross_libdir} %{buildroot}/lib64
ln -s lib %{buildroot}%{_cross_prefix}/lib64

%files
%dir %{_cross_rootdir}
%{_cross_rootdir}/*
%dir %{_cross_sysconfdir}
%dir %{_cross_localstatedir}

%{_prefix}
/bin
/sbin
/lib
/lib64

/boot
/dev
/proc
/root
/run
/sys
/tmp

/home
/local
/media
/mnt
/opt
/srv

%changelog
