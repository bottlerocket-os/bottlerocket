%global debug_package %{nil}

%global _cross_first_party 1
%global policytype fortified

Name: %{_cross_os}selinux-policy
Version: 0.0
Release: 0%{?dist}
Summary: SELinux policy
License: Apache-2.0 OR MIT
URL: https://github.com/bottlerocket-os/bottlerocket

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
Source13: mcs.cil

# Helpers for generating CIL
Source50: catgen.sh

# Misc config files
Source100: selinux.config
Source101: lxc_contexts
Source102: selinux-policy-files.service
Source103: selinux-policy-tmpfiles.conf

BuildRequires: secilc

%description
%{summary}.

%prep
%setup -T -c
cp -p \
  %{S:0} %{S:1} %{S:2} %{S:3} %{S:4} %{S:5} \
  %{S:6} %{S:7} %{S:8} %{S:9} %{S:10} %{S:11} \
  %{S:12} %{S:13} .

%build
%{_sourcedir}/catgen.sh > category.cil
secilc --policyvers=31 *.cil

%install
poldir="%{buildroot}%{_cross_factorydir}%{_cross_sysconfdir}/selinux"
install -d "${poldir}/%{policytype}/"{contexts/files,policy}
install -p -m 0644 %{S:100} "${poldir}/config"
install -p -m 0644 %{S:101} "${poldir}/%{policytype}/contexts"
install -p -m 0644 file_contexts "${poldir}/%{policytype}/contexts/files"
install -p -m 0644 policy.31 "${poldir}/%{policytype}/policy"

moddir="%{buildroot}%{_cross_factorydir}%{_cross_sharedstatedir}/selinux/%{policytype}/active/modules/100"
install -d "${moddir}"
for m in *.cil ; do
  mod="${m%.*}"
  install -d "${moddir}/${mod}"
  bzip2 -c "${m}" > "${moddir}/${mod}/cil"
  echo -n "cil" > "${moddir}/${mod}/lang_ext"
done

install -d %{buildroot}%{_cross_sysconfdir}
ln -s ..%{_cross_factorydir}%{_cross_sysconfdir}/selinux %{buildroot}%{_cross_sysconfdir}/selinux

install -d %{buildroot}%{_cross_unitdir}
install -p -m 0644 %{S:102} %{buildroot}%{_cross_unitdir}/selinux-policy-files.service

install -d %{buildroot}%{_cross_tmpfilesdir}
install -p -m 0644 %{S:103} %{buildroot}%{_cross_tmpfilesdir}/selinux-policy.conf

%files
%{_cross_factorydir}%{_cross_sysconfdir}/selinux/config
%{_cross_factorydir}%{_cross_sysconfdir}/selinux/%{policytype}/contexts/files/file_contexts
%{_cross_factorydir}%{_cross_sysconfdir}/selinux/%{policytype}/contexts/lxc_contexts
%{_cross_factorydir}%{_cross_sysconfdir}/selinux/%{policytype}/policy/policy.31
%{_cross_factorydir}%{_cross_sharedstatedir}/selinux/%{policytype}
%{_cross_sysconfdir}/selinux
%{_cross_tmpfilesdir}/selinux-policy.conf
%{_cross_unitdir}/selinux-policy-files.service

%changelog
