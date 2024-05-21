%global _cross_first_party 1
%undefine _debugsource_packages

%global extension_name metrics

Name: %{_cross_os}settings-%{extension_name}
Version: 0.0
Release: 0%{?dist}
Summary: settings-%{extension_name}
License: Apache-2.0 OR MIT
URL: https://github.com/bottlerocket-os/bottlerocket

BuildRequires: %{_cross_os}glibc-devel

%description
%{summary}.

%prep
%setup -T -c
%cargo_prep

%build
%cargo_build --manifest-path %{_builddir}/sources/Cargo.toml \
    -p settings-extension-%{extension_name}

%install
install -d %{buildroot}%{_cross_libexecdir}
install -p -m 0755 \
    ${HOME}/.cache/%{__cargo_target}/release/settings-extension-%{extension_name} \
    %{buildroot}%{_cross_libexecdir}

install -d %{buildroot}%{_cross_libexecdir}/settings
ln -sf \
    ../settings-extension-%{extension_name} \
    %{buildroot}%{_cross_libexecdir}/settings/%{extension_name}

%files
%{_cross_libexecdir}/settings-extension-%{extension_name}
%{_cross_libexecdir}/settings/%{extension_name}
