# syntax=docker/dockerfile:experimental

FROM fedora:30 AS origin
RUN dnf makecache && dnf -y update

FROM origin AS base
RUN dnf -y groupinstall "C Development Tools and Libraries" \
   && dnf -y install \
        rpmdevtools dnf-plugins-core createrepo_c \
        cmake git meson perl-ExtUtils-MakeMaker python which \
        bc hostname intltool grub2-tools gperf kmod rsync wget \
        elfutils-devel libcap-devel openssl-devel \
   && useradd builder

FROM origin AS util
RUN dnf -y install createrepo_c e2fsprogs gdisk grub2-tools kpartx lz4 veritysetup dosfstools mtools

FROM base AS rpmbuild
ARG PACKAGE
ARG ARCH
ARG NOCACHE
WORKDIR /home/builder

USER builder
ENV PACKAGE=${PACKAGE} ARCH=${ARCH}
COPY ./macros/${ARCH} ./macros/shared ./macros/rust ./macros/cargo ./packages/${PACKAGE}/* .
RUN rpmdev-setuptree \
   && cat ${ARCH} shared rust cargo > .rpmmacros \
   && rm ${ARCH} shared rust cargo \
   && mv *.spec rpmbuild/SPECS \
   && find . -maxdepth 1 -not -path '*/\.*' -type f -exec mv {} rpmbuild/SOURCES/ \; \
   && echo ${NOCACHE}

USER root
RUN --mount=target=/host \
    ln -s /host/build/*.rpm ./rpmbuild/RPMS \
    && createrepo_c \
        -o ./rpmbuild/RPMS \
        -x '*-debuginfo-*.rpm' \
        -x '*-debugsource-*.rpm' \
        --no-database \
        /host/build \
    && cp .rpmmacros /etc/rpm/macros \
    && dnf -y \
        --disablerepo '*' \
        --repofrompath repo,./rpmbuild/RPMS \
        --enablerepo 'repo' \
        --nogpgcheck \
        builddep rpmbuild/SPECS/${PACKAGE}.spec

USER builder
RUN --mount=source=.cargo,target=/home/builder/.cargo \
    --mount=type=cache,target=/home/builder/.cache,uid=1000,id=${PACKAGE} \
    --mount=source=workspaces,target=/home/builder/rpmbuild/BUILD/workspaces \
    rpmbuild -ba --clean rpmbuild/SPECS/${PACKAGE}.spec

FROM scratch AS rpm
COPY --from=rpmbuild /home/builder/rpmbuild/RPMS/*/*.rpm /

FROM util AS imgbuild
ARG PACKAGES
ARG ARCH
ARG NOCACHE
WORKDIR /root

USER root
RUN --mount=target=/host \
    mkdir -p /local/rpms ./rpmbuild/RPMS \
    && ln -s /host/build/*.rpm ./rpmbuild/RPMS \
    && createrepo_c \
        -o ./rpmbuild/RPMS \
        -x '*-debuginfo-*.rpm' \
        -x '*-debugsource-*.rpm' \
        --no-database \
        /host/build \
    && dnf -y \
        --disablerepo '*' \
        --repofrompath repo,./rpmbuild/RPMS \
        --enablerepo 'repo' \
        --nogpgcheck \
        --downloadonly \
        --downloaddir . \
        install $(printf "thar-${ARCH}-%s\n" ${PACKAGES}) \
    && mv *.rpm /local/rpms \
    && createrepo_c /local/rpms \
    && /host/bin/rpm2img \
        --package-dir=/local/rpms \
        --output-dir=/local/output \
    && echo ${NOCACHE}

FROM scratch AS image
COPY --from=imgbuild /local/output/* /
