Name: %{_cross_os}socat
Version: 1.7.4.0
Release: 1%{?dist}
Summary: Transfer data between two channels
License: GPL-2.0-only
URL: http://www.dest-unreach.org/socat/
Source0: http://www.dest-unreach.org/socat/download/socat-%{version}.tar.bz2
BuildRequires: %{_cross_os}glibc-devel

%description
%{summary}.

%prep
%autosetup -n socat-%{version} -p1

%build
%cross_configure \
  CFLAGS="-Wformat ${CFLAGS}" \
  --enable-help \
  --enable-ip4 \
  --enable-ip6 \
  --enable-stdio \
  --enable-tcp \
  --enable-udp \
  --disable-abstract-unixsocket \
  --disable-creat \
  --disable-exec \
  --disable-ext2 \
  --disable-fdnum \
  --disable-filan \
  --disable-file \
  --disable-fips \
  --disable-genericsocket \
  --disable-gopen \
  --disable-interface \
  --disable-listen \
  --disable-libwrap \
  --disable-openssl \
  --disable-pipe \
  --disable-proxy \
  --disable-pty \
  --disable-rawip \
  --disable-readline \
  --disable-retry \
  --disable-sctp \
  --disable-socks4 \
  --disable-socks4a \
  --disable-sycls \
  --disable-system \
  --disable-termios \
  --disable-tun \
  --disable-unix \

%make_build

%install
%make_install

%files
%license COPYING
%{_cross_attribution_file}
%{_cross_bindir}/socat
%exclude %{_cross_bindir}/filan
%exclude %{_cross_bindir}/procan
%exclude %{_cross_mandir}/*

%changelog
