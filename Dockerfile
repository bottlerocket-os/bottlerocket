# syntax=docker/dockerfile:experimental

FROM fedora:latest AS origin
RUN dnf makecache && dnf -y update

FROM origin AS base
RUN dnf -y install rpmdevtools dnf-plugins-core createrepo_c \
   && dnf -y groupinstall "C Development Tools and Libraries" \
   && useradd builder

FROM base AS rpmbuild
ARG PACKAGE
ARG ARCH
ARG HASH
ARG RPMS
WORKDIR /home/builder

USER builder
ENV PACKAGE=${PACKAGE} ARCH=${ARCH}
COPY ./macros/${ARCH} ./macros/shared ./packages/${PACKAGE}/* .
RUN rpmdev-setuptree \
   && cat ${ARCH} shared > .rpmmacros \
   && rm ${ARCH} shared \
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
