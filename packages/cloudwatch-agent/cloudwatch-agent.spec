%global goproject github.com/aws
%global gorepo amazon-cloudwatch-agent
%global goimport %{goproject}/%{gorepo}

%global gover 1.247359.1
%global rpmver %{gover}

%global _dwz_low_mem_die_limit 0
%global debug_package %{nil}

Name: %{_cross_os}cloudwatch-agent
Version: %{rpmver}
Release: 1%{?dist}
Summary: Amazon Cloudwatch Agent daemon
License: MIT License. Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
URL: https://github.com/aws/amazon-cloudwatch-agent
Source0: https://%{goimport}/archive/v%{gover}/%{gorepo}-%{gover}.tar.gz
Source1: bundled-amazon-cloudwatch-agent-%{gover}.tar.gz
Source2: cloudwatch-agent.service
Source3: cloudwatch-agent.conf
Source4: cloudwatch-agent-tmpfiles.conf
Source5: CWAGENT_VERSION
Source6: config.json
Source7: amazon-cloudwatch-agent-schema.json
Source9: common-config.toml
Source10: env-config.json

Source1000: clarify.toml

BuildRequires: %{_cross_os}glibc-devel

%description
%{summary}.

%prep
%setup -n %{gorepo}-%{gover} -q
%setup -T -D -n %{gorepo}-%{gover} -b 1 -q

%build
%set_cross_go_flags
go build -buildmode=pie -ldflags="${GOLDFLAGS}" -o=config-downloader ./cmd/config-downloader/
go build -buildmode=pie -ldflags="${GOLDFLAGS}" -o=config-translator ./cmd/config-translator
go build -buildmode=pie -ldflags="${GOLDFLAGS}" -o=amazon-cloudwatch-agent ./cmd/amazon-cloudwatch-agent
go build -buildmode=pie -ldflags="${GOLDFLAGS}" -o=start-amazon-cloudwatch-agent ./cmd/start-amazon-cloudwatch-agent

%install
install -d %{buildroot}%{_cross_unitdir}
install -p -m 0644 %{S:2} %{buildroot}%{_cross_unitdir}/cloudwatch-agent.service

install -d %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}
install -p -m 0644 %{S:3} %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}/cloudwatch-agent.conf

install -d %{buildroot}%{_cross_libexecdir}/aws/amazon-cloudwatch-agent/bin
install -p -m 0755 packaging/dependencies/amazon-cloudwatch-agent-ctl  %{buildroot}%{_cross_libexecdir}/aws/amazon-cloudwatch-agent/bin/amazon-cloudwatch-agent-ctl
install -p -m 0755 config-downloader  %{buildroot}%{_cross_libexecdir}/aws/amazon-cloudwatch-agent/bin/config-downloader
install -p -m 0755 config-translator %{buildroot}%{_cross_libexecdir}/aws/amazon-cloudwatch-agent/bin/config-translator
install -p -m 0755 amazon-cloudwatch-agent %{buildroot}%{_cross_libexecdir}/aws/amazon-cloudwatch-agent/bin/amazon-cloudwatch-agent
install -p -m 0755 start-amazon-cloudwatch-agent %{buildroot}%{_cross_libexecdir}/aws/amazon-cloudwatch-agent/bin/start-amazon-cloudwatch-agent
install -p -m 0644 %{S:5} %{buildroot}%{_cross_libexecdir}/aws/amazon-cloudwatch-agent/bin/CWAGENT_VERSION
install -p -m 0644 %{S:6} %{buildroot}%{_cross_libexecdir}/aws/amazon-cloudwatch-agent/bin/config.json

install -d %{buildroot}%{_cross_factorydir}/opt/aws/amazon-cloudwatch-agent/doc
install -p -m 0644 %{S:7} %{buildroot}%{_cross_factorydir}/opt/aws/amazon-cloudwatch-agent/doc/amazon-cloudwatch-agent-schema.json

install -d %{buildroot}%{_cross_factorydir}/opt/aws/amazon-cloudwatch-agent/etc
install -p -m 0644 %{S:9} %{buildroot}%{_cross_factorydir}/opt/aws/amazon-cloudwatch-agent/etc/common-config.toml
install -p -m 0644 %{S:10} %{buildroot}%{_cross_factorydir}/opt/aws/amazon-cloudwatch-agent/etc/env-config.json

install -d %{buildroot}%{_cross_factorydir}/opt/aws/amazon-cloudwatch-agent/etc/amazon-cloudwatch-agent.d
install -p -m 0644 %{S:6} %{buildroot}%{_cross_factorydir}/opt/aws/amazon-cloudwatch-agent/etc/amazon-cloudwatch-agent.d/file_config.json

install -d %{buildroot}%{_cross_tmpfilesdir}
install -p -m 0644 %{S:4} %{buildroot}%{_cross_tmpfilesdir}/cloudwatch-agent-tmpfiles.conf

%cross_scan_attribution --clarify %{S:1000} go-vendor vendor

%files
%license LICENSE
%{_cross_attribution_file}
%{_cross_attribution_vendor_dir}
%{_cross_unitdir}/cloudwatch-agent.service
%{_cross_factorydir}%{_cross_sysconfdir}/cloudwatch-agent.conf
%{_cross_libexecdir}/aws/amazon-cloudwatch-agent/bin/amazon-cloudwatch-agent-ctl
%{_cross_libexecdir}/aws/amazon-cloudwatch-agent/bin/config-downloader
%{_cross_libexecdir}/aws/amazon-cloudwatch-agent/bin/config-translator
%{_cross_libexecdir}/aws/amazon-cloudwatch-agent/bin/amazon-cloudwatch-agent
%{_cross_libexecdir}/aws/amazon-cloudwatch-agent/bin/start-amazon-cloudwatch-agent
%{_cross_libexecdir}/aws/amazon-cloudwatch-agent/bin/CWAGENT_VERSION
%{_cross_libexecdir}/aws/amazon-cloudwatch-agent/bin/config.json
%{_cross_factorydir}/opt/aws/amazon-cloudwatch-agent/doc/amazon-cloudwatch-agent-schema.json
%{_cross_factorydir}/opt/aws/amazon-cloudwatch-agent/etc/common-config.toml
%{_cross_factorydir}/opt/aws/amazon-cloudwatch-agent/etc/env-config.json
%{_cross_factorydir}/opt/aws/amazon-cloudwatch-agent/etc/amazon-cloudwatch-agent.d/file_config.json
%{_cross_tmpfilesdir}/cloudwatch-agent-tmpfiles.conf


