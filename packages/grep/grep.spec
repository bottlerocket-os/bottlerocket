Name: %{_cross_os}grep
Version: 3.9
Release: 1%{?dist}
Summary: GNU grep utility
URL: https://www.gnu.org/software/grep/
License: GPL-3.0-or-later
Source: https://mirrors.kernel.org/gnu/grep/grep-%{version}.tar.xz
BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}libpcre-devel
Requires: %{_cross_os}libpcre

%description
%{summary}.

%prep
%setup -n grep-%{version}

%build
%cross_configure --without-included-regex --disable-silent-rules
%make_build

%install
%make_install

%files
%license COPYING
%{_cross_bindir}/grep
%{_cross_attribution_file}
# Exclude fgrep and egrep because they are shell scripts
%exclude %{_cross_bindir}/fgrep
%exclude %{_cross_bindir}/egrep
%exclude %{_cross_infodir}
%exclude %{_cross_localedir}
%exclude %{_cross_mandir}
