%global goproject github.com/containerd
%global gorepo containerd
%global goimport %{goproject}/%{gorepo}

%global gover 1.3.0
%global rpmver %{gover}

%global _dwz_low_mem_die_limit 0

Name: %{_cross_os}%{gorepo}
Version: %{rpmver}
Release: 1%{?dist}
Summary: An industry-standard container runtime
License: ASL 2.0
URL: https://%{goimport}
Source0: https://%{goimport}/archive/v%{gover}/%{gorepo}-%{gover}.tar.gz
Source1: containerd.service
Source2: containerd-config-toml
Source3: containerd-tmpfiles.conf
BuildRequires: git
BuildRequires: gcc-%{_cross_target}
BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}golang
BuildRequires: %{_cross_os}libseccomp-devel
Requires: %{_cross_os}cni-plugins
Requires: %{_cross_os}glibc
Requires: %{_cross_os}libseccomp
Requires: %{_cross_os}runc
Requires: %{_cross_os}socat
Requires: %{_cross_os}systemd

%description
%{summary}.

%prep
%autosetup -Sgit -n %{gorepo}-%{gover} -p1
%cross_go_setup %{gorepo}-%{gover} %{goproject} %{goimport}

%build
%cross_go_configure %{goimport}
export BUILDTAGS="no_btrfs rpm_crashtraceback seccomp selinux"
for bin in \
  containerd \
  containerd-shim \
  containerd-shim-runc-v1 \
  containerd-shim-runc-v2 \
  ctr ;
do
  go build -buildmode pie -tags="${BUILDTAGS}" -o ${bin} %{goimport}/cmd/${bin}
done

%install
install -d %{buildroot}%{_cross_bindir}
for bin in \
  containerd \
  containerd-shim \
  containerd-shim-runc-v1 \
  containerd-shim-runc-v2 \
  ctr ;
do
  install -p -m 0755 ${bin} %{buildroot}%{_cross_bindir}
done

install -d %{buildroot}%{_cross_unitdir}
install -p -m 0644 %{S:1} %{buildroot}%{_cross_unitdir}/containerd.service

install -d %{buildroot}%{_cross_templatedir}
install -d %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}/containerd
install -p -m 0644 %{S:2} %{buildroot}%{_cross_templatedir}/containerd-config-toml

install -d %{buildroot}%{_cross_tmpfilesdir}
install -p -m 0644 %{S:3} %{buildroot}%{_cross_tmpfilesdir}/containerd.conf

%files
%{_cross_bindir}/containerd
%{_cross_bindir}/containerd-shim
%{_cross_bindir}/containerd-shim-runc-v1
%{_cross_bindir}/containerd-shim-runc-v2
%{_cross_bindir}/ctr
%{_cross_unitdir}/containerd.service
%dir %{_cross_factorydir}%{_cross_sysconfdir}/containerd
%{_cross_templatedir}/containerd-config-toml
%{_cross_tmpfilesdir}/containerd.conf

%changelog
