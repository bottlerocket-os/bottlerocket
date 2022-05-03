%global _cross_first_party 1
%global workspace_name ecs-gpu-init

Name: %{_cross_os}%{workspace_name}
Version: 0.0
Release: 0%{?dist}
Summary: Tool to generate the ECS agent's GPU configuration
License: Apache-2.0 OR MIT
Source1: ecs-gpu-init.service
Source2: ecs-gpu-init-tmpfiles.conf
BuildRequires: %{_cross_os}glibc-devel

%description
%{summary}.

%prep
%setup -T -c
cp -r %{_builddir}/sources/%{workspace_name}/* .

%build
%set_cross_go_flags
# We don't set `-Wl,-z,now`, because the binary uses lazy loading
# to load the NVIDIA libraries in the host
export CGO_LDFLAGS="-Wl,-z,relro"
go build -buildmode=pie -ldflags="${GOLDFLAGS}" -o ecs-gpu-init ./cmd/ecs-gpu-init

%install
install -d %{buildroot}%{_cross_bindir}
install -p -m 0755 ecs-gpu-init %{buildroot}%{_cross_bindir}

install -d %{buildroot}%{_cross_unitdir}
install -D -p -m 0644 %{S:1} %{buildroot}%{_cross_unitdir}

install -D -p -m 0644 %{S:2} %{buildroot}%{_cross_tmpfilesdir}/ecs-gpu-init.conf

%cross_scan_attribution go-vendor vendor

%files
%{_cross_attribution_vendor_dir}
%{_cross_bindir}/ecs-gpu-init
%{_cross_unitdir}/ecs-gpu-init.service
%{_cross_tmpfilesdir}/ecs-gpu-init.conf

%changelog
