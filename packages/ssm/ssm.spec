%global goproject github.com/aws
%global gorepo amazon-ssm-agent
%global goimport %{goproject}/%{gorepo}

%global gover 2.3.672.0
%global rpmver %{gover}

%global ssmdir  %{buildroot}%{_cross_factorydir}/%{_cross_sharedstatedir}/amazon/ssm
%global ssmdir_installed  %{_cross_factorydir}/%{_cross_sharedstatedir}/amazon/ssm

%global _dwz_low_mem_die_limit 0
%global debug_package %{nil}

Name: %{_cross_os}%{gorepo}
Version: %{gover}
Release: 1%{?dist}
Summary: AWS SSM Agent
License: ASL 2.0
URL: https://%{goimport}
Source0: https://%{goimport}/archive/%{gover}/%{gorepo}-%{gover}.tar.gz
Source1: ssm-tmpfiles.conf
Source2: amazon-ssm-agent.service
Source3: Dockerfile
Patch1: 0001-Use-absolute-path-to-launch-shell.patch
Patch2: 0002-shell-Allow-root-user.patch
BuildRequires: gcc-%{_cross_target}
BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}golang
Requires: %{_cross_os}glibc

%description
%{summary}.

%prep
%autosetup -n amazon-ssm-agent-%{version} -p1
%cross_go_setup %{gorepo}-%{gover} %{goproject} %{goimport}
sed -i -e 's#const[ \s]*Version.*#const Version = "%{version}"#g' agent/version/version.go

%build
%cross_go_configure %{goimport}
export GOPATH="${GOPATH}:${PWD}/GOPATH/src/%{goimport}/vendor"

go build -ldflags "-linkmode=external" -o bin/amazon-ssm-agent -v %{goimport}/agent
go build -ldflags "-linkmode=external" -o bin/ssm-document-worker -v %{goimport}/agent/framework/processor/executer/outofproc/worker
go build -ldflags "-linkmode=external" -o bin/ssm-session-worker -v %{goimport}/agent/framework/processor/executer/outofproc/sessionworker
go build -ldflags "-linkmode=external" -o bin/ssm-session-logger -v %{goimport}/agent/session/logging
go build -ldflags "-linkmode=external" -o bin/ssm-cli -v %{goimport}/agent/cli-main

%install
# Create a pretend tree under /var/lib/amazon/ssm and remove executable
# permissions; these will only be used from within the SSM container.
install -d %{ssmdir}%{_bindir}
install -p -m 0644 bin/amazon-ssm-agent %{ssmdir}%{_bindir}
install -p -m 0644 bin/ssm-cli %{ssmdir}%{_bindir}
install -p -m 0644 bin/ssm-document-worker %{ssmdir}%{_bindir}
install -p -m 0644 bin/ssm-session-logger %{ssmdir}%{_bindir}
install -p -m 0644 bin/ssm-session-worker %{ssmdir}%{_bindir}

install -d %{ssmdir}%{_sysconfdir}/amazon/ssm
install -p -m 0644 seelog_unix.xml %{ssmdir}%{_sysconfdir}/amazon/ssm/seelog.xml
install -p -m 0644 amazon-ssm-agent.json.template %{ssmdir}%{_sysconfdir}/amazon/ssm/amazon-ssm-agent.json

# Install the Dockerfile at the top of this tree
install -p -m 0644 %{S:3} %{ssmdir}/Dockerfile

mkdir -p %{buildroot}/%{_cross_unitdir}
install -p -m 0644 %{S:2} %{buildroot}%{_cross_unitdir}/amazon-ssm-agent.service

install -d %{buildroot}%{_cross_tmpfilesdir}
install -p -m 0644 %{S:1} %{buildroot}%{_cross_tmpfilesdir}/ssm.conf

%files
%dir %{ssmdir_installed}%{_sysconfdir}/amazon/ssm
%dir %{ssmdir_installed}%{_bindir}
%{ssmdir_installed}%{_bindir}/amazon-ssm-agent
%{ssmdir_installed}%{_bindir}/ssm-cli
%{ssmdir_installed}%{_bindir}/ssm-document-worker
%{ssmdir_installed}%{_bindir}/ssm-session-logger
%{ssmdir_installed}%{_bindir}/ssm-session-worker
%{ssmdir_installed}%{_sysconfdir}/amazon/ssm/seelog.xml
%{ssmdir_installed}%{_sysconfdir}/amazon/ssm/amazon-ssm-agent.json
%dir %{_cross_factorydir}%{_cross_sharedstatedir}/amazon/ssm
%{_cross_factorydir}%{_cross_sharedstatedir}/amazon/ssm/Dockerfile
%{_cross_unitdir}/amazon-ssm-agent.service
%{_cross_tmpfilesdir}/ssm.conf

%changelog
