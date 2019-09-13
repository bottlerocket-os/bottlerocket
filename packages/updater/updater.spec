%global workspace_name updater
%global workspace_dir %{_builddir}/workspaces/%{workspace_name}
%undefine _debugsource_packages

Name: %{_cross_os}%{workspace_name}
Version: 0.0
Release: 0%{?dist}
Summary: Thar updater packages
License: FIXME
Source1: root.json
Source2: updog-toml
Source3: updog.conf
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
%setup -T -c
%cargo_prep

%build
for p in signpost updog ; do
  %cargo_build --path %{workspace_dir}/${p}
done

%install
install -d %{buildroot}%{_cross_bindir}
install -p -m 0755 bin/signpost %{buildroot}%{_cross_bindir}
install -p -m 0755 bin/updog %{buildroot}%{_cross_bindir}

install -d %{buildroot}/%{_cross_datadir}/updog
install -m 0644 -t %{buildroot}/%{_cross_datadir}/updog %{SOURCE1}

install -d %{buildroot}%{_cross_templatedir}
install -p -m 0644 %{SOURCE2} %{buildroot}%{_cross_templatedir}/updog-toml

install -d %{buildroot}%{_cross_tmpfilesdir}
install -p -m 0644 %{SOURCE3} %{buildroot}%{_cross_tmpfilesdir}/updog.conf

%files -n %{_cross_os}signpost
%{_cross_bindir}/signpost

%files -n %{_cross_os}updog
%{_cross_bindir}/updog
%{_cross_datadir}/updog
%{_cross_tmpfilesdir}/updog.conf
%dir %{_cross_templatedir}
%{_cross_templatedir}/updog-toml

%changelog
