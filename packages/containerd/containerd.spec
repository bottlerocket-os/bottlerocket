%global goproject github.com/containerd
%global gorepo containerd
%global goimport %{goproject}/%{gorepo}

%global gover 1.3.7
%global rpmver %{gover}
%global gitrev 8fba4e9a7d01810a393d5d25a3621dc101981175

%global _dwz_low_mem_die_limit 0

Name: %{_cross_os}%{gorepo}
Version: %{rpmver}
Release: 1%{?dist}
Summary: An industry-standard container runtime
License: Apache-2.0
URL: https://%{goimport}
Source0: https://%{goimport}/archive/v%{gover}/%{gorepo}-%{gover}.tar.gz
Source1: containerd.service
Source2: containerd-config-toml_aws-k8s
Source3: containerd-config-toml_aws-dev
Source4: containerd-config-toml_aws-ecs-1
Source5: containerd-tmpfiles.conf
Source1000: clarify.toml

# Upstream patch; can drop when we move to v1.4.0.
Patch0001: 0001-Use-spec-s-mountLabel-when-mounting-the-rootfs.patch

# TODO: submit this upstream.
Patch1001: 1001-cri-reduce-logging-when-no-errors-have-occurred.patch

# Local patches for CRI to use the default system SELinux labels.
# TODO: these need to be reworked for the MCS changes in v1.4.0.
Patch2001: 2001-selinux-add-DefaultLabels-helper.patch
Patch2002: 2002-cri-use-default-SELinux-labels-as-a-fallback.patch

# Local patch for CRI to override the default RLIMIT_NOFILE.
# TODO: submit this upstream, including a unit test.
Patch3001: 3001-cri-set-default-RLIMIT_NOFILE.patch

# Upstream patches; can drop when we move to 1.4.1
Patch4001: 4001-Exit-signal-forward-if-process-not-found.patch
Patch4002: 4002-Ignore-SIGURG-signals-in-signal-forwarder.patch

# Upstream patch; can drop when we move to 1.4.1
Patch5001: 5001-Always-consume-shim-logs.patch

# Upstream patch; can drop when we move to 1.3.9 or 1.4.2
Patch6001: 6001-CVE-2020-15257.patch

BuildRequires: git
BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}libseccomp-devel
Requires: %{_cross_os}cni-plugins
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
export BUILDTAGS="no_btrfs seccomp selinux"
export LD_VERSION="-X github.com/containerd/containerd/version.Version=%{gover}+bottlerocket"
export LD_REVISION="-X github.com/containerd/containerd/version.Revision=%{gitrev}"
for bin in \
  containerd \
  containerd-shim \
  containerd-shim-runc-v1 \
  containerd-shim-runc-v2 \
  ctr ;
do
  go build \
     -buildmode=pie \
     -ldflags="-linkmode=external ${LD_VERSION} ${LD_REVISION}" \
     -tags="${BUILDTAGS}" \
     -o ${bin} \
     %{goimport}/cmd/${bin}
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
install -p -m 0644 %{S:2} %{S:3} %{S:4} %{buildroot}%{_cross_templatedir}

install -d %{buildroot}%{_cross_tmpfilesdir}
install -p -m 0644 %{S:5} %{buildroot}%{_cross_tmpfilesdir}/containerd.conf

%cross_scan_attribution --clarify %{S:1000} go-vendor vendor

%files
%license LICENSE NOTICE
%{_cross_attribution_file}
%{_cross_attribution_vendor_dir}
%{_cross_bindir}/containerd
%{_cross_bindir}/containerd-shim
%{_cross_bindir}/containerd-shim-runc-v1
%{_cross_bindir}/containerd-shim-runc-v2
%{_cross_bindir}/ctr
%{_cross_unitdir}/containerd.service
%dir %{_cross_factorydir}%{_cross_sysconfdir}/containerd
%{_cross_templatedir}/containerd-config-toml*
%{_cross_tmpfilesdir}/containerd.conf

%changelog
