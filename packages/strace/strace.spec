Name: %{_cross_os}strace
Version: 6.4
Release: 1%{?dist}
Summary: Linux syscall tracer
License: LGPL-2.1-or-later
URL: https://strace.io/
Source0: https://strace.io/files/%{version}/strace-%{version}.tar.xz
BuildRequires: %{_cross_os}glibc-devel

%description
%{summary}.

%prep
%autosetup -n strace-%{version} -p1

%build
%cross_configure \
  --disable-mpers \

%make_build

%install
%make_install

%files
%license COPYING LGPL-2.1-or-later
%{_cross_attribution_file}
%{_cross_bindir}/strace
%exclude %{_cross_bindir}/strace-graph
%exclude %{_cross_bindir}/strace-log-merge
%exclude %{_cross_mandir}/*

%changelog
