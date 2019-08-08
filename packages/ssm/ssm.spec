%global goproject github.com/aws
%global gorepo amazon-ssm-agent
%global goimport %{goproject}/%{gorepo}

%global gover 2.3.672.0
%global rpmver %{gover}

%global _dwz_low_mem_die_limit 0

Name: %{_cross_os}%{gorepo}
Version: %{gover}
Release: 1%{?dist}
Summary: AWS SSM Agent
License: ASL 2.0
URL: https://%{goimport}
Source0: https://%{goimport}/archive/%{gover}/%{gorepo}-%{gover}.tar.gz
Source1: ssm-tmpfiles.conf
Source2: amazon-ssm-agent.service
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
install -d %{buildroot}%{_cross_bindir}
install -p -m 0755 bin/amazon-ssm-agent %{buildroot}%{_cross_bindir}
install -p -m 0755 bin/ssm-cli %{buildroot}%{_cross_bindir}
install -p -m 0755 bin/ssm-document-worker %{buildroot}%{_cross_bindir}
install -p -m 0755 bin/ssm-session-logger %{buildroot}%{_cross_bindir}
install -p -m 0755 bin/ssm-session-worker %{buildroot}%{_cross_bindir}

install -d %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}/amazon/ssm
install -p -m 0755 seelog_unix.xml %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}/amazon/ssm/seelog.xml
install -p -m 0755 amazon-ssm-agent.json.template %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}/amazon/ssm/amazon-ssm-agent.json

mkdir -p %{buildroot}/%{_cross_unitdir}
install -p -m 0644 %{S:2} %{buildroot}%{_cross_unitdir}/amazon-ssm-agent.service

install -d %{buildroot}%{_cross_tmpfilesdir}
install -p -m 0644 %{S:1} %{buildroot}%{_cross_tmpfilesdir}/ssm.conf

%files
%{_cross_bindir}/amazon-ssm-agent
%{_cross_bindir}/ssm-cli
%{_cross_bindir}/ssm-document-worker
%{_cross_bindir}/ssm-session-logger
%{_cross_bindir}/ssm-session-worker
%dir %{_cross_factorydir}%{_cross_sysconfdir}/amazon
%dir %{_cross_factorydir}%{_cross_sysconfdir}/amazon/ssm
%{_cross_factorydir}%{_cross_sysconfdir}/amazon/ssm/seelog.xml
%{_cross_factorydir}%{_cross_sysconfdir}/amazon/ssm/amazon-ssm-agent.json
%{_cross_unitdir}/amazon-ssm-agent.service
%{_cross_tmpfilesdir}/ssm.conf

%changelog
