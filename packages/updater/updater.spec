%global workspace_name updater

Name: %{_cross_os}%{workspace_name}
Version: 0.0
Release: 0%{?dist}
Summary: Thar updater packages
License: FIXME
Source0: %{workspace_name}.crate
Source1: root.json
Source2: updog.toml
Source3: updog.conf
%cargo_bundle_crates -n %{workspace_name} -t 0
BuildRequires: gcc-%{_cross_target}
BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}rust
Requires: %{_cross_os}glibc

%description
%{summary}.

%package -n %{_cross_os}signpost
Summary: Thar GPT priority querier/switcher
%description -n %{_cross_os}signpost
%{summary}.

%package -n %{_cross_os}updog
Summary: Thar updater CLI
%description -n %{_cross_os}updog
not much what's up with you

%prep
%setup -qn %{workspace_name}
%cargo_prep

%build
%cargo_build --all

%install
%cargo_install -p signpost
%cargo_install -p updog

install -d %{buildroot}/%{_cross_datadir}/updog
install -m 0644 -t %{buildroot}/%{_cross_datadir}/updog %{SOURCE1}

install -d %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}
install -p -m 0644 %{SOURCE2} %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}/updog.toml

install -d %{buildroot}%{_cross_tmpfilesdir}
install -p -m 0644 %{SOURCE3} %{buildroot}%{_cross_tmpfilesdir}/updog.conf

%check
%cargo_test --all

%files -n %{_cross_os}signpost
%{_cross_bindir}/signpost

%files -n %{_cross_os}updog
%{_cross_bindir}/updog
%{_cross_datadir}/updog
%{_cross_factorydir}%{_cross_sysconfdir}/updog.toml
%{_cross_tmpfilesdir}/updog.conf

%changelog
