Name: %{_cross_os}tcpdump
Version: 4.99.1
Release: 1%{?dist}
Summary: Network monitoring tool
License: BSD-3-Clause
URL: http://www.tcpdump.org
Source0: http://www.tcpdump.org/release/tcpdump-%{version}.tar.gz
BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}libpcap-devel
Requires: %{_cross_os}libpcap

%description
%{summary}.

%prep
%autosetup -n tcpdump-%{version} -p1

%build
%cross_configure

%make_build

%install
install -d %{buildroot}%{_cross_bindir}
install -p -m 0755 tcpdump %{buildroot}%{_cross_bindir}

%files
%license LICENSE
%{_cross_attribution_file}
%{_cross_bindir}/tcpdump

%changelog
