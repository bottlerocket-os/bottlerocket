FROM fedora:30 as base

# Everything we need to build our SDK and packages.
RUN \
  dnf makecache && \
  dnf -y update && \
  dnf -y groupinstall "C Development Tools and Libraries" && \
  dnf -y install \
    rpmdevtools dnf-plugins-core createrepo_c \
    cmake git meson perl-ExtUtils-MakeMaker python which \
    bc hostname intltool grub2-tools gperf kmod rsync wget \
    elfutils-devel libcap-devel openssl-devel \
    createrepo_c e2fsprogs gdisk grub2-tools \
    kpartx lz4 veritysetup dosfstools mtools && \
  dnf clean all && \
  useradd builder

FROM base as toolchain
USER builder

# Configure Git for any subsequent use.
RUN \
  git config --global user.name "Builder" && \
  git config --global user.email "builder@localhost"

ARG BRVER="2019.08.2"
ARG KVER="4.19.81"

WORKDIR /home/builder
COPY ./hashes ./
RUN \
  curl -OJL https://github.com/buildroot/buildroot/archive/${BRVER}.tar.gz && \
  grep buildroot-${BRVER}.tar.gz hashes | sha512sum --check - && \
  tar xf buildroot-${BRVER}.tar.gz && \
  rm buildroot-${BRVER}.tar.gz && \
  mv buildroot-${BRVER} buildroot

