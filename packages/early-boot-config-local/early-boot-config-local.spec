%global _cross_first_party 1
%undefine _debugsource_packages

%global user_data_provider local

Name: %{_cross_os}early-boot-config-%{user_data_provider}
Version: 0.0
Release: 0%{?dist}
Summary: %{user_data_provider}-provider
License: Apache-2.0 OR MIT
URL: https://github.com/bottlerocket-os/bottlerocket

BuildRequires: %{_cross_os}glibc-devel

%description
%{summary}.

# Includes just the user data provider binary
%package -n %{_cross_os}local-user-data-client
Summary: Local user data client
%description -n %{_cross_os}local-user-data-client
%{summary}.

# Symlinks the binary to the appropriate .d directory as 20-
%package -n %{_cross_os}local-user-data-provider
Summary: Local user data provider config
%description -n %{_cross_os}local-user-data-provider
%{summary}.

%prep
%setup -T -c
%cargo_prep

%build
%cargo_build --manifest-path %{_builddir}/sources/Cargo.toml \
    -p early-boot-config --bin %{user_data_provider}-user-data-provider

%install
install -d %{buildroot}%{_cross_libexecdir}/early-boot-config/bin
install -p -m 0755 \
    ${HOME}/.cache/%{__cargo_target}/release/%{user_data_provider}-user-data-provider \
    %{buildroot}%{_cross_libexecdir}/early-boot-config/bin

install -d %{buildroot}%{_cross_datadir}/early-boot-config/data-providers.d
ln -sf \
    ../../../libexec/early-boot-config/bin/%{user_data_provider}-user-data-provider \
    %{buildroot}%{_cross_datadir}/early-boot-config/data-providers.d/20-%{user_data_provider}


%files -n %{_cross_os}local-user-data-client
%{_cross_libexecdir}/early-boot-config/bin/%{user_data_provider}-user-data-provider

%files -n %{_cross_os}local-user-data-provider
%{_cross_datadir}/early-boot-config/data-providers.d/20-%{user_data_provider}