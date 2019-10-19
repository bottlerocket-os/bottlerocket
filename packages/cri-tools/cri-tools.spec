%global goproject github.com/kubernetes-sigs
%global gorepo cri-tools
%global goimport %{goproject}/%{gorepo}

%global gover 1.14.0
%global rpmver %{gover}

%global _dwz_low_mem_die_limit 0

Name: %{_cross_os}%{gorepo}
Version: %{rpmver}
Release: 1%{?dist}
Summary: CLI and validation tools for Container Runtime Interface
License: ASL 2.0
URL: https://%{goimport}
Source0: https://%{goimport}/archive/v%{gover}/%{gorepo}-%{gover}.tar.gz
BuildRequires: git
BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}libseccomp-devel
Requires: %{_cross_os}glibc
Requires: %{_cross_os}libseccomp

%description
%{summary}.

%prep
%autosetup -Sgit -n %{gorepo}-%{gover} -p1
%cross_go_setup %{gorepo}-%{gover} %{goproject} %{goimport}

%build
%cross_go_configure %{goimport}
export BUILDTAGS="rpm_crashtraceback seccomp selinux"
go build -buildmode pie -tags="${BUILDTAGS}" -o crictl %{goimport}/cmd/crictl
go test -c -buildmode pie -tags="${BUILDTAGS}" -o critest %{goimport}/cmd/critest

%install
install -d %{buildroot}%{_cross_bindir}
install -p -m 0755 crictl %{buildroot}%{_cross_bindir}
install -p -m 0755 critest %{buildroot}%{_cross_bindir}

%files
%{_cross_bindir}/crictl
%{_cross_bindir}/critest

%changelog
