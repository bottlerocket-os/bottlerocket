Name: %{_cross_os}strace
Version: 5.4
Release: 1%{?dist}
Summary: Linux syscall tracer
License: LGPLv2.1+
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
%{_cross_bindir}/strace
%exclude %{_cross_bindir}/strace-graph
%exclude %{_cross_bindir}/strace-log-merge
%exclude %{_cross_mandir}/*

%changelog
