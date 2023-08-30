%global goproject github.com/containerd
%global gorepo containerd
%global goimport %{goproject}/%{gorepo}

%global gover 1.6.23
%global rpmver %{gover}
%global gitrev 1e1ea6e986c6c86565bc33d52e34b81b3e2bc71f

%global _dwz_low_mem_die_limit 0

Name: %{_cross_os}%{gorepo}
Version: %{rpmver}
Release: 1%{?dist}
Summary: An industry-standard container runtime
License: Apache-2.0
URL: https://%{goimport}
Source0: https://%{goimport}/archive/v%{gover}/%{gorepo}-%{gover}.tar.gz
Source1: containerd.service
Source2: containerd-config-toml_k8s_containerd_sock
Source3: containerd-config-toml_basic
Source4: containerd-config-toml_k8s_nvidia_containerd_sock
Source5: containerd-tmpfiles.conf
Source6: containerd-cri-base-json

# Mount for writing containerd configuration
Source100: etc-containerd.mount

# Create container storage mount point.
Source110: prepare-var-lib-containerd.service

Source1000: clarify.toml

BuildRequires: git
BuildRequires: %{_cross_os}glibc-devel
Requires: %{_cross_os}runc
Requires: %{_cross_os}pigz

%description
%{summary}.

%prep
%autosetup -Sgit -n %{gorepo}-%{gover} -p1
%cross_go_setup %{gorepo}-%{gover} %{goproject} %{goimport}

%build
%cross_go_configure %{goimport}
export BUILDTAGS="no_btrfs selinux"
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
     -ldflags="${GOLDFLAGS} ${LD_VERSION} ${LD_REVISION}" \
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
install -p -m 0644 %{S:1} %{S:100} %{S:110} %{buildroot}%{_cross_unitdir}

install -d %{buildroot}%{_cross_templatedir}
install -d %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}/containerd
install -p -m 0644 %{S:2} %{S:3} %{S:4} %{S:6} %{buildroot}%{_cross_templatedir}

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
%{_cross_unitdir}/etc-containerd.mount
%{_cross_unitdir}/prepare-var-lib-containerd.service
%dir %{_cross_factorydir}%{_cross_sysconfdir}/containerd
%{_cross_templatedir}/containerd-config-toml*
%{_cross_templatedir}/containerd-cri-base-json
%{_cross_tmpfilesdir}/containerd.conf

%changelog
