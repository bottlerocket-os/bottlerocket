Name: %{_cross_os}aws-eks-ami
Version: 1.0
Release: 1%{?dist}
Summary: AWS EKS AMI
License: MIT

BuildArch: noarch
Requires: %{_cross_os}aws-iam-authenticator
Requires: %{_cross_os}cni
Requires: %{_cross_os}cni-plugins
Requires: %{_cross_os}docker-cli
Requires: %{_cross_os}docker-engine
Requires: %{_cross_os}docker-init
Requires: %{_cross_os}docker-proxy
Requires: %{_cross_os}kubernetes
Requires: %{_cross_os}release

%description
%{summary}.

%prep

%build

%install

%files

%changelog
