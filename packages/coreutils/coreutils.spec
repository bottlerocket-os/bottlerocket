Name: %{_cross_os}coreutils
Version: 9.3
Release: 1%{?dist}
Summary: A set of basic GNU tools
License: GPL-3.0-or-later
URL: https://www.gnu.org/software/coreutils/
Source0: https://ftp.gnu.org/gnu/coreutils/coreutils-%{version}.tar.xz
BuildRequires: %{_cross_os}glibc-devel
BuildRequires: %{_cross_os}libacl-devel
BuildRequires: %{_cross_os}libattr-devel
BuildRequires: %{_cross_os}libcap-devel
BuildRequires: %{_cross_os}libselinux-devel
BuildRequires: %{_cross_os}libxcrypt-devel
Requires: %{_cross_os}libacl
Requires: %{_cross_os}libattr
Requires: %{_cross_os}libcap
Requires: %{_cross_os}libselinux
Requires: %{_cross_os}libxcrypt

%description
%{summary}.

%prep
%autosetup -n coreutils-%{version} -p1

%build
%cross_configure \
  --disable-acl \
  --disable-rpath \
  --enable-single-binary=symlinks \
  --enable-no-install-program=kill,stdbuf,uptime \
  --with-selinux \
  --without-gmp \
  --without-openssl \

%make_build

%install
%make_install

%files
%license COPYING
%{_cross_attribution_file}
%{_cross_bindir}/[
%{_cross_bindir}/b2sum
%{_cross_bindir}/base32
%{_cross_bindir}/base64
%{_cross_bindir}/basename
%{_cross_bindir}/basenc
%{_cross_bindir}/cat
%{_cross_bindir}/chcon
%{_cross_bindir}/chgrp
%{_cross_bindir}/chmod
%{_cross_bindir}/chown
%{_cross_bindir}/chroot
%{_cross_bindir}/cksum
%{_cross_bindir}/comm
%{_cross_bindir}/coreutils
%{_cross_bindir}/cp
%{_cross_bindir}/csplit
%{_cross_bindir}/cut
%{_cross_bindir}/date
%{_cross_bindir}/dd
%{_cross_bindir}/df
%{_cross_bindir}/dir
%{_cross_bindir}/dircolors
%{_cross_bindir}/dirname
%{_cross_bindir}/du
%{_cross_bindir}/echo
%{_cross_bindir}/env
%{_cross_bindir}/expand
%{_cross_bindir}/expr
%{_cross_bindir}/factor
%{_cross_bindir}/false
%{_cross_bindir}/fmt
%{_cross_bindir}/fold
%{_cross_bindir}/groups
%{_cross_bindir}/head
%{_cross_bindir}/hostid
%{_cross_bindir}/id
%{_cross_bindir}/install
%{_cross_bindir}/join
%{_cross_bindir}/link
%{_cross_bindir}/ln
%{_cross_bindir}/logname
%{_cross_bindir}/ls
%{_cross_bindir}/md5sum
%{_cross_bindir}/mkdir
%{_cross_bindir}/mkfifo
%{_cross_bindir}/mknod
%{_cross_bindir}/mktemp
%{_cross_bindir}/mv
%{_cross_bindir}/nice
%{_cross_bindir}/nl
%{_cross_bindir}/nohup
%{_cross_bindir}/nproc
%{_cross_bindir}/numfmt
%{_cross_bindir}/od
%{_cross_bindir}/paste
%{_cross_bindir}/pathchk
%{_cross_bindir}/pinky
%{_cross_bindir}/pr
%{_cross_bindir}/printenv
%{_cross_bindir}/printf
%{_cross_bindir}/ptx
%{_cross_bindir}/pwd
%{_cross_bindir}/readlink
%{_cross_bindir}/realpath
%{_cross_bindir}/rm
%{_cross_bindir}/rmdir
%{_cross_bindir}/runcon
%{_cross_bindir}/seq
%{_cross_bindir}/sha1sum
%{_cross_bindir}/sha224sum
%{_cross_bindir}/sha256sum
%{_cross_bindir}/sha384sum
%{_cross_bindir}/sha512sum
%{_cross_bindir}/shred
%{_cross_bindir}/shuf
%{_cross_bindir}/sleep
%{_cross_bindir}/sort
%{_cross_bindir}/split
%{_cross_bindir}/stat
%{_cross_bindir}/stty
%{_cross_bindir}/sum
%{_cross_bindir}/sync
%{_cross_bindir}/tac
%{_cross_bindir}/tail
%{_cross_bindir}/tee
%{_cross_bindir}/test
%{_cross_bindir}/timeout
%{_cross_bindir}/touch
%{_cross_bindir}/tr
%{_cross_bindir}/true
%{_cross_bindir}/truncate
%{_cross_bindir}/tsort
%{_cross_bindir}/tty
%{_cross_bindir}/uname
%{_cross_bindir}/unexpand
%{_cross_bindir}/uniq
%{_cross_bindir}/unlink
%{_cross_bindir}/users
%{_cross_bindir}/vdir
%{_cross_bindir}/wc
%{_cross_bindir}/who
%{_cross_bindir}/whoami
%{_cross_bindir}/yes
%exclude %{_cross_infodir}
%exclude %{_cross_localedir}
%exclude %{_cross_mandir}

%changelog
