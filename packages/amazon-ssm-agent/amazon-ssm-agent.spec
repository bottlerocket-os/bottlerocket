# Don't generate debug packages because we are compiling without CGO,
# and the `gc` compiler doesn't append the  the ".note.gnu.build-id" section
# https://fedoraproject.org/wiki/PackagingDrafts/Go#Build_ID
%global debug_package %{nil}

%global goproject github.com/aws
%global gorepo amazon-ssm-agent
%global goimport %{goproject}/%{gorepo}

Name: %{_cross_os}amazon-ssm-agent
Version: 3.2.1478.0
Release: 1%{?dist}
Summary: An agent to enable remote management of EC2 instances
License: Apache-2.0
URL: https://github.com/aws/amazon-ssm-agent
Source0: %{gorepo}-%{version}.tar.gz
BuildRequires: %{_cross_os}glibc-devel

%description
%{summary}.

%prep
%setup -n %{gorepo}-%{version}

%build
%set_cross_go_flags

# Set CGO_ENABLED=0 to statically link binaries that will be bind-mounted by the ECS agent
CGO_ENABLED=0 go build ${GOFLAGS} -installsuffix cgo -a -ldflags "-s" -o amazon-ssm-agent \
  ./core/agent.go ./core/agent_unix.go ./core/agent_parser.go
CGO_ENABLED=0 go build ${GOFLAGS} -installsuffix cgo -a -ldflags "-s" -o ssm-agent-worker \
  ./agent/agent.go ./agent/agent_unix.go ./agent/agent_parser.go
CGO_ENABLED=0 go build ${GOFLAGS} -installsuffix cgo -a -ldflags "-s" -o ssm-session-worker \
  ./agent/framework/processor/executer/outofproc/sessionworker/main.go

%install
# Install the SSM agent under 'libexecdir', since it is meant to be used by other programs
install -d %{buildroot}%{_cross_libexecdir}/amazon-ssm-agent/bin/%{version}
for b in amazon-ssm-agent ssm-agent-worker ssm-session-worker; do
  install -D -p -m 0755 ${b} %{buildroot}%{_cross_libexecdir}/amazon-ssm-agent/bin/%{version}
done

%cross_scan_attribution go-vendor vendor

%files
%license LICENSE
%{_cross_attribution_file}
%{_cross_attribution_vendor_dir}
%dir %{_cross_libexecdir}/amazon-ssm-agent
%{_cross_libexecdir}/amazon-ssm-agent
