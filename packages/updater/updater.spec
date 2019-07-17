%global workspace_name updater

Name: %{_cross_os}%{workspace_name}
Version: 0.0
Release: 0%{?dist}
Summary: Thar updater packages
License: FIXME
Source0: %{workspace_name}.crate
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

%prep
%setup -qn %{workspace_name}
%cargo_prep

%build
%cargo_build --all

%install
%cargo_install -p signpost

%check
%cargo_test --all

%files -n %{_cross_os}signpost
%{_cross_bindir}/signpost

%changelog
