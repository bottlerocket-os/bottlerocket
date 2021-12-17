%global debug_package %{nil}

%global project hotpatch-for-apache-log4j2

Name: %{_cross_os}log4j2-hotpatch
Version: 1.3.0
Release: 1%{?dist}
Summary: Tool for hot patching log4j2 vulnerabilities
License: Apache-2.0
URL: https://github.com/corretto/%{project}
Source0: https://github.com/corretto/%{project}/archive/%{version}/%{version}.tar.gz#/%{project}-%{version}.tar.gz

%description
%{summary}.

%prep
%autosetup -n %{project}-%{version}

%build
xmvn --offline clean package

%install
install -d %{buildroot}%{_cross_datadir}/hotdog
install -p -m 0644 target/Log4jHotPatch.jar %{buildroot}%{_cross_datadir}/hotdog

%files
%license LICENSE
%{_cross_attribution_file}
%dir %{_cross_datadir}/hotdog
%{_cross_datadir}/hotdog/Log4jHotPatch.jar

%changelog
