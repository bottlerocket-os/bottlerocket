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
ARG HASH
ARG RPMS
WORKDIR /home/builder

USER builder
ENV PACKAGE=${PACKAGE} ARCH=${ARCH}
COPY ./macros/${ARCH} ./macros/shared ./macros/rust ./macros/cargo ./packages/${PACKAGE}/* .
RUN rpmdev-setuptree \
   && cat ${ARCH} shared rust cargo > .rpmmacros \
   && rm ${ARCH} shared rust cargo \
   && mv *.spec rpmbuild/SPECS \
   && find . -maxdepth 1 -not -path '*/\.*' -type f -exec mv {} rpmbuild/SOURCES/ \; \
   && echo ${HASH}

USER root
RUN --mount=target=/host \
    for rpm in ${RPMS} ; do cp -a /host/build/${rpm##*/} rpmbuild/RPMS ; done \
    && createrepo_c rpmbuild/RPMS \
    && chown -R builder: rpmbuild/RPMS \
    && cp .rpmmacros /etc/rpm/macros \
    && dnf -y \
        --disablerepo '*' \
        --repofrompath repo,./rpmbuild/RPMS \
        --enablerepo 'repo' --nogpgcheck \
        builddep rpmbuild/SPECS/${PACKAGE}.spec

USER builder
RUN rpmbuild -ba --clean rpmbuild/SPECS/${PACKAGE}.spec

FROM scratch AS rpm
COPY --from=rpmbuild /home/builder/rpmbuild/RPMS/*/*.rpm /

FROM util AS imgbuild
ARG PACKAGE
ARG ARCH
ARG HASH
WORKDIR /root

USER root
RUN --mount=target=/host \
    mkdir -p {/local/,}rpms \
    && cp /host/build/*-${ARCH}-*.rpm rpms \
    && createrepo_c rpms \
    && dnf -y \
        --disablerepo '*' \
        --repofrompath repo,rpms \
        --enablerepo 'repo' --nogpgcheck \
        --downloadonly \
        --downloaddir . \
        install ${PACKAGE} \
    && mv *.rpm /local/rpms \
    && createrepo_c /local/rpms \
    && /host/bin/rpm2img \
        --package-dir=/local/rpms \
        --output-dir=/local/output \
    && echo ${HASH}

FROM scratch AS image
COPY --from=imgbuild /local/output/* /
