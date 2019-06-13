# syntax=docker/dockerfile:experimental

FROM fedora:30 AS origin
ARG DATE
RUN dnf makecache && dnf -y update && echo ${DATE}

FROM origin AS base
RUN dnf -y groupinstall "C Development Tools and Libraries" \
   && dnf -y install \
        rpmdevtools dnf-plugins-core createrepo_c \
        git rsync which cmake meson \
   && useradd builder

FROM origin AS util
RUN dnf -y install e2fsprogs gdisk grub2-tools kpartx lz4 veritysetup

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
    && dnf -y --repofrompath repo,./rpmbuild/RPMS --nogpgcheck builddep rpmbuild/SPECS/${PACKAGE}.spec

USER builder
RUN rpmbuild -ba --clean rpmbuild/SPECS/${PACKAGE}.spec

FROM scratch AS rpm
COPY --from=rpmbuild /home/builder/rpmbuild/RPMS/*/*.rpm /

FROM base AS imgbuild
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
        --repofrompath repo,rpms \
        --repo repo --nogpgcheck \
        --downloadonly \
        --downloaddir . \
        install ${PACKAGE} \
    && mv *.rpm /local/rpms \
    && createrepo_c /local/rpms \
    && cp /host/bin/rpm2img /local \
    && echo ${HASH}

FROM util AS builder
COPY --from=imgbuild /local/ /local/
ENTRYPOINT ["/local/rpm2img"]
CMD []
