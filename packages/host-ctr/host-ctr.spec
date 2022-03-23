%global _cross_first_party 1
%global workspace_name host-ctr

Name: %{_cross_os}%{workspace_name}
Version: 0.0
Release: 0%{?dist}
Summary: Bottlerocket host container runner
License: Apache-2.0 OR MIT
URL: https://github.com/bottlerocket-os/bottlerocket
BuildRequires: %{_cross_os}glibc-devel
Requires: %{_cross_os}containerd

Source10: host-containerd.service
Source11: host-containerd-tmpfiles.conf
Source12: host-containerd-config.toml

# Mount for writing host-ctr configuration
Source100: etc-host-containers.mount.in

Source1000: clarify.toml

%description
%{summary}.

%prep
%setup -T -c
cp -r %{_builddir}/sources/%{workspace_name}/* .

%build
%set_cross_go_flags
go build -buildmode=pie -ldflags="${GOLDFLAGS}" -o host-ctr ./cmd/host-ctr

%install
install -d %{buildroot}%{_cross_bindir}
install -p -m 0755 host-ctr %{buildroot}%{_cross_bindir}

install -d %{buildroot}%{_cross_unitdir}
install -p -m 0644 %{S:10} %{buildroot}%{_cross_unitdir}
ETC_HOST_CONTAINERS=$(systemd-escape --path /etc/host-containers)
install -p -m 0644 %{S:100} %{buildroot}%{_cross_unitdir}/${ETC_HOST_CONTAINERS}.mount

install -d %{buildroot}%{_cross_tmpfilesdir}
install -p -m 0644 %{S:11} %{buildroot}%{_cross_tmpfilesdir}/host-containerd.conf

install -d %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}/host-containerd
install -p -m 0644 %{S:12} %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}/host-containerd/config.toml

%cross_scan_attribution --clarify %{S:1000} go-vendor vendor

%files
%{_cross_attribution_vendor_dir}
%{_cross_bindir}/host-ctr
%{_cross_unitdir}/host-containerd.service
%{_cross_unitdir}/*.mount
%{_cross_tmpfilesdir}/host-containerd.conf
%{_cross_factorydir}%{_cross_sysconfdir}/host-containerd/config.toml

%changelog
