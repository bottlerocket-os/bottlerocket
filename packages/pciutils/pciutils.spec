Name:           pciutils
Version:        3.9.0
Release:        1
Source:         http://mj.ucw.cz/download/linux/pci/%{name}-%{version}.tar.gz
Copyright:      GNU GPL
Buildroot:      /tmp/%{name}-%{version}-root
ExclusiveOS:    Linux
Summary:        The PCI Utilities
Group:          Utilities/System

%description
This package contains various utilities for inspecting and
setting of devices connected to the PCI bus.

%prep
%setup -q

%build
make OPT="$RPM_OPT_FLAGS"

%install
rm -rf $RPM_BUILD_ROOT
make install PREFIX=$RPM_BUILD_ROOT/usr ROOT=$RPM_BUILD_ROOT/ \
     MANDIR=$RPM_BUILD_ROOT/%{_mandir}

%files
%defattr(0644, root, root, 0755)
%attr(0644, root, man) %{_mandir}/man8/*
%attr(0711, root, root) /usr/sbin/*
%config /usr/share/pci.ids
%doc README ChangeLog pciutils.lsm

%clean
rm -rf $RPM_BUILD_ROOT

%changelog
* Tue Sep 29 1998 Krzysztof G. Baranowski <kgb@knm.org.pl>
[1.07-1]
- build from non-root account against glibc-2.0
- written spec from scratch
