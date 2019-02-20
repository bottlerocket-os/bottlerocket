FROM fedora:latest AS origin
RUN dnf makecache && dnf -y update

FROM origin AS base
RUN dnf -y install rpmdevtools dnf-plugins-core \
   && dnf -y groupinstall "C Development Tools and Libraries" \
   && useradd builder

FROM base AS rpmbuild
ARG PACKAGE
ARG HASH
WORKDIR /home/builder

USER builder
COPY ./packages/${PACKAGE}/* .
COPY ./packages/rpmmacros .rpmmacros
RUN rpmdev-setuptree \
   && mv *.spec rpmbuild/SPECS \
   && find . -maxdepth 1 -not -path '*/\.*' -type f -exec mv {} rpmbuild/SOURCES/ \; \
   && echo ${HASH}

USER root
RUN dnf -y builddep rpmbuild/SPECS/${PACKAGE}.spec

USER builder
RUN rpmbuild -ba rpmbuild/SPECS/${PACKAGE}.spec

FROM scratch AS rpm
COPY --from=rpmbuild /home/builder/rpmbuild/RPMS/*/*.rpm /
