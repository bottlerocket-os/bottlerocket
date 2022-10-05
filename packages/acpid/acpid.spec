Name: %{_cross_os}acpid
Version: 2.0.34
Release: 1%{?dist}
Summary: ACPI event daemon
License: GPL-2.0-or-later
URL: http://sourceforge.net/projects/acpid2/
Source0: https://downloads.sourceforge.net/acpid2/acpid-%{version}.tar.xz
Source1: acpid.service
Source2: power.conf
Patch1: 0001-Remove-shell-dependency-by-only-shutting-down.patch
BuildRequires: %{_cross_os}glibc-devel

%description
%{summary}.

%prep
%autosetup -n acpid-%{version} -p1

%build
%cross_configure
%make_build

%install
%make_install

install -d %{buildroot}%{_cross_unitdir}
install -p -m 0644 %{S:1} %{buildroot}%{_cross_unitdir}

install -d %{buildroot}%{_cross_libdir}/acpid/events
install -p -m 0644 %{S:2} %{buildroot}%{_cross_libdir}/acpid/events/power

%files
%license COPYING
%{_cross_attribution_file}
%{_cross_sbindir}/acpid
%{_cross_unitdir}/acpid.service
%{_cross_libdir}/acpid/
%exclude %{_cross_bindir}/acpi_listen
%exclude %{_cross_sbindir}/kacpimon
%exclude %{_cross_docdir}
%exclude %{_cross_mandir}

%changelog
