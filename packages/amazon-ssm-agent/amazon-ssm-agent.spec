%global goproject github.com/aws
%global gorepo amazon-ssm-agent
%global goimport %{goproject}/%{gorepo}

Name: %{_cross_os}amazon-ssm-agent
Version: 3.3.418.0
Release: 1%{?dist}
Summary: An agent to enable remote management of EC2 instances
License: Apache-2.0
URL: https://github.com/aws/amazon-ssm-agent
Source0: %{gorepo}-%{version}.tar.gz
Source1000: clarify.toml

BuildRequires: %{_cross_os}glibc-devel
Requires: %{name}(binaries)

%description
%{summary}.

%package bin
Summary: Remote management agent binaries
Provides: %{name}(binaries)
Requires: (%{_cross_os}image-feature(no-fips) and %{name})
Conflicts: (%{_cross_os}image-feature(fips) or %{name}-fips-bin)

%description bin
%{summary}.

%package fips-bin
Summary: Remote management agent binaries, FIPS edition
Provides: %{name}(binaries)
Requires: (%{_cross_os}image-feature(fips) and %{name})
Conflicts: (%{_cross_os}image-feature(no-fips) or %{name}-bin)

%description fips-bin
%{summary}.

%prep
%setup -n %{gorepo}-%{version}

%build
%set_cross_go_flags_static

go build -ldflags "${GOLDFLAGS}" -o amazon-ssm-agent \
  ./core/agent.go ./core/agent_unix.go ./core/agent_parser.go

gofips build -ldflags "${GOLDFLAGS}" -o fips/amazon-ssm-agent \
  ./core/agent.go ./core/agent_unix.go ./core/agent_parser.go

go build -ldflags "${GOLDFLAGS}" -o ssm-agent-worker \
  ./agent/agent.go ./agent/agent_unix.go ./agent/agent_parser.go

gofips build -ldflags "${GOLDFLAGS}" -o fips/ssm-agent-worker \
  ./agent/agent.go ./agent/agent_unix.go ./agent/agent_parser.go

go build -ldflags "${GOLDFLAGS}" -o ssm-session-worker \
  ./agent/framework/processor/executer/outofproc/sessionworker/main.go

gofips build -ldflags "${GOLDFLAGS}" -o fips/ssm-session-worker \
  ./agent/framework/processor/executer/outofproc/sessionworker/main.go

%install
# Install the SSM agent under 'libexecdir', since it is meant to be used by other programs
install -d %{buildroot}{%{_cross_libexecdir},%{_cross_fips_libexecdir}}/amazon-ssm-agent/bin/%{version}
for b in amazon-ssm-agent ssm-agent-worker ssm-session-worker; do
  install -p -m 0755 ${b} %{buildroot}%{_cross_libexecdir}/amazon-ssm-agent/bin/%{version}
  install -p -m 0755 fips/${b} %{buildroot}%{_cross_fips_libexecdir}/amazon-ssm-agent/bin/%{version}
done

%cross_scan_attribution --clarify %{S:1000} go-vendor vendor

%files
%license LICENSE
%{_cross_attribution_file}
%{_cross_attribution_vendor_dir}

%files bin
%dir %{_cross_libexecdir}/amazon-ssm-agent
%{_cross_libexecdir}/amazon-ssm-agent/bin/%{version}/amazon-ssm-agent
%{_cross_libexecdir}/amazon-ssm-agent/bin/%{version}/ssm-agent-worker
%{_cross_libexecdir}/amazon-ssm-agent/bin/%{version}/ssm-session-worker

%files fips-bin
%dir %{_cross_fips_libexecdir}/amazon-ssm-agent
%{_cross_fips_libexecdir}/amazon-ssm-agent/bin/%{version}/amazon-ssm-agent
%{_cross_fips_libexecdir}/amazon-ssm-agent/bin/%{version}/ssm-agent-worker
%{_cross_fips_libexecdir}/amazon-ssm-agent/bin/%{version}/ssm-session-worker
