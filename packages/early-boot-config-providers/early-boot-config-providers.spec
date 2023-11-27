%global _cross_first_party 1
%undefine _debugsource_packages

Name: %{_cross_os}early-boot-config-providers
Version: 0.0
Release: 0%{?dist}
Summary: Platform user data providers
License: Apache-2.0 OR MIT
URL: https://github.com/bottlerocket-os/bottlerocket

Requires: %{_cross_os}local-defaults-user-data-client
Requires: %{_cross_os}local-defaults-user-data-provider
Requires: %{_cross_os}local-user-data-client
Requires: %{_cross_os}local-user-data-provider
Requires: %{_cross_os}local-overrides-user-data-client
Requires: %{_cross_os}local-overrides-user-data-provider
Requires: %{_cross_os}ec2-identity-doc-user-data-client
Requires: %{_cross_os}ec2-identity-doc-user-data-provider
Requires: %{_cross_os}ec2-imds-user-data-client
Requires: %{_cross_os}ec2-imds-user-data-provider
Requires: %{_cross_os}vmware-cd-rom-user-data-client
Requires: %{_cross_os}vmware-cd-rom-user-data-provider
Requires: %{_cross_os}vmware-guestinfo-user-data-client
Requires: %{_cross_os}vmware-guestinfo-user-data-provider

%description
%{summary}.

# AWS's user data source ordering is as follows:
# - local defaults file
# - local user data file
# - EC2 instance identity doc
# - EC2 IMDS
# - local overrides file
%package -n %{_cross_os}aws-data-providers
Summary: User data providers for AWS variants
Requires: %{_cross_os}local-defaults-user-data-client
Requires: %{_cross_os}local-defaults-user-data-provider
Requires: %{_cross_os}local-user-data-client
Requires: %{_cross_os}local-user-data-provider
Requires: %{_cross_os}ec2-identity-doc-user-data-client
Requires: %{_cross_os}ec2-identity-doc-user-data-provider
Requires: %{_cross_os}ec2-imds-user-data-client
Requires: %{_cross_os}ec2-imds-user-data-provider
Requires: %{_cross_os}local-overrides-user-data-client
Requires: %{_cross_os}local-overrides-user-data-provider
%description -n %{_cross_os}aws-data-providers
%{summary}.

# VMware's user data source ordering is as follows:
# - local defaults file
# - local user data file
# - CD-ROM OVF
# - guestinfo interface
# - local overrides file
%package -n %{_cross_os}vmware-data-providers
Summary: User data providers for VMware variants
Requires: %{_cross_os}local-defaults-user-data-client
Requires: %{_cross_os}local-defaults-user-data-provider
Requires: %{_cross_os}local-user-data-client
Requires: %{_cross_os}local-user-data-provider
Requires: %{_cross_os}vmware-cd-rom-user-data-client
Requires: %{_cross_os}vmware-cd-rom-user-data-provider
Requires: %{_cross_os}vmware-guestinfo-user-data-client
Requires: %{_cross_os}vmware-guestinfo-user-data-provider
Requires: %{_cross_os}local-overrides-user-data-client
Requires: %{_cross_os}local-overrides-user-data-provider
%description -n %{_cross_os}vmware-data-providers
%{summary}.

# Metal user data source ordering is as follows:
# - local defaults file
# - local user data file
# - local overrides file
%package -n %{_cross_os}metal-data-providers
Summary: User data providers for metal variants
Requires: %{_cross_os}local-defaults-user-data-client
Requires: %{_cross_os}local-defaults-user-data-provider
Requires: %{_cross_os}local-user-data-client
Requires: %{_cross_os}local-user-data-provider
Requires: %{_cross_os}local-overrides-user-data-client
Requires: %{_cross_os}local-overrides-user-data-provider
%description -n %{_cross_os}metal-data-providers
%{summary}.

%prep
%setup -T -c
%build
%install
%files -n %{_cross_os}aws-data-providers
%files -n %{_cross_os}vmware-data-providers
%files -n %{_cross_os}metal-data-providers
