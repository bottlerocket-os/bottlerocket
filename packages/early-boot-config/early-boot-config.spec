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

%package -n %{_cross_os}early-boot-config-local
Summary: local-provider

%description -n %{_cross_os}early-boot-config-local
%{summary}.

%package -n %{_cross_os}early-boot-config-aws
Summary: early-boot-config package for AWS
Requires: %{name}
Requires: %{_cross_os}early-boot-config-local

%description -n %{_cross_os}early-boot-config-aws
%{summary}.

%ifarch x86_64
%package -n %{_cross_os}early-boot-config-vmware
Summary: early-boot-config package for vmware
Requires: %{name}
Requires: %{_cross_os}early-boot-config-local

%description -n %{_cross_os}early-boot-config-vmware
%{summary}.
%endif

%package -n %{_cross_os}early-boot-config-metal
Summary: early-boot-config package for metal
Requires: %{name}
Requires: %{_cross_os}early-boot-config-local

%description -n %{_cross_os}early-boot-config-metal
%{summary}.

%prep
%setup -T -c
%cargo_prep

%build
%cargo_build --manifest-path %{_builddir}/sources/Cargo.toml \
    -p early-boot-config

# build aws user data providers
%cargo_build --manifest-path %{_builddir}/sources/Cargo.toml \
    -p ec2-identity-doc-user-data-provider \
    -p ec2-imds-user-data-provider

# build local file user data providers
%cargo_build --manifest-path %{_builddir}/sources/Cargo.toml \
    -p local-defaults-user-data-provider \
    -p local-file-user-data-provider \
    -p local-overrides-user-data-provider

%ifarch x86_64
# build vmware user data providers
%cargo_build --manifest-path %{_builddir}/sources/Cargo.toml \
    -p vmware-cd-rom-user-data-provider \
    -p vmware-guestinfo-user-data-provider
%endif

%global cargo_outdir %{getenv:HOME}/.cache/%{__cargo_target}/release
%global early_boot_config_bindir %{_cross_libexecdir}/early-boot-config/bin
%global early_boot_config_provider_dir %{_cross_libexecdir}/early-boot-config/data-providers.d

%install
install -d %{buildroot}%{_cross_bindir}
install -p -m 0755 %{cargo_outdir}/early-boot-config %{buildroot}%{_cross_bindir}

install -d %{buildroot}%{_cross_unitdir}
install -p -m 0644 %{S:100} %{buildroot}%{_cross_unitdir}

install -d %{buildroot}%{early_boot_config_bindir}
install -p -m 0755 \
    %{cargo_outdir}/ec2-identity-doc-user-data-provider \
    %{cargo_outdir}/ec2-imds-user-data-provider \
    %{cargo_outdir}/local-defaults-user-data-provider \
    %{cargo_outdir}/local-file-user-data-provider \
    %{cargo_outdir}/local-overrides-user-data-provider \
    %{buildroot}%{early_boot_config_bindir}

%ifarch x86_64
install -p -m 0755 \
    %{cargo_outdir}/vmware-cd-rom-user-data-provider \
    %{cargo_outdir}/vmware-guestinfo-user-data-provider \
    %{buildroot}%{early_boot_config_bindir}
%endif

install -d %{buildroot}%{early_boot_config_provider_dir}

ln -rs \
  %{buildroot}%{early_boot_config_bindir}/ec2-identity-doc-user-data-provider \
  %{buildroot}%{early_boot_config_provider_dir}/30-ec2-identity-doc

ln -rs \
  %{buildroot}%{early_boot_config_bindir}/ec2-imds-user-data-provider \
  %{buildroot}%{early_boot_config_provider_dir}/40-ec2-imds

ln -rs \
  %{buildroot}%{early_boot_config_bindir}/local-defaults-user-data-provider \
  %{buildroot}%{early_boot_config_provider_dir}/10-local-defaults

ln -rs \
  %{buildroot}%{early_boot_config_bindir}/local-file-user-data-provider \
  %{buildroot}%{early_boot_config_provider_dir}/20-local-user-data

ln -rs \
  %{buildroot}%{early_boot_config_bindir}/local-overrides-user-data-provider \
  %{buildroot}%{early_boot_config_provider_dir}/99-local-overrides

%ifarch x86_64
ln -rs \
  %{buildroot}%{early_boot_config_bindir}/vmware-cd-rom-user-data-provider \
  %{buildroot}%{early_boot_config_provider_dir}/30-vmware-cd-rom

ln -rs \
  %{buildroot}%{early_boot_config_bindir}/vmware-guestinfo-user-data-provider \
  %{buildroot}%{early_boot_config_provider_dir}/40-vmware-guestinfo
%endif

%files
%{_cross_bindir}/early-boot-config
%{_cross_unitdir}/early-boot-config.service
%dir %{early_boot_config_provider_dir}

%files -n %{_cross_os}early-boot-config-local
%{early_boot_config_bindir}/local-defaults-user-data-provider
%{early_boot_config_bindir}/local-file-user-data-provider
%{early_boot_config_bindir}/local-overrides-user-data-provider
%{early_boot_config_provider_dir}/10-local-defaults
%{early_boot_config_provider_dir}/20-local-user-data
%{early_boot_config_provider_dir}/99-local-overrides

%files -n %{_cross_os}early-boot-config-aws
%{early_boot_config_bindir}/ec2-identity-doc-user-data-provider
%{early_boot_config_bindir}/ec2-imds-user-data-provider
%{early_boot_config_provider_dir}/30-ec2-identity-doc
%{early_boot_config_provider_dir}/40-ec2-imds

%ifarch x86_64
%files -n %{_cross_os}early-boot-config-vmware
%{early_boot_config_bindir}/vmware-cd-rom-user-data-provider
%{early_boot_config_bindir}/vmware-guestinfo-user-data-provider
%{early_boot_config_provider_dir}/30-vmware-cd-rom
%{early_boot_config_provider_dir}/40-vmware-guestinfo
%endif

# There are no metal-specific providers, just dependencies like the local file providers.
%files -n %{_cross_os}early-boot-config-metal
