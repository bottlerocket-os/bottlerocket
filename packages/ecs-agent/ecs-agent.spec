%global goproject github.com/aws
%global gorepo amazon-ecs-agent
%global goimport %{goproject}/%{gorepo}

%global gover 1.41.0
# git rev-parse --short=8
%global gitrev 3776bee9

# Construct reproducible tar archives
# See https://reproducible-builds.org/docs/archives/
%global source_date_epoch 1234567890
%global tar_cf tar --sort=name --mtime="@%{source_date_epoch}" --owner=0 --group=0 --numeric-owner -cf

Name: %{_cross_os}ecs-agent
Version: %{gover}
Release: 1%{?dist}
Summary: Amazon Elastic Container Service agent
License: Apache-2.0
URL: https://%{goimport}
Source0: https://%{goimport}/archive/v%{gover}.tar.gz
Source1: ecs.service
Source2: ecs-tmpfiles.conf
Source3: ecs-sysctl.conf
Source4: pause-image-VERSION
Source5: pause-config.json
Source6: pause-manifest.json
Source7: pause-repositories

# Upstream: https://github.com/aws/amazon-ecs-agent/pull/2513
# Upstream status: Merged
Patch0001: 0001-engine-move-default-image-exclusions.patch

# Bottlerocket-specific - filesystem location of the pause image
Patch0002: 0002-bottlerocket-default-filesystem-locations.patch

# Bottlerocket-specific - version data can be set with linker options
Patch0003: 0003-bottlerocket-version-values-settable-with-linker.patch

BuildRequires: %{_cross_os}glibc-devel

Requires: %{_cross_os}docker-engine
Requires: %{_cross_os}iptables

%description
%{summary}.

%prep
%autosetup -Sgit -n %{gorepo}-%{gover} -p1
%cross_go_setup %{gorepo}-%{gover} %{goproject} %{goimport}

%build
# Build the agent
%cross_go_configure %{goimport}
PAUSE_CONTAINER_IMAGE_NAME="amazon/amazon-ecs-pause"
PAUSE_CONTAINER_IMAGE_TAG="bottlerocket"
LD_PAUSE_CONTAINER_NAME="-X github.com/aws/amazon-ecs-agent/agent/config.DefaultPauseContainerImageName=${PAUSE_CONTAINER_IMAGE_NAME}"
LD_PAUSE_CONTAINER_TAG="-X github.com/aws/amazon-ecs-agent/agent/config.DefaultPauseContainerTag=${PAUSE_CONTAINER_IMAGE_TAG}"
LD_VERSION="-X github.com/aws/amazon-ecs-agent/agent/version.Version=%{gover}"
LD_GIT_REV="-X github.com/aws/amazon-ecs-agent/agent/version.GitShortHash=%{gitrev}"
go build -a \
  -buildmode pie \
  -ldflags "${LD_PAUSE_CONTAINER_NAME} ${LD_PAUSE_CONTAINER_TAG} ${LD_VERSION} ${LD_GIT_REV}" \
  -o amazon-ecs-agent \
  ./agent

# Build the pause container
(
  set -x
  cd misc/pause-container/buildPause
  mkdir -p rootfs/usr/bin
  make BIN=rootfs/usr/bin/pause GCC=%{_cross_triple}-musl-gcc CFLAGS="%{_cross_cflags} -static"

  # Construct image
  mkdir -p image/rootfs
  %tar_cf image/rootfs/layer.tar -C rootfs .
  DIGEST=$(sha256sum image/rootfs/layer.tar | sed -e 's/ .*//')
  install -m 0644 %{S:4} image/rootfs/VERSION
  install -m 0644 %{S:5} image/config.json
  sed -i "s/~~digest~~/${DIGEST}/" image/config.json
  install -m 0644 %{S:6} image/manifest.json
  install -m 0644 %{S:7} image/repositories
  %tar_cf ../../../amazon-ecs-pause.tar -C image .
)

%install
install -D -p -m 0755 amazon-ecs-agent %{buildroot}%{_cross_bindir}/amazon-ecs-agent
install -D -p -m 0644 amazon-ecs-pause.tar %{buildroot}%{_cross_libdir}/amazon-ecs-agent/amazon-ecs-pause.tar

install -D -p -m 0644 %{S:1} %{buildroot}%{_cross_unitdir}/ecs.service
install -D -p -m 0644 %{S:2} %{buildroot}%{_cross_tmpfilesdir}/ecs.conf
install -D -p -m 0644 %{S:3} %{buildroot}%{_cross_sysctldir}/90-ecs.conf

%cross_scan_attribution go-vendor agent/vendor

%files
%{_cross_attribution_file}
%{_cross_attribution_vendor_dir}
%license LICENSE NOTICE THIRD-PARTY
%{_cross_bindir}/amazon-ecs-agent
%{_cross_unitdir}/ecs.service
%{_cross_tmpfilesdir}/ecs.conf
%{_cross_sysctldir}/90-ecs.conf
%{_cross_libdir}/amazon-ecs-agent/amazon-ecs-pause.tar

%changelog
