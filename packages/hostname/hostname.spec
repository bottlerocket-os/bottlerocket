Name: %{_cross_os}hostname
Version: 3.21
Release: 1%{?dist}
Summary: Utility to show or set hostname
License: GPLv2+
URL: http://packages.qa.debian.org/h/hostname.html
Source0: http://http.us.debian.org/debian/pool/main/h/hostname/hostname_%{version}.tar.gz
BuildRequires: gcc-%{_cross_target}
BuildRequires: %{_cross_os}glibc-devel
Requires: %{_cross_os}glibc

%description
%{summary}.

%prep
%autosetup -n hostname -p1

%build
make \
  CC="%{_cross_target}-gcc" \
  CFLAGS="%{_cross_cflags} -D_GNU_SOURCE" \
  LDFLAGS="%{_cross_ldflags}" \

%install
install -d %{buildroot}%{_cross_bindir}
install -p -m 0755 hostname %{buildroot}%{_cross_bindir}

%files
%{_cross_bindir}/hostname

%changelog