WORKDIR /home/builder/buildroot
COPY ./patches/buildroot/* ./
COPY ./configs/buildroot/* ./configs/
RUN \
  git init . && \
  git apply --whitespace=nowarn *.patch

FROM toolchain as toolchain-gnu
ARG ARCH
RUN \
  make O=output/${ARCH}-gnu defconfig BR2_DEFCONFIG=configs/sdk_${ARCH}_gnu_defconfig && \
  make O=output/${ARCH}-gnu toolchain && \
  find output/${ARCH}-gnu/build/linux-headers-${KVER}/usr/include -name '.*' -delete

FROM toolchain as toolchain-musl
ARG ARCH
RUN \
  make O=output/${ARCH}-musl defconfig BR2_DEFCONFIG=configs/sdk_${ARCH}_musl_defconfig && \
  make O=output/${ARCH}-musl toolchain && \
  find output/${ARCH}-musl/build/linux-headers-${KVER}/usr/include -name '.*' -delete

# Add our cross-compilers to the base SDK layer.
FROM base as sdk
USER root

ARG ARCH
ARG KVER="4.19.81"

WORKDIR /

COPY --from=toolchain-gnu \
  /home/builder/buildroot/output/${ARCH}-gnu/toolchain/ /
COPY --from=toolchain-gnu \
  /home/builder/buildroot/output/${ARCH}-gnu/build/linux-headers-${KVER}/usr/include/ \
  /${ARCH}-thar-linux-gnu/sys-root/usr/include/

COPY --from=toolchain-musl \
  /home/builder/buildroot/output/${ARCH}-musl/toolchain/ /
COPY --from=toolchain-musl \
  /home/builder/buildroot/output/${ARCH}-musl/build/linux-headers-${KVER}/usr/include/ \
  /${ARCH}-thar-linux-musl/sys-root/usr/include/

# Build C libraries so we can build our rust and golang toolchains.
FROM sdk as sdk-gnu
USER builder

ARG GLIBCVER="2.30"

WORKDIR /home/builder
COPY ./hashes ./
RUN \
  curl -OJL https://ftp.gnu.org/gnu/glibc/glibc-${GLIBCVER}.tar.xz && \
  grep glibc-${GLIBCVER}.tar.xz hashes | sha512sum --check - && \
  tar xf glibc-${GLIBCVER}.tar.xz && \
  rm glibc-${GLIBCVER}.tar.xz && \
  mv glibc-${GLIBCVER} glibc && \
  cd glibc && \
  mkdir build

ARG ARCH
ARG TARGET="${ARCH}-thar-linux-gnu"
ARG SYSROOT="/${TARGET}/sys-root"
ARG CFLAGS="-O2 -g -Wp,-D_GLIBCXX_ASSERTIONS -fstack-clash-protection"
ARG CXXFLAGS="${CFLAGS}"
ARG CPPFLAGS=""
ARG KVER="4.19"

WORKDIR /home/builder/glibc/build
RUN \
  ../configure \
    --prefix="${SYSROOT}/usr" \
    --sysconfdir="/etc" \
    --localstatedir="/var" \
    --target="${TARGET}" \
    --host="${TARGET}" \
    --with-headers="/${SYSROOT}/usr/include" \
    --enable-bind-now \
    --enable-kernel="${KVER}" \
    --enable-shared \
    --enable-stack-protector=strong \
    --disable-crypt \
    --disable-multi-arch \
    --disable-profile \
    --disable-systemtap \
    --disable-timezone-tools \
    --disable-tunables \
    --without-cvs \
    --without-gd \
    --without-selinux && \
  make -j$(nproc) -O -r

USER root
WORKDIR /home/builder/glibc/build
RUN make install

FROM sdk as sdk-musl
USER builder

ARG MUSLVER="1.1.24"

WORKDIR /home/builder
COPY ./hashes ./
RUN \
  curl -OJL https://www.musl-libc.org/releases/musl-${MUSLVER}.tar.gz && \
  grep musl-${MUSLVER}.tar.gz hashes | sha512sum --check - && \
  tar xf musl-${MUSLVER}.tar.gz && \
  rm musl-${MUSLVER}.tar.gz && \
  mv musl-${MUSLVER} musl

ARG TARGET="${ARCH}-thar-linux-musl"
ARG SYSROOT="/${TARGET}/sys-root"
ARG CFLAGS="-O2 -g -pipe -Wall -Werror=format-security -Wp,-D_FORTIFY_SOURCE=2 -Wp,-D_GLIBCXX_ASSERTIONS -fexceptions -fstack-clash-protection"
ARG LDFLAGS="-Wl,-z,relro -Wl,-z,now"

WORKDIR /home/builder/musl
RUN \
  ./configure \
    CFLAGS="${CFLAGS}" \
    LDFLAGS="${LDFLAGS}" \
    --target="${TARGET}" \
    --disable-gcc-wrapper \
    --enable-static \
    --prefix="${SYSROOT}/usr" \
    --libdir="${SYSROOT}/usr/lib" && \
   make -j$(nproc)

USER root
WORKDIR /home/builder/musl
RUN make install

ARG LLVMVER="9.0.0"

USER builder
WORKDIR /home/builder
COPY ./hashes ./

# Rust's musl targets depend on libunwind.
RUN \
  curl -OJL https://releases.llvm.org/${LLVMVER}/llvm-${LLVMVER}.src.tar.xz && \
  grep llvm-${LLVMVER}.src.tar.xz hashes | sha512sum --check - && \
  tar xf llvm-${LLVMVER}.src.tar.xz && \
  rm llvm-${LLVMVER}.src.tar.xz && \
  mv llvm-${LLVMVER}.src llvm && \
  curl -OJL https://releases.llvm.org/${LLVMVER}/libunwind-${LLVMVER}.src.tar.xz && \
  grep libunwind-${LLVMVER}.src.tar.xz hashes | sha512sum --check - && \
  tar xf libunwind-${LLVMVER}.src.tar.xz && \
  rm libunwind-${LLVMVER}.src.tar.xz && \
  mv libunwind-${LLVMVER}.src libunwind && \
  mkdir libunwind/build

WORKDIR /home/builder/libunwind/build
RUN \
  cmake \
    -DLLVM_PATH=../../llvm \
    -DLIBUNWIND_ENABLE_SHARED=1 \
    -DLIBUNWIND_ENABLE_STATIC=1 \
    -DCMAKE_INSTALL_PREFIX="/usr" \
    -DCMAKE_C_COMPILER="${TARGET}-gcc" \
    -DCMAKE_C_COMPILER_TARGET="${TARGET}" \
    -DCMAKE_CXX_COMPILER="${TARGET}-g++" \
    -DCMAKE_CXX_COMPILER_TARGET="${TARGET}" \
    -DCMAKE_AR="/usr/bin/${TARGET}-ar" \
    -DCMAKE_RANLIB="/usr/bin/${TARGET}-ranlib" \
    .. && \
  make unwind

USER root
WORKDIR /home/builder/libunwind/build
RUN make install-unwind DESTDIR="${SYSROOT}"

FROM sdk as sdk-libc

ARG GNU_TARGET="${ARCH}-thar-linux-gnu"
ARG GNU_SYSROOT="/${GNU_TARGET}/sys-root"
ARG MUSL_TARGET="${ARCH}-thar-linux-musl"
ARG MUSL_SYSROOT="/${MUSL_TARGET}/sys-root"

COPY --from=sdk-gnu ${GNU_SYSROOT}/ ${GNU_SYSROOT}/
COPY --from=sdk-musl ${MUSL_SYSROOT}/ ${MUSL_SYSROOT}/

FROM sdk-libc as sdk-rust

USER root
RUN \
  mkdir -p /usr/libexec/rust && \
  chown -R builder:builder /usr/libexec/rust

ARG ARCH
ARG TARGET="${ARCH}-thar-linux-gnu"
ARG RUSTVER="1.39.0"

USER builder
WORKDIR /home/builder
COPY ./hashes ./
RUN \
  curl -OJL https://static.rust-lang.org/dist/rustc-${RUSTVER}-src.tar.xz && \
  grep rustc-${RUSTVER}-src.tar.xz hashes | sha512sum --check - && \
  tar xf rustc-${RUSTVER}-src.tar.xz && \
  rm rustc-${RUSTVER}-src.tar.xz && \
  mv rustc-${RUSTVER}-src rust

WORKDIR /home/builder/rust
COPY ./configs/rust/* ./
RUN \
  cp config-${ARCH}.toml config.toml && \
  ./x.py install

FROM sdk-libc as sdk-go

ARG ARCH
ARG TARGET="${ARCH}-thar-linux-gnu"
ARG GOVER="1.13.4"

USER root
RUN dnf -y install golang

USER builder
WORKDIR /home/builder
COPY ./hashes ./
RUN \
  curl -OJL https://dl.google.com/go/go${GOVER}.src.tar.gz && \
  grep go${GOVER}.src.tar.gz hashes | sha512sum --check - && \
  tar xf go${GOVER}.src.tar.gz && \
  rm go${GOVER}.src.tar.gz

ARG GOROOT_FINAL="/usr/libexec/go"
ARG GOOS="linux"
ARG CGO_ENABLED=1
ARG GOARCH_aarch64="arm64"
ARG GOARCH_x86_64="amd64"
ARG GOARCH_ARCH="GOARCH_${ARCH}"
ARG CFLAGS="-O2 -g -pipe -Wall -Werror=format-security -Wp,-D_FORTIFY_SOURCE=2 -Wp,-D_GLIBCXX_ASSERTIONS -fexceptions -fstack-clash-protection"
ARG CXXFLAGS="${CFLAGS}"
ARG LDFLAGS="-Wl,-z,relro -Wl,-z,now"
ARG CGO_CFLAGS="${CFLAGS}"
ARG CGO_CXXFLAGS="${CXXFLAGS}"
ARG CGO_LDFLAGS="${LDFLAGS}"

WORKDIR /home/builder/go/src
RUN ./make.bash --no-clean

# Build the standard library with and without PIE. Target binaries
# should use PIE, but any host binaries generated during the build
# might not.
WORKDIR /home/builder/go
RUN \
  export GOARCH="${!GOARCH_ARCH}" ; \
  export CC="${TARGET}-gcc" ; \
  export CC_FOR_TARGET="${TARGET}-gcc" ; \
  export CC_FOR_${GOOS}_${GOARCH}="${TARGET}-gcc" ; \
  export CXX="${TARGET}-g++" ; \
  export CXX_FOR_TARGET="${TARGET}-g++" ; \
  export CXX_FOR_${GOOS}_${GOARCH}="${TARGET}-g++" ; \
  export GOFLAGS="-mod=vendor" ; \
  export GOPROXY="off" ; \
  export GOSUMDB="off" ; \
  export GOROOT="${PWD}" ; \
  export PATH="${PWD}/bin:${PATH}" ; \
  go install std && \
  go install -buildmode=pie std

# Collect all builds in a single layer.
FROM scratch as sdk-final
USER root

ARG ARCH
ARG GNU_TARGET="${ARCH}-thar-linux-gnu"
ARG GNU_SYSROOT="/${GNU_TARGET}/sys-root"
ARG MUSL_TARGET="${ARCH}-thar-linux-musl"
ARG MUSL_SYSROOT="/${MUSL_TARGET}/sys-root"

WORKDIR /
# "sdk" has our C/C++ toolchain and kernel headers for both targets.
COPY --from=sdk / /

# "sdk-musl" has a superset of the above, and includes C library and headers.
# We omit "sdk-gnu" because we expect to build glibc again for the target OS,
# while we will use the musl artifacts directly to generate static binaries
# such as migrations.
COPY --chown=0:0 --from=sdk-musl ${MUSL_SYSROOT}/ ${MUSL_SYSROOT}/

# "sdk-rust" has our Rust toolchain with the required targets.
COPY --chown=0:0 --from=sdk-rust /usr/libexec/rust/ /usr/libexec/rust/

# "sdk-go" has the Go toolchain and standard library builds.
COPY --chown=0:0 --from=sdk-go /home/builder/go/bin /usr/libexec/go/bin/
COPY --chown=0:0 --from=sdk-go /home/builder/go/lib /usr/libexec/go/lib/
COPY --chown=0:0 --from=sdk-go /home/builder/go/pkg /usr/libexec/go/pkg/
COPY --chown=0:0 --from=sdk-go /home/builder/go/src /usr/libexec/go/src/

# Add Rust programs and libraries to the path.
RUN \
  for b in /usr/libexec/rust/bin/* ; do \
    ln -s ../libexec/rust/bin/${b##*/} /usr/bin/${b##*/} ; \
  done && \
  echo '/usr/libexec/rust/lib' > /etc/ld.so.conf.d/rust.conf && \
  ldconfig

# Strip and deduplicate Rust's LLVM libraries.
RUN \
  export HOSTDIR="/usr/libexec/rust/lib/rustlib/x86_64-unknown-linux-gnu/" ; \
  diff /usr/libexec/rust/lib/libLLVM-*.so ${HOSTDIR}/lib/libLLVM-*.so && \
  strip -g /usr/libexec/rust/lib/libLLVM-*.so && \
  ln -f /usr/libexec/rust/lib/libLLVM-*.so ${HOSTDIR}/lib/ && \
  strip -g ${HOSTDIR}/codegen-backends/librustc_codegen_llvm-llvm.so

# Add Go programs to $PATH and sync timestamps to avoid rebuilds.
RUN \
  ln -s ../libexec/go/bin/go /usr/bin/go && \
  ln -s ../libexec/go/bin/gofmt /usr/bin/gofmt && \
  find /usr/libexec/go -type f -exec touch -r /usr/libexec/go/bin/go {} \+

# Add target binutils to $PATH to override programs used to extract debuginfo.
RUN \
  ln -s ../../${GNU_TARGET}/bin/nm /usr/local/bin/nm && \
  ln -s ../../${GNU_TARGET}/bin/objcopy /usr/local/bin/objcopy && \
  ln -s ../../${GNU_TARGET}/bin/objdump /usr/local/bin/objdump && \
  ln -s ../../${GNU_TARGET}/bin/strip /usr/local/bin/strip

# Reset permissions for `builder`.
RUN chown builder:builder -R /home/builder

USER builder
RUN rpmdev-setuptree

CMD ["/bin/bash"]
