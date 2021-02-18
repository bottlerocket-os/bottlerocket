%global debug_package %{nil}

%global tiniver 0.19.0

Name: %{_cross_os}docker-init
Version: 19.03.15
Release: 1%{?dist}
Summary: Init for containers
License: MIT
URL: https://github.com/krallin/tini
Source0: https://github.com/krallin/tini/archive/v%{tiniver}/tini-%{tiniver}.tar.gz
BuildRequires: cmake
BuildRequires: %{_cross_os}glibc-devel

%description
%{summary}.

%prep
%autosetup -n tini-%{tiniver} -p1

%build
%{cross_cmake} .
%make_build tini-static

%install
install -d %{buildroot}%{_cross_bindir}
install -p -m 0755 tini-static %{buildroot}%{_cross_bindir}/docker-init

%files
%license LICENSE
%{_cross_attribution_file}
%{_cross_bindir}/docker-init

%changelog
