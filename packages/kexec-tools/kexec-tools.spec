Name: %{_cross_os}kexec-tools
Version: 2.0.26
Release: 1%{?dist}
Summary: Linux tool to load kernels from the running system
License: GPL-2.0-or-later AND GPL-2.0-only
URL: https://www.kernel.org/doc/html/latest/admin-guide/kdump/kdump.html
Source0: https://kernel.org/pub/linux/utils/kernel/kexec/kexec-tools-%{version}.tar.xz

BuildRequires: %{_cross_os}glibc-devel

%description
%{summary}.

%prep
%setup -n kexec-tools-%{version}
rm -f kexec-tools.spec.in

%build
%cross_configure
%make_build

%install
%make_install

%files
%license COPYING
%{_cross_attribution_file}
%{_cross_sbindir}/kexec
%exclude %{_cross_libdir}
%exclude %{_cross_mandir}
%exclude %{_cross_sbindir}/vmcore-dmesg
