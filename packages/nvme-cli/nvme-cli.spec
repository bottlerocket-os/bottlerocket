Name: %{_cross_os}nvme-cli
Version: 2.9.1
Release: 1%{?dist}
Summary: CLI to interact with NVMe devices
License: LGPL-2.1-only AND GPL-2.0-only AND CC0-1.0 AND MIT
URL: https://github.com/linux-nvme/nvme-cli
Source0: https://github.com/linux-nvme/nvme-cli/archive/v%{version}/nvme-cli-%{version}.tar.gz

BuildRequires: meson
BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}libnvme-devel
Requires: %{_cross_os}libnvme

%description
%{summary}.

%prep
%autosetup -n nvme-cli-%{version} -p1

%build
CONFIGURE_OPTS=(
 -Ddocs=false
 -Ddocs-build=false
 -Djson-c=disabled
)

%cross_meson "${CONFIGURE_OPTS[@]}"
%cross_meson_build

%install
%cross_meson_install
# This is an empty configuration file with comments with examples of how to
# configure the systemd services
rm %{buildroot}%{_sysconfdir}/nvme/discovery.conf

%files
%license LICENSE ccan/licenses/LGPL-2.1 ccan/licenses/BSD-MIT ccan/licenses/CC0
%{_cross_attribution_file}
%{_cross_sbindir}/nvme
%exclude %{_cross_udevrulesdir}
%exclude %{_cross_unitdir}
%exclude %{_cross_datadir}
%exclude %{_cross_prefix}/lib/dracut

%changelog
