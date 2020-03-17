%global _cross_first_party 1
%global policytype fortified

Name: %{_cross_os}selinux-policy
Version: 0.0
Release: 0%{?dist}
Summary: SELinux policy
License: Apache-2.0 OR MIT

# CIL policy files
Source0: base.cil
Source1: sid.cil
Source2: class.cil
Source3: subject.cil
Source4: object.cil
Source5: fs.cil
Source6: processes.cil
Source7: files.cil
Source8: sockets.cil
Source9: networks.cil
Source10: ipcs.cil
Source11: systems.cil
Source12: rules.cil

# Helpers for generating CIL
Source50: catgen.sh

# Misc config files
Source100: selinux.config
Source101: lxc_contexts

BuildArch: noarch
BuildRequires: secilc

%description
%{summary}.

%prep
%setup -T -c

%build
%{_sourcedir}/catgen.sh > category.cil
secilc --policyvers=31 \
  %{S:0} %{S:1} %{S:2} %{S:3} %{S:4} %{S:5} \
  %{S:6} %{S:7} %{S:8} %{S:9} %{S:10} %{S:11} \
  %{S:12} *.cil

%install
install -d %{buildroot}%{_cross_libdir}/selinux/%{policytype}/{contexts/files,policy}
install -p -m 0644 %{S:100} %{buildroot}%{_cross_libdir}/selinux/config
install -p -m 0644 %{S:101} %{buildroot}%{_cross_libdir}/selinux/%{policytype}/contexts
install -p -m 0644 file_contexts %{buildroot}%{_cross_libdir}/selinux/%{policytype}/contexts/files
install -p -m 0644 policy.31 %{buildroot}%{_cross_libdir}/selinux/%{policytype}/policy

%files
%{_cross_libdir}/selinux/config
%{_cross_libdir}/selinux/%{policytype}/contexts/files/file_contexts
%{_cross_libdir}/selinux/%{policytype}/contexts/lxc_contexts
%{_cross_libdir}/selinux/%{policytype}/policy/policy.31

%changelog
