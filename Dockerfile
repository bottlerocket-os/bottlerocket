FROM fedora:latest AS origin
RUN dnf makecache && dnf -y update

FROM origin AS base
RUN dnf -y install rpmdevtools dnf-plugins-core \
   && dnf -y groupinstall "C Development Tools and Libraries" \
   && useradd builder

FROM base AS rpmbuild
ARG PACKAGE
ARG ARCH
ARG HASH
WORKDIR /home/builder

USER builder
COPY ./macros/${ARCH} ./macros/shared ./packages/${PACKAGE}/* .
RUN rpmdev-setuptree \
   && cat ${ARCH} shared > .rpmmacros \
   && rm ${ARCH} shared \
   && mv *.spec rpmbuild/SPECS \
   && find . -maxdepth 1 -not -path '*/\.*' -type f -exec mv {} rpmbuild/SOURCES/ \; \
   && echo ${HASH}

USER root
RUN dnf -y builddep rpmbuild/SPECS/${PACKAGE}.spec

USER builder
RUN rpmbuild -ba --clean rpmbuild/SPECS/${PACKAGE}.spec

FROM scratch AS rpm
COPY --from=rpmbuild /home/builder/rpmbuild/RPMS/*/*.rpm /
