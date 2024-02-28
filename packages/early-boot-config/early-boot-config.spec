%global _cross_first_party 1
%undefine _debugsource_packages

Name: %{_cross_os}early-boot-config
Version: 0.0
Release: 0%{?dist}
Summary: early-boot-config
License: Apache-2.0 OR MIT
URL: https://github.com/bottlerocket-os/bottlerocket

Source100: early-boot-config.service

BuildRequires: %{_cross_os}glibc-devel

%description
%{summary}.

%package -n %{_cross_os}early-boot-config-common
Summary: early-boot-config binary
%description -n %{_cross_os}early-boot-config-common
%{summary}.

%package -n %{_cross_os}early-boot-config-data-providers
Summary: early-boot-config data providers directory
%description -n %{_cross_os}early-boot-config-data-providers
%{summary}.

%package -n %{_cross_os}early-boot-config-local
Summary: local-provider
Requires: %{_cross_os}early-boot-config-data-providers
%description -n %{_cross_os}early-boot-config-local
%{summary}.

%package -n %{_cross_os}early-boot-config-aws
Summary: early-boot-config package for AWS
Provides: %{_cross_os}early-boot-config
Requires: %{_cross_os}early-boot-config-common
Requires: %{_cross_os}early-boot-config-data-providers
Requires: %{_cross_os}early-boot-config-local
%description -n %{_cross_os}early-boot-config-aws
%{summary}.

%ifarch x86_64
%package -n %{_cross_os}early-boot-config-vmware
Summary: early-boot-config package for vmware
Provides: %{_cross_os}early-boot-config
Requires: %{_cross_os}early-boot-config-common
Requires: %{_cross_os}early-boot-config-data-providers
Requires: %{_cross_os}early-boot-config-local
%description -n %{_cross_os}early-boot-config-vmware
%{summary}.
%endif

%package -n %{_cross_os}early-boot-config-metal
Summary: early-boot-config package for metal
Provides: %{_cross_os}early-boot-config
Requires: %{_cross_os}early-boot-config-common
Requires: %{_cross_os}early-boot-config-data-providers
Requires: %{_cross_os}early-boot-config-local
%description -n %{_cross_os}early-boot-config-metal
%{summary}.

%prep
%setup -T -c
%cargo_prep

%build
%cargo_build --manifest-path %{_builddir}/sources/Cargo.toml \
    -p early-boot-config \
    --bin early-boot-config \
    --bin ec2-identity-doc-provider \
    --bin ec2-imds-provider \
    --bin local-user-data-provider \
    --bin local-defaults-provider \
    --bin local-overrides-provider \

%ifarch x86_64
%cargo_build --manifest-path %{_builddir}/sources/Cargo.toml \
    -p vmware-cd-rom-user-data-provider \
    -p vmware-guestinfo-user-data-provider
%endif

%install
install -d %{buildroot}%{_cross_bindir}
install -p -m 0755 ${HOME}/.cache/%{__cargo_target}/release/early-boot-config %{buildroot}%{_cross_bindir}

install -d %{buildroot}%{_cross_unitdir}
install -p -m 0644 %{S:100} %{buildroot}%{_cross_unitdir}

install -d %{buildroot}%{_cross_libexecdir}/early-boot-config/bin
install -p -m 0755 \
    ${HOME}/.cache/%{__cargo_target}/release/ec2-identity-doc-provider \
    ${HOME}/.cache/%{__cargo_target}/release/ec2-imds-provider \
    ${HOME}/.cache/%{__cargo_target}/release/local-user-data-provider \
    ${HOME}/.cache/%{__cargo_target}/release/local-defaults-provider \
    ${HOME}/.cache/%{__cargo_target}/release/local-overrides-provider \
    %{buildroot}%{_cross_libexecdir}/early-boot-config/bin

%ifarch x86_64
install -p -m 0755 \
    ${HOME}/.cache/%{__cargo_target}/release/vmware-cd-rom-provider \
    ${HOME}/.cache/%{__cargo_target}/release/vmware-guestinfo-user-data-provider \
    %{buildroot}%{_cross_libexecdir}/early-boot-config/bin
%endif

install -d %{buildroot}%{_cross_datadir}/early-boot-config/data-providers.d

%post -n %{_cross_os}early-boot-config-aws -p <lua>
posix.symlink("../../../libexec/early-boot-config/bin/ec2-identity-doc-provider", "%{_cross_datadir}/early-boot-config/data-providers.d/30-ec2-identity-doc")
posix.symlink("../../../libexec/early-boot-config/bin/ec2-imds-provider", "%{_cross_datadir}/early-boot-config/data-providers.d/40-ec2-imds")

%post -n %{_cross_os}early-boot-config-local -p <lua>
posix.symlink("../../../libexec/early-boot-config/bin/local-user-data-provider", "%{_cross_datadir}/early-boot-config/data-providers.d/20-local-user-data")
posix.symlink("../../../libexec/early-boot-config/bin/local-defaults-provider", "%{_cross_datadir}/early-boot-config/data-providers.d/10-local-defaults")
posix.symlink("../../../libexec/early-boot-config/bin/local-overrides-provider", "%{_cross_datadir}/early-boot-config/data-providers.d/50-local-overrides")

%ifarch x86_64
%post -n %{_cross_os}early-boot-config-vmware -p <lua>
posix.symlink("../../../libexec/early-boot-config/bin/vmware-cd-rom-provider", "%{_cross_datadir}/early-boot-config/data-providers.d/30-vmware-cd-rom")
posix.symlink("../../../libexec/early-boot-config/bin/vmware-guestinfo-user-data-provider", "%{_cross_datadir}/early-boot-config/data-providers.d/40-vmware-guestinfo")
%endif

%files -n %{_cross_os}early-boot-config-common
%{_cross_bindir}/early-boot-config
%{_cross_unitdir}/early-boot-config.service

%files -n %{_cross_os}early-boot-config-data-providers
%dir %{_cross_datadir}/early-boot-config/data-providers.d

%files -n %{_cross_os}early-boot-config-local
%{_cross_libexecdir}/early-boot-config/bin/local-user-data-provider
%{_cross_libexecdir}/early-boot-config/bin/local-defaults-provider
%{_cross_libexecdir}/early-boot-config/bin/local-overrides-provider

%files -n %{_cross_os}early-boot-config-aws
%{_cross_libexecdir}/early-boot-config/bin/ec2-identity-doc-provider
%{_cross_libexecdir}/early-boot-config/bin/ec2-imds-provider

%ifarch x86_64
%files -n %{_cross_os}early-boot-config-vmware
%{_cross_libexecdir}/early-boot-config/bin/vmware-cd-rom-provider
%{_cross_libexecdir}/early-boot-config/bin/vmware-guestinfo-user-data-provider
%endif

%files -n %{_cross_os}early-boot-config-metal
