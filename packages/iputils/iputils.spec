Name: %{_cross_os}iputils
Version: 20221126
Release: 1%{?dist}
Summary: A set of network monitoring tools
License: GPL-2.0-or-later AND BSD-3-Clause
URL: https://github.com/iputils/iputils
Source0: https://github.com/iputils/iputils/archive/%{version}.tar.gz#/iputils-%{version}.tar.gz

BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}libcap-devel
Requires: %{_cross_os}libcap

%description
%{summary}.

%prep
%autosetup -n iputils-%{version} -p1

%build
CONFIGURE_OPTS=(
 -DUSE_CAP=true
 -DUSE_GETTEXT=false
 -DUSE_IDN=false

 -DBUILD_ARPING=true
 -DBUILD_PING=true
 -DBUILD_TRACEPATH=true

 -DBUILD_CLOCKDIFF=false

 -DBUILD_MANS=false
 -DBUILD_HTML_MANS=false
)

%cross_meson "${CONFIGURE_OPTS[@]}"
%cross_meson_build

%install
%cross_meson_install

%files
%license LICENSE Documentation/LICENSE.GPL2 Documentation/LICENSE.BSD3
%{_cross_attribution_file}
%attr(0755,root,root) %caps(cap_net_raw=p) %{_cross_bindir}/arping
%attr(0755,root,root) %caps(cap_net_raw=p cap_net_admin=p) %{_cross_bindir}/ping
%{_cross_bindir}/tracepath

%changelog
