%global goproject github.com/bdwyertech
%global gorepo journald-cloudwatch-logs
%global goimport %{goproject}/%{gorepo}

%global gover 0.2.11
%global rpmver %{gover}

%global _dwz_low_mem_die_limit 0

Name: %{_cross_os}journald-cloudwatch-logs
Version: %{rpmver}
Release: 1%{?dist}
Summary: Tool to send journald logs to cloud watch logs
License: 2015-Say-Media-Inc
URL: https://github.com/bdwyertech/journald-cloudwatch-logs
Source0: https://github.com/bdwyertech/journald-cloudwatch-logs/archive/v%{version}/journald-cloudwatch-logs-%{version}.tar.gz
Source1: journald-cloudwatch-logs.service
Source2: journald-cloudwatch-logs.conf
Source3: journald-cloudwatch-logs-tmpfiles.conf

Source1000: clarify.toml

BuildRequires: %{_cross_os}systemd-devel
BuildRequires: %{_cross_os}glibc-devel

%description
%{summary}.

%prep
%setup -n %{gorepo}-%{gover} -q

%build
%set_cross_go_flags
go build -buildmode=pie -ldflags="${GOLDFLAGS}" -o=journald-cloudwatch-logs

%install
install -d %{buildroot}%{_cross_bindir}
install -p -m 0755 journald-cloudwatch-logs %{buildroot}%{_cross_bindir}/journald-cloudwatch-logs

install -d %{buildroot}%{_cross_unitdir}
install -p -m 0644 %{S:1} %{buildroot}%{_cross_unitdir}/journald-cloudwatch-logs.service

install -d %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}
install -p -m 0644 %{S:2} %{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}/journald-cloudwatch-logs.conf

install -d %{buildroot}%{_cross_tmpfilesdir}
install -p -m 0644 %{S:3} %{buildroot}%{_cross_tmpfilesdir}/journald-cloudwatch-logs-tmpfiles.conf

%cross_scan_attribution --clarify %{S:1000} go-vendor vendor

%files
%{_cross_attribution_file}
%{_cross_attribution_vendor_dir}
%{_cross_unitdir}/journald-cloudwatch-logs.service
%{_cross_bindir}/journald-cloudwatch-logs
%{_cross_factorydir}%{_cross_sysconfdir}/journald-cloudwatch-logs.conf
%{_cross_tmpfilesdir}/journald-cloudwatch-logs-tmpfiles.conf


