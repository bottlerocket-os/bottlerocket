%global goproject github.com/docker
%global gorepo cli
%global goimport %{goproject}/%{gorepo}

%global gover 25.0.2
%global rpmver %{gover}
%global gitrev 29cf62922279a56e122dc132eb84fe98f61d5950

%global source_date_epoch 1492525740

%global _dwz_low_mem_die_limit 0

Name: %{_cross_os}docker-%{gorepo}
Version: %{rpmver}
Release: 1%{?dist}
Summary: Docker CLI
License: Apache-2.0
URL: https://%{goimport}
Source0: https://%{goimport}/archive/v%{gover}/cli-%{gover}.tar.gz
Source1000: clarify.toml

BuildRequires: git
BuildRequires: %{_cross_os}glibc-devel
Requires: %{name}(binaries)

%description
%{summary}.

%package bin
Summary: Docker CLI binaries
Provides: %{name}(binaries)
Requires: (%{_cross_os}image-feature(no-fips) and %{name})
Conflicts: (%{_cross_os}image-feature(fips) or %{name}-fips-bin)

%description bin
%{summary}.

%package fips-bin
Summary: Docker CLI binaries, FIPS edition
Provides: %{name}(binaries)
Requires: (%{_cross_os}image-feature(fips) and %{name})
Conflicts: (%{_cross_os}image-feature(no-fips) or %{name}-bin)

%description fips-bin
%{summary}.

%prep
%autosetup -Sgit -n %{gorepo}-%{gover} -p1
%cross_go_setup %{gorepo}-%{gover} %{goproject} %{goimport}

%build
%cross_go_configure %{goimport}
LD_VERSION="-X github.com/docker/cli/cli/version.Version=%{gover}"
LD_GIT_REV="-X github.com/docker/cli/cli/version.GitCommit=%{gitrev}"
LD_PLATFORM="-X \"github.com/docker/cli/cli/version.PlatformName=Docker Engine - Community\""
BUILDTIME=$(date -u -d "@%{source_date_epoch}" --rfc-3339 ns 2> /dev/null | sed -e 's/ /T/')
LD_BUILDTIME="-X github.com/docker/cli/cli/version.BuildTime=${BUILDTIME}"

declare -a BUILD_ARGS
BUILD_ARGS=(
  -ldflags="${GOLDFLAGS} ${LD_VERSION} ${LD_GIT_REV} ${LD_PLATFORM} ${LD_BUILDTIME}"
)

go build "${BUILD_ARGS[@]}" -o docker %{goimport}/cmd/docker
gofips build "${BUILD_ARGS[@]}" -o fips/docker %{goimport}/cmd/docker

%install
install -d %{buildroot}%{_cross_bindir}
install -p -m 0755 docker %{buildroot}%{_cross_bindir}

install -d %{buildroot}%{_cross_fips_bindir}
install -p -m 0755 fips/docker %{buildroot}%{_cross_fips_bindir}

%cross_scan_attribution --clarify %{S:1000} go-vendor vendor

%files
%license LICENSE NOTICE
%{_cross_attribution_file}
%{_cross_attribution_vendor_dir}

%files bin
%{_cross_bindir}/docker

%files fips-bin
%{_cross_fips_bindir}/docker

%changelog
