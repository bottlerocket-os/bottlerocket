%global workspace_name host-ctr
%global systemd_systemdir %{_cross_libdir}/systemd/system

Name: %{_cross_os}%{workspace_name}
Version: 0.0
Release: 0%{?dist}
Summary: Thar host container runner
License: FIXME
BuildRequires: %{_cross_os}glibc-devel

%description
%{summary}.

%prep
%setup -T -c
cp -r %{_builddir}/workspaces/%{workspace_name}/cmd/host-ctr/* .

%build
%set_cross_go_flags
export BUILDTAGS="rpm_crashtraceback"
go build -buildmode=pie -tags="${BUILDTAGS}" -o host-ctr

%install
install -d %{buildroot}%{_cross_bindir}
install -p -m 0755 host-ctr %{buildroot}%{_cross_bindir}

%files
%{_cross_bindir}/host-ctr

%changelog
